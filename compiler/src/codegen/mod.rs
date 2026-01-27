//! Code generation module using Cranelift

#![allow(
    clippy::similar_names,
    clippy::match_same_arms,
    clippy::missing_errors_doc,
    clippy::unused_self
)]

mod class;
mod function;

use std::collections::HashMap;
use std::path::Path;

use cranelift::prelude::*;
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::ast::{Function, Program, QualifiedName, Type};
use crate::errors::CompileError;
use crate::types::{build_class_registry, ClassRegistry};
use class::ClassCodeGen;
use function::FunctionCompiler;
use miette::Result;

/// Compile a program to a native executable.
pub fn compile(program: &Program, output_path: &Path, emit_ir: bool) -> Result<()> {
    let mut codegen = CodeGen::new()?;
    codegen.compile_program(program)?;

    if emit_ir {
        eprintln!("=== Cranelift IR ===");
    }

    codegen.write_object(output_path)?;
    Ok(())
}

struct CodeGen {
    module: ObjectModule,
    ctx: codegen::Context,
    functions: HashMap<String, FuncId>,
    data_id_counter: u32,
    class_codegen: ClassCodeGen,
    class_registry: ClassRegistry,
    /// Maps PHP function names to their intrinsic runtime function names
    intrinsics: HashMap<String, String>,
}

impl CodeGen {
    fn new() -> Result<Self> {
        let isa_builder = cranelift_native::builder().map_err(|e| CompileError::CodegenError {
            message: format!("Failed to create ISA builder: {e}"),
        })?;

        let isa = isa_builder
            .finish(settings::Flags::new(settings::builder()))
            .map_err(|e| CompileError::CodegenError {
                message: format!("Failed to create ISA: {e}"),
            })?;

        let builder = ObjectBuilder::new(isa, "main", cranelift_module::default_libcall_names())
            .map_err(|e| CompileError::CodegenError {
                message: format!("Failed to create object builder: {e}"),
            })?;

        let module = ObjectModule::new(builder);
        let ctx = module.make_context();

        Ok(Self {
            module,
            ctx,
            functions: HashMap::new(),
            data_id_counter: 0,
            class_codegen: ClassCodeGen::new(),
            class_registry: ClassRegistry::new(),
            intrinsics: HashMap::new(),
        })
    }

    fn compile_program(&mut self, program: &Program) -> Result<()> {
        // Build class registry
        self.class_registry = build_class_registry(program);

        // Declare runtime functions
        self.declare_printf()?;
        self.declare_runtime_functions()?;

        // Declare class methods (including trait methods)
        self.class_codegen.declare_methods(
            &program.classes,
            &program.traits,
            &self.class_registry,
            &mut self.module,
        )?;

        // Create vtables for all classes (after methods are declared)
        self.class_codegen
            .create_vtables(&self.class_registry, &mut self.module)?;

        // Create static properties
        self.class_codegen
            .create_static_properties(&program.classes, &mut self.module)?;

        // Copy method FuncIds to functions map
        for (name, func_id) in &self.class_codegen.methods {
            self.functions.insert(name.clone(), *func_id);
        }

        // First pass: declare all functions
        for func in &program.functions {
            self.declare_function(func)?;
        }

        // Second pass: compile class methods
        for class in &program.classes {
            for method in &class.methods {
                if !method.is_abstract && method.body.is_some() {
                    self.compile_method(class, method)?;
                }
            }

            // Compile trait methods for this class
            for trait_use in &class.trait_uses {
                for trait_qn in &trait_use.traits {
                    let trait_name = trait_qn.full_path();
                    if let Some(trait_def) = program.traits.iter().find(|t| {
                        t.qualified_name
                            .as_ref()
                            .map_or(t.name == trait_name, |qn: &crate::ast::QualifiedName| {
                                qn.full_path() == trait_name
                            })
                    }) {
                        for method in &trait_def.methods {
                            // Only compile if class doesn't override
                            if !class.methods.iter().any(|m| m.name == method.name)
                                && !method.is_abstract
                                && method.body.is_some()
                            {
                                self.compile_trait_method(class, method)?;
                            }
                        }
                    }
                }
            }
        }

        // Third pass: compile function bodies
        for func in &program.functions {
            self.compile_function(func)?;
        }

        Ok(())
    }

