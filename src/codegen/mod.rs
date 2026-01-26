use std::collections::HashMap;
use std::path::Path;

use cranelift::prelude::*;
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::ast::{BinaryOp, Expr, ExprKind, Function, Program, Stmt, StmtKind, Type, UnaryOp};
use crate::errors::CompileError;
use miette::Result;

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
        // Declare printf
        self.declare_printf()?;

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
                    // main returns exit code 0
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

struct FunctionCompiler<'a, 'b> {
    builder: &'a mut FunctionBuilder<'b>,
    module: &'a mut ObjectModule,
    variables: HashMap<String, Variable>,
    var_counter: usize,
    functions: &'a HashMap<String, FuncId>,
    data_id_counter: &'a mut u32,
    terminated: bool,
}

impl<'a, 'b> FunctionCompiler<'a, 'b> {
    fn new(
        builder: &'a mut FunctionBuilder<'b>,
        module: &'a mut ObjectModule,
        functions: &'a HashMap<String, FuncId>,
        data_id_counter: &'a mut u32,
    ) -> Self {
        Self {
            builder,
            module,
            variables: HashMap::new(),
            var_counter: 0,
            functions,
            data_id_counter,
            terminated: false,
        }
    }

    fn declare_variable(&mut self, name: &str, ty: &Type) -> Variable {
        let var = Variable::new(self.var_counter);
        self.var_counter += 1;

        let cl_ty = match ty {
            Type::Int => types::I64,
            Type::Float => types::F64,
            Type::Bool => types::I8,
            Type::String => self.module.target_config().pointer_type(),
            _ => types::I64,
        };

        self.builder.declare_var(var, cl_ty);
        self.variables.insert(name.to_string(), var);
        var
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        if self.terminated {
            return Ok(());
        }

        match &stmt.kind {
            StmtKind::Let { name, ty, init } => {
                let ty = ty.as_ref().unwrap_or(&Type::Int);
                let var = self.declare_variable(name, ty);
                let val = self.compile_expr(init)?;
                self.builder.def_var(var, val);
            }

            StmtKind::Assign { target, value } => {
                let val = self.compile_expr(value)?;
                if let Some(&var) = self.variables.get(target) {
                    self.builder.def_var(var, val);
                } else {
                    let ty = value.ty.as_ref().unwrap_or(&Type::Int);
                    let var = self.declare_variable(target, ty);
                    self.builder.def_var(var, val);
                }
            }

            StmtKind::CompoundAssign { target, op, value } => {
                let var = *self.variables.get(target).unwrap();
                let current = self.builder.use_var(var);
                let rhs = self.compile_expr(value)?;
                let result = self.compile_binary_op(*op, current, rhs)?;
                self.builder.def_var(var, result);
            }

            StmtKind::Expr(expr) => {
                self.compile_expr(expr)?;
            }

            StmtKind::Return(expr) => {
                if let Some(e) = expr {
                    let val = self.compile_expr(e)?;
                    self.builder.ins().return_(&[val]);
                } else {
                    self.builder.ins().return_(&[]);
                }
                self.terminated = true;
            }

            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond = self.compile_expr(condition)?;
                let cond_bool = self.to_bool(cond);

                let then_block = self.builder.create_block();
                let else_block = self.builder.create_block();
                let merge_block = self.builder.create_block();

                self.builder
                    .ins()
                    .brif(cond_bool, then_block, &[], else_block, &[]);

                // Then branch
                self.builder.switch_to_block(then_block);
                self.builder.seal_block(then_block);
                let mut then_terminated = false;
                for s in then_branch {
                    self.compile_stmt(s)?;
                    if self.terminated {
                        then_terminated = true;
                        break;
                    }
                }
                if !then_terminated {
                    self.builder.ins().jump(merge_block, &[]);
                }
                self.terminated = false;

                // Else branch
                self.builder.switch_to_block(else_block);
                self.builder.seal_block(else_block);
                let mut else_terminated = false;
                if let Some(else_stmts) = else_branch {
                    for s in else_stmts {
                        self.compile_stmt(s)?;
                        if self.terminated {
                            else_terminated = true;
                            break;
                        }
                    }
                }
                if !else_terminated {
                    self.builder.ins().jump(merge_block, &[]);
                }
                self.terminated = false;

                self.builder.switch_to_block(merge_block);
                self.builder.seal_block(merge_block);

                // If both branches return, we're terminated
                self.terminated = then_terminated && else_terminated;
            }

            StmtKind::While { condition, body } => {
                let header_block = self.builder.create_block();
                let body_block = self.builder.create_block();
                let exit_block = self.builder.create_block();

                self.builder.ins().jump(header_block, &[]);

                // Header
                self.builder.switch_to_block(header_block);
                let cond = self.compile_expr(condition)?;
                let cond_bool = self.to_bool(cond);
                self.builder
                    .ins()
                    .brif(cond_bool, body_block, &[], exit_block, &[]);

                // Body
                self.builder.switch_to_block(body_block);
                self.builder.seal_block(body_block);
                for s in body {
                    self.compile_stmt(s)?;
                    if self.terminated {
                        break;
                    }
                }
                if !self.terminated {
                    self.builder.ins().jump(header_block, &[]);
                }
                self.terminated = false;

                self.builder.seal_block(header_block);
                self.builder.switch_to_block(exit_block);
                self.builder.seal_block(exit_block);
            }

            StmtKind::For {
                init,
                condition,
                update,
                body,
            } => {
                // Init
                if let Some(i) = init {
                    self.compile_stmt(i)?;
                }

                let header_block = self.builder.create_block();
                let body_block = self.builder.create_block();
                let update_block = self.builder.create_block();
                let exit_block = self.builder.create_block();

                self.builder.ins().jump(header_block, &[]);

                // Header
                self.builder.switch_to_block(header_block);
                if let Some(c) = condition {
                    let cond = self.compile_expr(c)?;
                    let cond_bool = self.to_bool(cond);
                    self.builder
                        .ins()
                        .brif(cond_bool, body_block, &[], exit_block, &[]);
                } else {
                    self.builder.ins().jump(body_block, &[]);
                }

                // Body
                self.builder.switch_to_block(body_block);
                self.builder.seal_block(body_block);
                for s in body {
                    self.compile_stmt(s)?;
                    if self.terminated {
                        break;
                    }
                }
                if !self.terminated {
                    self.builder.ins().jump(update_block, &[]);
                }
                self.terminated = false;

                // Update
                self.builder.switch_to_block(update_block);
                self.builder.seal_block(update_block);
                if let Some(u) = update {
                    self.compile_expr(u)?;
                }
                self.builder.ins().jump(header_block, &[]);

                self.builder.seal_block(header_block);
                self.builder.switch_to_block(exit_block);
                self.builder.seal_block(exit_block);
            }

            StmtKind::Echo(exprs) => {
                for expr in exprs {
                    let val = self.compile_expr(expr)?;
                    self.emit_print(val, expr.ty.as_ref().unwrap_or(&Type::Int))?;
                }
            }

            StmtKind::Block(stmts) => {
                for s in stmts {
                    self.compile_stmt(s)?;
                    if self.terminated {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<Value> {
        match &expr.kind {
            ExprKind::IntLit(v) => Ok(self.builder.ins().iconst(types::I64, *v)),

            ExprKind::FloatLit(v) => Ok(self.builder.ins().f64const(*v)),

            ExprKind::BoolLit(v) => Ok(self.builder.ins().iconst(types::I8, i64::from(*v))),

            ExprKind::StringLit(s) => {
                let ptr_ty = self.module.target_config().pointer_type();

                let data_name = format!("str_{}", *self.data_id_counter);
                *self.data_id_counter += 1;

                let mut data_bytes = s.as_bytes().to_vec();
                data_bytes.push(0);

                let data_id = self
                    .module
                    .declare_data(&data_name, Linkage::Local, false, false)
                    .map_err(|e| CompileError::CodegenError {
                        message: format!("Failed to declare string data: {e}"),
                    })?;

                let mut data_description = cranelift_module::DataDescription::new();
                data_description.define(data_bytes.into_boxed_slice());

                self.module
                    .define_data(data_id, &data_description)
                    .map_err(|e| CompileError::CodegenError {
                        message: format!("Failed to define string data: {e}"),
                    })?;

                let local_id = self.module.declare_data_in_func(data_id, self.builder.func);
                let ptr = self.builder.ins().symbol_value(ptr_ty, local_id);
                Ok(ptr)
            }

            ExprKind::Null => {
                let ptr_ty = self.module.target_config().pointer_type();
                Ok(self.builder.ins().iconst(ptr_ty, 0))
            }

            ExprKind::Variable(name) => {
                if let Some(&var) = self.variables.get(name) {
                    Ok(self.builder.use_var(var))
                } else {
                    Err(CompileError::CodegenError {
                        message: format!("Undefined variable: {name}"),
                    }
                    .into())
                }
            }

            ExprKind::Binary { left, op, right } => {
                let lhs = self.compile_expr(left)?;
                let rhs = self.compile_expr(right)?;
                self.compile_binary_op(*op, lhs, rhs)
            }

            ExprKind::Unary { op, operand } => {
                let val = self.compile_expr(operand)?;
                self.compile_unary_op(*op, val)
            }

            ExprKind::Call { name, args } => {
                let func_id =
                    self.functions
                        .get(name)
                        .ok_or_else(|| CompileError::CodegenError {
                            message: format!("Unknown function: {name}"),
                        })?;

                let func_ref = self
                    .module
                    .declare_func_in_func(*func_id, self.builder.func);

                let compiled_args: Vec<Value> = args
                    .iter()
                    .map(|a| self.compile_expr(a))
                    .collect::<Result<Vec<_>>>()?;

                let call = self.builder.ins().call(func_ref, &compiled_args);
                let results = self.builder.inst_results(call);

                if results.is_empty() {
                    Ok(self.builder.ins().iconst(types::I64, 0))
                } else {
                    Ok(results[0])
                }
            }

            ExprKind::Ref(inner) | ExprKind::RefMut(inner) => self.compile_expr(inner),

            ExprKind::Assign { target, value } => {
                let val = self.compile_expr(value)?;
                if let Some(&var) = self.variables.get(target) {
                    self.builder.def_var(var, val);
                }
                Ok(val)
            }

            ExprKind::PrefixOp { op, target } => {
                let var = *self.variables.get(target).unwrap();
                let current = self.builder.use_var(var);

                let one = self.builder.ins().iconst(types::I64, 1);
                let result = match op {
                    UnaryOp::Inc => self.builder.ins().iadd(current, one),
                    UnaryOp::Dec => self.builder.ins().isub(current, one),
                    _ => unreachable!(),
                };

                self.builder.def_var(var, result);
                Ok(result)
            }

            ExprKind::PostfixOp { op, target } => {
                let var = *self.variables.get(target).unwrap();
                let current = self.builder.use_var(var);

                let one = self.builder.ins().iconst(types::I64, 1);
                let result = match op {
                    UnaryOp::Inc => self.builder.ins().iadd(current, one),
                    UnaryOp::Dec => self.builder.ins().isub(current, one),
                    _ => unreachable!(),
                };

                self.builder.def_var(var, result);
                Ok(current)
            }
        }
    }

    fn compile_binary_op(&mut self, op: BinaryOp, lhs: Value, rhs: Value) -> Result<Value> {
        let lhs_ty = self.builder.func.dfg.value_type(lhs);
        let is_float = lhs_ty == types::F64;

        if is_float {
            let result = match op {
                BinaryOp::Add => self.builder.ins().fadd(lhs, rhs),
                BinaryOp::Sub => self.builder.ins().fsub(lhs, rhs),
                BinaryOp::Mul => self.builder.ins().fmul(lhs, rhs),
                BinaryOp::Div => self.builder.ins().fdiv(lhs, rhs),
                BinaryOp::Mod => {
                    let div = self.builder.ins().fdiv(lhs, rhs);
                    let floor = self.builder.ins().floor(div);
                    let mul = self.builder.ins().fmul(floor, rhs);
                    self.builder.ins().fsub(lhs, mul)
                }
                BinaryOp::Eq => self.builder.ins().fcmp(FloatCC::Equal, lhs, rhs),
                BinaryOp::Ne => self.builder.ins().fcmp(FloatCC::NotEqual, lhs, rhs),
                BinaryOp::Lt => self.builder.ins().fcmp(FloatCC::LessThan, lhs, rhs),
                BinaryOp::Le => self.builder.ins().fcmp(FloatCC::LessThanOrEqual, lhs, rhs),
                BinaryOp::Gt => self.builder.ins().fcmp(FloatCC::GreaterThan, lhs, rhs),
                BinaryOp::Ge => self
                    .builder
                    .ins()
                    .fcmp(FloatCC::GreaterThanOrEqual, lhs, rhs),
                _ => {
                    return Err(CompileError::CodegenError {
                        message: format!("Unsupported float operation: {op}"),
                    }
                    .into());
                }
            };
            Ok(result)
        } else {
            let result = match op {
                BinaryOp::Add => self.builder.ins().iadd(lhs, rhs),
                BinaryOp::Sub => self.builder.ins().isub(lhs, rhs),
                BinaryOp::Mul => self.builder.ins().imul(lhs, rhs),
                BinaryOp::Div => self.builder.ins().sdiv(lhs, rhs),
                BinaryOp::Mod => self.builder.ins().srem(lhs, rhs),
                BinaryOp::Eq => self.builder.ins().icmp(IntCC::Equal, lhs, rhs),
                BinaryOp::Ne => self.builder.ins().icmp(IntCC::NotEqual, lhs, rhs),
                BinaryOp::Lt => self.builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs),
                BinaryOp::Le => self
                    .builder
                    .ins()
                    .icmp(IntCC::SignedLessThanOrEqual, lhs, rhs),
                BinaryOp::Gt => self.builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs),
                BinaryOp::Ge => self
                    .builder
                    .ins()
                    .icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs),
                BinaryOp::And => self.builder.ins().band(lhs, rhs),
                BinaryOp::Or => self.builder.ins().bor(lhs, rhs),
                BinaryOp::Concat => {
                    return Err(CompileError::CodegenError {
                        message: "String concatenation not implemented".to_string(),
                    }
                    .into());
                }
            };
            Ok(result)
        }
    }

    fn compile_unary_op(&mut self, op: UnaryOp, val: Value) -> Result<Value> {
        let ty = self.builder.func.dfg.value_type(val);

        match op {
            UnaryOp::Neg => {
                if ty == types::F64 {
                    Ok(self.builder.ins().fneg(val))
                } else {
                    Ok(self.builder.ins().ineg(val))
                }
            }
            UnaryOp::Not => {
                let bool_val = self.to_bool(val);
                let one = self.builder.ins().iconst(types::I8, 1);
                Ok(self.builder.ins().bxor(bool_val, one))
            }
            UnaryOp::Inc | UnaryOp::Dec => Ok(val),
        }
    }

    fn emit_print(&mut self, val: Value, ty: &Type) -> Result<()> {
        let printf_id = self
            .functions
            .get("printf")
            .ok_or_else(|| CompileError::CodegenError {
                message: "printf not found".to_string(),
            })?;

        let printf_ref = self
            .module
            .declare_func_in_func(*printf_id, self.builder.func);
        let ptr_ty = self.module.target_config().pointer_type();

        let fmt_str = match ty {
            Type::Int => "%lld",
            Type::Float => "%f",
            Type::Bool => "%d",
            Type::String => "%s",
            _ => "%s",
        };

        let fmt_name = format!("fmt_{}", *self.data_id_counter);
        *self.data_id_counter += 1;

        let mut fmt_bytes = fmt_str.as_bytes().to_vec();
        fmt_bytes.push(0);

        let fmt_id = self
            .module
            .declare_data(&fmt_name, Linkage::Local, false, false)
            .map_err(|e| CompileError::CodegenError {
                message: format!("Failed to declare format string: {e}"),
            })?;

        let mut fmt_desc = cranelift_module::DataDescription::new();
        fmt_desc.define(fmt_bytes.into_boxed_slice());

        self.module
            .define_data(fmt_id, &fmt_desc)
            .map_err(|e| CompileError::CodegenError {
                message: format!("Failed to define format string: {e}"),
            })?;

        let fmt_local = self.module.declare_data_in_func(fmt_id, self.builder.func);
        let fmt_ptr = self.builder.ins().symbol_value(ptr_ty, fmt_local);

        let val_ty = self.builder.func.dfg.value_type(val);
        let mut sig = self.module.make_signature();
        sig.params.push(AbiParam::new(ptr_ty));
        sig.params.push(AbiParam::new(val_ty));
        sig.returns.push(AbiParam::new(types::I32));

        let sig_ref = self.builder.import_signature(sig);
        let printf_ptr = self.builder.ins().func_addr(ptr_ty, printf_ref);

        self.builder
            .ins()
            .call_indirect(sig_ref, printf_ptr, &[fmt_ptr, val]);

        Ok(())
    }

    fn to_bool(&mut self, val: Value) -> Value {
        let ty = self.builder.func.dfg.value_type(val);
        if ty == types::I8 {
            val
        } else {
            let zero = self.builder.ins().iconst(ty, 0);
            self.builder.ins().icmp(IntCC::NotEqual, val, zero)
        }
    }

    fn default_value(&mut self, ty: &Type) -> Value {
        match ty {
            Type::Int => self.builder.ins().iconst(types::I64, 0),
            Type::Float => self.builder.ins().f64const(0.0),
            Type::Bool => self.builder.ins().iconst(types::I8, 0),
            Type::String => {
                let ptr_ty = self.module.target_config().pointer_type();
                self.builder.ins().iconst(ptr_ty, 0)
            }
            _ => self.builder.ins().iconst(types::I64, 0),
        }
    }
}
