//! Code generation module using Cranelift

#![allow(
    clippy::similar_names,
    clippy::match_same_arms,
    clippy::missing_errors_doc,
    clippy::unused_self
)]

mod function;

use std::collections::HashMap;
use std::path::Path;

use cranelift::prelude::*;
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::ast::{Function, Program, Type};
use crate::errors::CompileError;
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
        })
    }

    fn compile_program(&mut self, program: &Program) -> Result<()> {
        // Declare runtime functions
        self.declare_printf()?;
        self.declare_runtime_functions()?;

        // First pass: declare all functions
        for func in &program.functions {
            self.declare_function(func)?;
        }

        // Second pass: compile function bodies
        for func in &program.functions {
            self.compile_function(func)?;
        }

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

        // strlen: (ptr) -> i64
        self.declare_runtime_func("strlen", &[ptr_type], Some(types::I64))?;

        // malloc: (i64) -> ptr
        self.declare_runtime_func("malloc", &[types::I64], Some(ptr_type))?;

        // strcpy: (ptr, ptr) -> ptr
        self.declare_runtime_func("strcpy", &[ptr_type, ptr_type], Some(ptr_type))?;

        // strcat: (ptr, ptr) -> ptr
        self.declare_runtime_func("strcat", &[ptr_type, ptr_type], Some(ptr_type))?;

        // sprintf: (ptr, ptr, ...) -> i32
        self.declare_runtime_func("sprintf", &[ptr_type, ptr_type], Some(types::I32))?;

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

    #[cfg(target_os = "windows")]
    fn link(&self, obj_path: &Path, output_path: &Path) -> Result<()> {
        // Try gcc first (MinGW)
        if let Ok(status) = std::process::Command::new("gcc")
            .args([
                "-o",
                &output_path.display().to_string(),
                &obj_path.display().to_string(),
                "-lmsvcrt",
            ])
            .status()
        {
            if status.success() {
                return Ok(());
            }
        }

        // Try clang
        if let Ok(status) = std::process::Command::new("clang")
            .args([
                "-o",
                &output_path.display().to_string(),
                &obj_path.display().to_string(),
            ])
            .status()
        {
            if status.success() {
                return Ok(());
            }
        }

        // Try MSVC link
        if let Ok(status) = std::process::Command::new("link")
            .args([
                "/NOLOGO",
                "/ENTRY:main",
                "/SUBSYSTEM:CONSOLE",
                &format!("/OUT:{}", output_path.display()),
                &obj_path.display().to_string(),
                "msvcrt.lib",
                "legacy_stdio_definitions.lib",
            ])
            .status()
        {
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
        let status = std::process::Command::new("cc")
            .args([
                "-o",
                &output_path.display().to_string(),
                &obj_path.display().to_string(),
            ])
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