    fn compile_method(
        &mut self,
        class: &crate::ast::ClassDef,
        method: &crate::ast::Method,
    ) -> Result<()> {
        // Use qualified name for mangling if available
        let class_key = class
            .qualified_name
            .as_ref()
            .map_or_else(|| class.name.clone(), QualifiedName::mangle);
        let mangled_name = format!("{}_{}", class_key, method.name);
        let func_id =
            *self
                .functions
                .get(&mangled_name)
                .ok_or_else(|| CompileError::CodegenError {
                    message: format!("Method {mangled_name} not found"),
                })?;

        self.ctx.func.signature = self
            .module
            .declarations()
            .get_function_decl(func_id)
            .signature
            .clone();

        let mut builder_ctx = FunctionBuilderContext::new();
        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut builder_ctx);

            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            let mut compiler = FunctionCompiler::new(
                &mut builder,
                &mut self.module,
                &self.functions,
                &mut self.data_id_counter,
            );

            // Set class context (use qualified name for proper parent resolution)
            compiler.current_class = Some(
                class
                    .qualified_name
                    .as_ref()
                    .map_or_else(|| class.name.clone(), QualifiedName::full_path),
            );
            compiler.class_registry = Some(&self.class_registry);
            compiler.class_codegen = Some(&self.class_codegen);
            compiler.intrinsics = Some(&self.intrinsics);

            let mut param_idx = 0;

            // Add $this for non-static methods
            if !method.is_static {
                let this_val = compiler.builder.block_params(entry_block)[param_idx];
                let this_var = compiler.declare_variable("this", &Type::Class(class.name.clone()));
                compiler.builder.def_var(this_var, this_val);
                param_idx += 1;
            }

            // Add parameters as variables
            for param in &method.params {
                let val = compiler.builder.block_params(entry_block)[param_idx];
                let var = compiler.declare_variable(&param.name, &param.ty);
                compiler.builder.def_var(var, val);
                param_idx += 1;
            }

            // Compile body
            if let Some(body) = &method.body {
                for stmt in body {
                    compiler.compile_stmt(stmt)?;
                }
            }

            // Add implicit return if needed
            if !compiler.terminated {
                if method.return_type == Type::Void {
                    compiler.builder.ins().return_(&[]);
                } else {
                    let default_val = compiler.default_value(&method.return_type);
                    compiler.builder.ins().return_(&[default_val]);
                }
            }

            builder.finalize();
        }

        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| CompileError::CodegenError {
                message: format!("Failed to define method {mangled_name}: {e}"),
            })?;

        self.module.clear_context(&mut self.ctx);
        Ok(())
    }

    fn compile_trait_method(
        &mut self,
        class: &crate::ast::ClassDef,
        method: &crate::ast::Method,
    ) -> Result<()> {
        // Use the class's qualified name for mangling
        let class_key = class
            .qualified_name
            .as_ref()
            .map_or_else(|| class.name.clone(), QualifiedName::mangle);
        let mangled_name = format!("{}_{}", class_key, method.name);
        let func_id =
            *self
                .functions
                .get(&mangled_name)
                .ok_or_else(|| CompileError::CodegenError {
                    message: format!("Trait method {mangled_name} not found"),
                })?;

        self.ctx.func.signature = self
            .module
            .declarations()
            .get_function_decl(func_id)
            .signature
            .clone();

        let mut builder_ctx = FunctionBuilderContext::new();
        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut builder_ctx);

            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            let mut compiler = FunctionCompiler::new(
                &mut builder,
                &mut self.module,
                &self.functions,
                &mut self.data_id_counter,
            );

            // Set class context to the class using the trait
            compiler.current_class = Some(
                class
                    .qualified_name
                    .as_ref()
                    .map_or_else(|| class.name.clone(), QualifiedName::full_path),
            );
            compiler.class_registry = Some(&self.class_registry);
            compiler.class_codegen = Some(&self.class_codegen);
            compiler.intrinsics = Some(&self.intrinsics);

            let mut param_idx = 0;

            // Add $this for non-static methods
            if !method.is_static {
                let this_val = compiler.builder.block_params(entry_block)[param_idx];
                let this_var = compiler.declare_variable("this", &Type::Class(class.name.clone()));
                compiler.builder.def_var(this_var, this_val);
                param_idx += 1;
            }

            // Add parameters as variables
            for param in &method.params {
                let val = compiler.builder.block_params(entry_block)[param_idx];
                let var = compiler.declare_variable(&param.name, &param.ty);
                compiler.builder.def_var(var, val);
                param_idx += 1;
            }

            // Compile body
            if let Some(body) = &method.body {
                for stmt in body {
                    compiler.compile_stmt(stmt)?;
                }
            }

            // Add implicit return if needed
            if !compiler.terminated {
                if method.return_type == Type::Void {
                    compiler.builder.ins().return_(&[]);
                } else {
                    let default_val = compiler.default_value(&method.return_type);
                    compiler.builder.ins().return_(&[default_val]);
                }
            }

            builder.finalize();
        }

        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| CompileError::CodegenError {
                message: format!("Failed to define trait method {mangled_name}: {e}"),
            })?;

        self.module.clear_context(&mut self.ctx);
        Ok(())
    }

    fn declare_printf(&mut self) -> Result<()> {
        let ptr_type = self.module.target_config().pointer_type();
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(ptr_type));
        sig.returns.push(AbiParam::new(types::I32));

        let func_id = self
            .module
            .declare_function("printf", Linkage::Import, &sig)
            .map_err(|e| CompileError::CodegenError {
                message: format!("Failed to declare printf: {e}"),
            })?;

        self.functions.insert("printf".to_string(), func_id);
        Ok(())
    }

    fn declare_runtime_functions(&mut self) -> Result<()> {
        let ptr_type = self.module.target_config().pointer_type();

        // === libc functions ===
        self.declare_runtime_func("strlen", &[ptr_type], Some(types::I64))?;
        self.declare_runtime_func("malloc", &[types::I64], Some(ptr_type))?;
        self.declare_runtime_func("strcpy", &[ptr_type, ptr_type], Some(ptr_type))?;
        self.declare_runtime_func("strcat", &[ptr_type, ptr_type], Some(ptr_type))?;
        self.declare_runtime_func("strcmp", &[ptr_type, ptr_type], Some(types::I32))?;
        self.declare_runtime_func("sprintf", &[ptr_type, ptr_type], Some(types::I32))?;
        self.declare_runtime_func("_gcvt", &[types::F64, types::I32, ptr_type], Some(ptr_type))?;
        self.declare_runtime_func("puts", &[ptr_type], Some(types::I32))?;
        self.declare_runtime_func("fputs", &[ptr_type, ptr_type], Some(types::I32))?;

        // === PHPRS Runtime string functions (C-string wrappers) ===
        self.declare_runtime_func("rt_cstr_len", &[ptr_type], Some(types::I64))?;
        self.declare_runtime_func(
            "rt_cstr_substr",
            &[ptr_type, types::I64, types::I64],
            Some(ptr_type),
        )?;
        self.declare_runtime_func("rt_cstr_strpos", &[ptr_type, ptr_type], Some(types::I64))?;
        self.declare_runtime_func("rt_cstr_contains", &[ptr_type, ptr_type], Some(types::I8))?;
        self.declare_runtime_func(
            "rt_cstr_starts_with",
            &[ptr_type, ptr_type],
            Some(types::I8),
        )?;
        self.declare_runtime_func("rt_cstr_ends_with", &[ptr_type, ptr_type], Some(types::I8))?;
        self.declare_runtime_func("rt_cstr_tolower", &[ptr_type], Some(ptr_type))?;
        self.declare_runtime_func("rt_cstr_toupper", &[ptr_type], Some(ptr_type))?;
        self.declare_runtime_func("rt_cstr_trim", &[ptr_type], Some(ptr_type))?;
        self.declare_runtime_func("rt_cstr_ltrim", &[ptr_type], Some(ptr_type))?;
        self.declare_runtime_func("rt_cstr_rtrim", &[ptr_type], Some(ptr_type))?;
        self.declare_runtime_func(
            "rt_cstr_replace",
            &[ptr_type, ptr_type, ptr_type],
            Some(ptr_type),
        )?;
        self.declare_runtime_func("rt_cstr_ord", &[ptr_type], Some(types::I64))?;
        self.declare_runtime_func("rt_cstr_chr", &[types::I64], Some(ptr_type))?;
        self.declare_runtime_func("rt_cstr_rev", &[ptr_type], Some(ptr_type))?;
        self.declare_runtime_func("rt_cstr_repeat", &[ptr_type, types::I64], Some(ptr_type))?;
        self.declare_runtime_func("rt_cstr_cmp", &[ptr_type, ptr_type], Some(types::I32))?;
        self.declare_runtime_func("rt_cstr_free", &[ptr_type], None)?;

        // === PHPRS Runtime math functions ===
        self.declare_runtime_func("rt_abs", &[types::I64], Some(types::I64))?;
        self.declare_runtime_func("rt_fabs", &[types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_min", &[types::I64, types::I64], Some(types::I64))?;
        self.declare_runtime_func("rt_max", &[types::I64, types::I64], Some(types::I64))?;
        self.declare_runtime_func("rt_fmin", &[types::F64, types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_fmax", &[types::F64, types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_round", &[types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_floor", &[types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_ceil", &[types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_trunc", &[types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_sqrt", &[types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_pow", &[types::F64, types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_powi", &[types::F64, types::I32], Some(types::F64))?;
        self.declare_runtime_func("rt_exp", &[types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_log", &[types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_log10", &[types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_sin", &[types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_cos", &[types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_tan", &[types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_asin", &[types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_acos", &[types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_atan", &[types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_atan2", &[types::F64, types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_rand", &[], Some(types::I64))?;
        self.declare_runtime_func("rt_srand", &[types::I64], None)?;
        self.declare_runtime_func("rt_rand_range", &[types::I64, types::I64], Some(types::I64))?;
        self.declare_runtime_func("rt_fmod", &[types::F64, types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_hypot", &[types::F64, types::F64], Some(types::F64))?;
        self.declare_runtime_func("rt_is_finite", &[types::F64], Some(types::I8))?;
        self.declare_runtime_func("rt_is_nan", &[types::F64], Some(types::I8))?;
        self.declare_runtime_func("rt_is_infinite", &[types::F64], Some(types::I8))?;

        // === PHPRS Runtime type conversion functions ===
        self.declare_runtime_func("rt_int_to_float", &[types::I64], Some(types::F64))?;
        self.declare_runtime_func("rt_float_to_int", &[types::F64], Some(types::I64))?;
        self.declare_runtime_func("rt_bool_to_int", &[types::I8], Some(types::I64))?;
        self.declare_runtime_func("rt_int_to_bool", &[types::I64], Some(types::I8))?;
        self.declare_runtime_func("rt_float_to_bool", &[types::F64], Some(types::I8))?;
        self.declare_runtime_func("rt_cstr_to_int", &[ptr_type], Some(types::I64))?;
        self.declare_runtime_func("rt_cstr_to_float", &[ptr_type], Some(types::F64))?;
        self.declare_runtime_func("rt_cstr_to_bool", &[ptr_type], Some(types::I8))?;
        self.declare_runtime_func("rt_int_to_cstr", &[types::I64], Some(ptr_type))?;
        self.declare_runtime_func("rt_float_to_cstr", &[types::F64], Some(ptr_type))?;
        self.declare_runtime_func("rt_bool_to_cstr", &[types::I8], Some(ptr_type))?;

        // === PHPRS Runtime array functions ===
        self.declare_runtime_func("rt_array_new", &[], Some(ptr_type))?;
        self.declare_runtime_func("rt_array_with_capacity", &[types::I64], Some(ptr_type))?;
        self.declare_runtime_func("rt_array_len", &[ptr_type], Some(types::I64))?;
        self.declare_runtime_func("rt_array_free", &[ptr_type], None)?;
        self.declare_runtime_func("rt_count", &[ptr_type], Some(types::I64))?;
        self.declare_runtime_func("rt_array_sum", &[ptr_type], Some(types::I64))?;
        self.declare_runtime_func("rt_array_sum_float", &[ptr_type], Some(types::F64))?;
        self.declare_runtime_func("rt_array_product", &[ptr_type], Some(types::I64))?;
        self.declare_runtime_func("rt_array_product_float", &[ptr_type], Some(types::F64))?;
        self.declare_runtime_func("rt_in_array", &[ptr_type, ptr_type], Some(types::I8))?;
        self.declare_runtime_func("rt_in_array_strict", &[ptr_type, ptr_type], Some(types::I8))?;
        self.declare_runtime_func(
            "rt_array_key_exists_int",
            &[types::I64, ptr_type],
            Some(types::I8),
        )?;
        self.declare_runtime_func(
            "rt_array_key_exists_str",
            &[ptr_type, ptr_type],
            Some(types::I8),
        )?;
        self.declare_runtime_func("rt_array_push_value", &[ptr_type, ptr_type], None)?;
        self.declare_runtime_func("rt_array_pop", &[ptr_type], Some(ptr_type))?;
        self.declare_runtime_func("rt_array_shift", &[ptr_type], Some(ptr_type))?;
        self.declare_runtime_func("rt_array_get_int", &[ptr_type, types::I64], Some(ptr_type))?;
        self.declare_runtime_func("rt_array_set_int", &[ptr_type, types::I64, ptr_type], None)?;
        self.declare_runtime_func("rt_array_get_str", &[ptr_type, ptr_type], Some(ptr_type))?;
        self.declare_runtime_func("rt_array_set_str", &[ptr_type, ptr_type, ptr_type], None)?;
        self.declare_runtime_func("rt_array_first", &[ptr_type], Some(ptr_type))?;
        self.declare_runtime_func("rt_array_last", &[ptr_type], Some(ptr_type))?;
        self.declare_runtime_func("rt_array_search", &[ptr_type, ptr_type], Some(types::I64))?;

        Ok(())
    }

    fn declare_runtime_func(
        &mut self,
        name: &str,
        params: &[types::Type],
        ret: Option<types::Type>,
    ) -> Result<()> {
        let mut sig = self.module.make_signature();
        for &p in params {
            sig.params.push(AbiParam::new(p));
        }
        if let Some(r) = ret {
            sig.returns.push(AbiParam::new(r));
        }

        let func_id = self
            .module
            .declare_function(name, Linkage::Import, &sig)
            .map_err(|e| CompileError::CodegenError {
                message: format!("Failed to declare {name}: {e}"),
            })?;

        self.functions.insert(name.to_string(), func_id);
        Ok(())
    }

    fn declare_function(&mut self, func: &Function) -> Result<()> {
        // Check for #[Intrinsic("rt_func_name")] attribute
        if let Some(intrinsic_name) = func.attributes.get_intrinsic() {
            // Store the mapping for later use during calls
            self.intrinsics
                .insert(func.name.clone(), intrinsic_name.to_string());

            // Declare the runtime function if not already declared
            if !self.functions.contains_key(intrinsic_name) {
                let mut sig = self.module.make_signature();
                for param in &func.params {
                    let ty = self.cranelift_type(&param.ty);
                    sig.params.push(AbiParam::new(ty));
                }
                if func.return_type != Type::Void {
                    let ret_ty = self.cranelift_type(&func.return_type);
                    sig.returns.push(AbiParam::new(ret_ty));
                }

                let func_id = self
                    .module
                    .declare_function(intrinsic_name, Linkage::Import, &sig)
                    .map_err(|e| CompileError::CodegenError {
                        message: format!("Failed to declare intrinsic {intrinsic_name}: {e}"),
                    })?;

                self.functions.insert(intrinsic_name.to_string(), func_id);
            }

            // Also register the PHP function name pointing to the same FuncId
            if let Some(&func_id) = self.functions.get(intrinsic_name) {
                self.functions.insert(func.name.clone(), func_id);
            }

            return Ok(());
        }

        let mut sig = self.module.make_signature();

        for param in &func.params {
            let ty = self.cranelift_type(&param.ty);
            sig.params.push(AbiParam::new(ty));
        }

        // main returns i32 for exit code
        if func.name == "main" {
            sig.returns.push(AbiParam::new(types::I32));
        } else if func.return_type != Type::Void {
            let ret_ty = self.cranelift_type(&func.return_type);
            sig.returns.push(AbiParam::new(ret_ty));
        }

        let linkage = if func.name == "main" {
            Linkage::Export
        } else {
            Linkage::Local
        };

        let func_id = self
            .module
            .declare_function(&func.name, linkage, &sig)
            .map_err(|e| CompileError::CodegenError {
                message: format!("Failed to declare function {}: {e}", func.name),
            })?;

        self.functions.insert(func.name.clone(), func_id);
        Ok(())
    }

    fn compile_function(&mut self, func: &Function) -> Result<()> {
        // Skip compilation for intrinsic functions - they forward to runtime
        if func.attributes.get_intrinsic().is_some() {
            return Ok(());
        }

        let func_id = *self.functions.get(&func.name).unwrap();

        self.ctx.func.signature = self
            .module
            .declarations()
            .get_function_decl(func_id)
            .signature
            .clone();

        let mut builder_ctx = FunctionBuilderContext::new();
        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut builder_ctx);

            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            let mut compiler = FunctionCompiler::new(
                &mut builder,
                &mut self.module,
                &self.functions,
                &mut self.data_id_counter,
            );

            // Set class context for OOP support
            compiler.class_registry = Some(&self.class_registry);
            compiler.class_codegen = Some(&self.class_codegen);
            compiler.intrinsics = Some(&self.intrinsics);

            // Initialize vtables at the start of main()
            if func.name == "main" {
                compiler.emit_vtable_init(&self.functions)?;
            }

            // Add parameters as variables
            for (i, param) in func.params.iter().enumerate() {
                let val = compiler.builder.block_params(entry_block)[i];
                let var = compiler.declare_variable(&param.name, &param.ty);
                compiler.builder.def_var(var, val);
            }

            // Compile body
            for stmt in &func.body {
                compiler.compile_stmt(stmt)?;
            }

            // Add implicit return if needed
            if !compiler.terminated {
                if func.name == "main" {
                    let zero = compiler.builder.ins().iconst(types::I32, 0);
                    compiler.builder.ins().return_(&[zero]);
                } else if func.return_type == Type::Void {
                    compiler.builder.ins().return_(&[]);
                } else {
                    let default_val = compiler.default_value(&func.return_type);
                    compiler.builder.ins().return_(&[default_val]);
                }
            }

            builder.finalize();
        }

        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| CompileError::CodegenError {
                message: format!("Failed to define function {}: {e}", func.name),
            })?;

        self.module.clear_context(&mut self.ctx);
        Ok(())
    }

    fn cranelift_type(&self, ty: &Type) -> types::Type {
        match ty {
            Type::Int => types::I64,
            Type::Float => types::F64,
            Type::Bool => types::I8,
            Type::String => self.module.target_config().pointer_type(),
            Type::Void => types::I64,
            Type::Ref(_) | Type::RefMut(_) => self.module.target_config().pointer_type(),
            Type::Class(_) | Type::Interface(_) => self.module.target_config().pointer_type(),
            Type::Nullable(inner) => self.cranelift_type(inner),
            Type::Array(_) => self.module.target_config().pointer_type(),
            Type::Closure(_, _) => self.module.target_config().pointer_type(), // Function pointer
            Type::SelfType | Type::StaticType => self.module.target_config().pointer_type(),
            Type::Unknown => types::I64,
        }
    }

    fn write_object(&mut self, output_path: &Path) -> Result<()> {
        let product = std::mem::replace(&mut self.module, {
            let isa_builder = cranelift_native::builder().unwrap();
            let isa = isa_builder
                .finish(settings::Flags::new(settings::builder()))
                .unwrap();
            let builder =
                ObjectBuilder::new(isa, "main", cranelift_module::default_libcall_names()).unwrap();
            ObjectModule::new(builder)
        })
        .finish();

        let obj_bytes = product.emit().map_err(|e| CompileError::CodegenError {
            message: format!("Failed to emit object: {e}"),
        })?;

        let obj_path = output_path.with_extension("o");
        std::fs::write(&obj_path, obj_bytes).map_err(|e| CompileError::CodegenError {
            message: format!("Failed to write object file: {e}"),
        })?;

        self.link(&obj_path, output_path)?;

        let _ = std::fs::remove_file(&obj_path);

        Ok(())
    }

    /// Get the path to the runtime library
    fn get_runtime_lib_path() -> Option<std::path::PathBuf> {
        // Try to find the runtime library in various locations
        let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();

        // Check relative to executable
        let candidates = [
            // Same directory as executable (workspace target/debug or target/release)
            exe_dir.join("libphprs_runtime.a"),
            exe_dir.join("phprs_runtime.lib"),
            // Parent directory (in case exe is in deps/)
            exe_dir.join("../libphprs_runtime.a"),
            exe_dir.join("../phprs_runtime.lib"),
            // Lib subdirectory (installed version)
            exe_dir.join("lib/libphprs_runtime.a"),
            exe_dir.join("lib/phprs_runtime.lib"),
        ];

        for path in &candidates {
            if path.exists() {
                return Some(path.clone());
            }
        }

        None
    }

    #[cfg(target_os = "windows")]
    fn link(&self, obj_path: &Path, output_path: &Path) -> Result<()> {
        let runtime_lib = Self::get_runtime_lib_path();

        // Try gcc first (MinGW)
        let mut gcc_args = vec![
            "-o".to_string(),
            output_path.display().to_string(),
            obj_path.display().to_string(),
        ];
        if let Some(ref lib_path) = runtime_lib {
            gcc_args.push(lib_path.display().to_string());
        }
        gcc_args.extend([
            "-lmsvcrt".to_string(),
            "-lws2_32".to_string(),
            "-luserenv".to_string(),
            "-lntdll".to_string(),
            "-lbcrypt".to_string(),
            "-ladvapi32".to_string(),
            "-lkernel32".to_string(),
        ]);

        if let Ok(status) = std::process::Command::new("gcc").args(&gcc_args).status() {
            if status.success() {
                return Ok(());
            }
        }

        // Try clang
        let mut clang_args = vec![
            "-o".to_string(),
            output_path.display().to_string(),
            obj_path.display().to_string(),
        ];
        if let Some(ref lib_path) = runtime_lib {
            clang_args.push(lib_path.display().to_string());
        }
        clang_args.extend([
            "-lws2_32".to_string(),
            "-luserenv".to_string(),
            "-lntdll".to_string(),
            "-lbcrypt".to_string(),
            "-ladvapi32".to_string(),
            "-lkernel32".to_string(),
        ]);

        if let Ok(status) = std::process::Command::new("clang")
            .args(&clang_args)
            .status()
        {
            if status.success() {
                return Ok(());
            }
        }

        // Try MSVC link
        let mut link_args = vec![
            "/NOLOGO".to_string(),
            "/ENTRY:main".to_string(),
            "/SUBSYSTEM:CONSOLE".to_string(),
            format!("/OUT:{}", output_path.display()),
            obj_path.display().to_string(),
        ];
        if let Some(ref lib_path) = runtime_lib {
            link_args.push(lib_path.display().to_string());
        }
        link_args.extend([
            "msvcrt.lib".to_string(),
            "legacy_stdio_definitions.lib".to_string(),
            "ws2_32.lib".to_string(),
            "userenv.lib".to_string(),
            "ntdll.lib".to_string(),
            "bcrypt.lib".to_string(),
            "advapi32.lib".to_string(),
            "kernel32.lib".to_string(),
        ]);

        if let Ok(status) = std::process::Command::new("link").args(&link_args).status() {
            if status.success() {
                return Ok(());
            }
        }

        Err(CompileError::CodegenError {
            message: "Linking failed. Please install gcc, clang, or MSVC".to_string(),
        }
        .into())
    }

    #[cfg(not(target_os = "windows"))]
    fn link(&self, obj_path: &Path, output_path: &Path) -> Result<()> {
        let runtime_lib = Self::get_runtime_lib_path();

        let mut args = vec![
            "-o".to_string(),
            output_path.display().to_string(),
            obj_path.display().to_string(),
        ];

        if let Some(ref lib_path) = runtime_lib {
            args.push(lib_path.display().to_string());
        }

        // Link with math library
        args.push("-lm".to_string());

        let status = std::process::Command::new("cc")
            .args(&args)
            .status()
            .map_err(|e| CompileError::CodegenError {
                message: format!("Failed to run linker: {e}"),
            })?;

        if !status.success() {
            return Err(CompileError::CodegenError {
                message: "Linking failed".to_string(),
            }
            .into());
        }
        Ok(())
    }
}
