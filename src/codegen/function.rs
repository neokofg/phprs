//! Function compilation for Cranelift codegen

#![allow(
    clippy::similar_names,
    clippy::match_same_arms,
    clippy::unnecessary_wraps,
    clippy::wrong_self_convention,
    clippy::ref_option
)]

use std::collections::HashMap;

use cranelift::prelude::*;
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::ObjectModule;

use crate::ast::{BinaryOp, Expr, ExprKind, Stmt, StmtKind, Type, UnaryOp};
use crate::errors::CompileError;
use miette::Result;

pub struct FunctionCompiler<'a, 'b> {
    pub builder: &'a mut FunctionBuilder<'b>,
    pub module: &'a mut ObjectModule,
    pub variables: HashMap<String, Variable>,
    pub var_counter: usize,
    pub functions: &'a HashMap<String, FuncId>,
    pub data_id_counter: &'a mut u32,
    pub terminated: bool,
}

impl<'a, 'b> FunctionCompiler<'a, 'b> {
    pub fn new(
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

    pub fn declare_variable(&mut self, name: &str, ty: &Type) -> Variable {
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

    pub fn compile_stmt(&mut self, stmt: &Stmt) -> Result<()> {
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
                self.compile_if(condition, then_branch, else_branch)?;
            }

            StmtKind::While { condition, body } => {
                self.compile_while(condition, body)?;
            }

            StmtKind::For {
                init,
                condition,
                update,
                body,
            } => {
                self.compile_for(init, condition, update, body)?;
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

    fn compile_if(
        &mut self,
        condition: &Expr,
        then_branch: &[Stmt],
        else_branch: &Option<Vec<Stmt>>,
    ) -> Result<()> {
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
        Ok(())
    }

    fn compile_while(&mut self, condition: &Expr, body: &[Stmt]) -> Result<()> {
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
        Ok(())
    }

    fn compile_for(
        &mut self,
        init: &Option<Box<Stmt>>,
        condition: &Option<Expr>,
        update: &Option<Expr>,
        body: &[Stmt],
    ) -> Result<()> {
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
        Ok(())
    }

    pub fn compile_expr(&mut self, expr: &Expr) -> Result<Value> {
        match &expr.kind {
            ExprKind::IntLit(v) => Ok(self.builder.ins().iconst(types::I64, *v)),
            ExprKind::FloatLit(v) => Ok(self.builder.ins().f64const(*v)),
            ExprKind::BoolLit(v) => Ok(self.builder.ins().iconst(types::I8, i64::from(*v))),
            ExprKind::StringLit(s) => self.compile_string_lit(s),
            ExprKind::Null => {
                let ptr_ty = self.module.target_config().pointer_type();
                Ok(self.builder.ins().iconst(ptr_ty, 0))
            }
            ExprKind::Variable(name) => self.compile_variable(name),
            ExprKind::Binary { left, op, right } => {
                let lhs = self.compile_expr(left)?;
                let rhs = self.compile_expr(right)?;
                self.compile_binary_op(*op, lhs, rhs)
            }
            ExprKind::Unary { op, operand } => {
                let val = self.compile_expr(operand)?;
                self.compile_unary_op(*op, val)
            }
            ExprKind::Call { name, args } => self.compile_call(name, args),
            ExprKind::Ref(inner) | ExprKind::RefMut(inner) => self.compile_expr(inner),
            ExprKind::Assign { target, value } => self.compile_assign_expr(target, value),
            ExprKind::PrefixOp { op, target } => self.compile_prefix_op(*op, target),
            ExprKind::PostfixOp { op, target } => self.compile_postfix_op(*op, target),
            // OOP expressions
            ExprKind::New { .. } => self.compile_new(),
            ExprKind::This => self.compile_this(),
            ExprKind::PropertyAccess { object, .. } => self.compile_property_access(object),
            ExprKind::MethodCall { object, args, .. } => self.compile_method_call(object, args),
            ExprKind::StaticPropertyAccess { .. } => self.compile_static_property(),
            ExprKind::StaticMethodCall { args, .. } => self.compile_static_method(args),
            ExprKind::PropertyAssign { object, value, .. } => {
                self.compile_property_assign(object, value)
            }
            ExprKind::ArrayLit(elements) => self.compile_array_lit(elements),
            ExprKind::ArrayAccess { array, index } => self.compile_array_access(array, index),
        }
    }

    fn compile_string_lit(&mut self, s: &str) -> Result<Value> {
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

    fn compile_variable(&mut self, name: &str) -> Result<Value> {
        if let Some(&var) = self.variables.get(name) {
            Ok(self.builder.use_var(var))
        } else {
            Err(CompileError::CodegenError {
                message: format!("Undefined variable: {name}"),
            }
            .into())
        }
    }

    fn compile_call(&mut self, name: &str, args: &[Expr]) -> Result<Value> {
        let func_id = self
            .functions
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

    fn compile_assign_expr(&mut self, target: &str, value: &Expr) -> Result<Value> {
        let val = self.compile_expr(value)?;
        if let Some(&var) = self.variables.get(target) {
            self.builder.def_var(var, val);
        }
        Ok(val)
    }

    fn compile_prefix_op(&mut self, op: UnaryOp, target: &str) -> Result<Value> {
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

    fn compile_postfix_op(&mut self, op: UnaryOp, target: &str) -> Result<Value> {
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

    // OOP placeholder implementations
    fn compile_new(&mut self) -> Result<Value> {
        let ptr_ty = self.module.target_config().pointer_type();
        Ok(self.builder.ins().iconst(ptr_ty, 0))
    }

    fn compile_this(&mut self) -> Result<Value> {
        let ptr_ty = self.module.target_config().pointer_type();
        Ok(self.builder.ins().iconst(ptr_ty, 0))
    }

    fn compile_property_access(&mut self, object: &Expr) -> Result<Value> {
        let _obj = self.compile_expr(object)?;
        let ptr_ty = self.module.target_config().pointer_type();
        Ok(self.builder.ins().iconst(ptr_ty, 0))
    }

    fn compile_method_call(&mut self, object: &Expr, args: &[Expr]) -> Result<Value> {
        let _obj = self.compile_expr(object)?;
        for arg in args {
            let _ = self.compile_expr(arg)?;
        }
        let ptr_ty = self.module.target_config().pointer_type();
        Ok(self.builder.ins().iconst(ptr_ty, 0))
    }

    fn compile_static_property(&mut self) -> Result<Value> {
        let ptr_ty = self.module.target_config().pointer_type();
        Ok(self.builder.ins().iconst(ptr_ty, 0))
    }

    fn compile_static_method(&mut self, args: &[Expr]) -> Result<Value> {
        for arg in args {
            let _ = self.compile_expr(arg)?;
        }
        let ptr_ty = self.module.target_config().pointer_type();
        Ok(self.builder.ins().iconst(ptr_ty, 0))
    }

    fn compile_property_assign(&mut self, object: &Expr, value: &Expr) -> Result<Value> {
        let _obj = self.compile_expr(object)?;
        let val = self.compile_expr(value)?;
        Ok(val)
    }

    fn compile_array_lit(&mut self, elements: &[crate::ast::ArrayElement]) -> Result<Value> {
        for elem in elements {
            if let Some(key) = &elem.key {
                let _ = self.compile_expr(key)?;
            }
            let _ = self.compile_expr(&elem.value)?;
        }
        let ptr_ty = self.module.target_config().pointer_type();
        Ok(self.builder.ins().iconst(ptr_ty, 0))
    }

    fn compile_array_access(&mut self, array: &Expr, index: &Expr) -> Result<Value> {
        let _arr = self.compile_expr(array)?;
        let _idx = self.compile_expr(index)?;
        Ok(self.builder.ins().iconst(types::I64, 0))
    }

    pub fn compile_binary_op(&mut self, op: BinaryOp, lhs: Value, rhs: Value) -> Result<Value> {
        let lhs_ty = self.builder.func.dfg.value_type(lhs);
        let is_float = lhs_ty == types::F64;

        if is_float {
            self.compile_float_binary_op(op, lhs, rhs)
        } else {
            self.compile_int_binary_op(op, lhs, rhs)
        }
    }

    fn compile_float_binary_op(&mut self, op: BinaryOp, lhs: Value, rhs: Value) -> Result<Value> {
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
    }

    fn compile_int_binary_op(&mut self, op: BinaryOp, lhs: Value, rhs: Value) -> Result<Value> {
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
                return self.concat_strings(lhs, rhs);
            }
        };
        Ok(result)
    }

    fn concat_strings(&mut self, lhs: Value, rhs: Value) -> Result<Value> {
        let strlen_id = *self.functions.get("strlen").unwrap();
        let strlen_ref = self.module.declare_func_in_func(strlen_id, self.builder.func);

        let malloc_id = *self.functions.get("malloc").unwrap();
        let malloc_ref = self.module.declare_func_in_func(malloc_id, self.builder.func);

        let strcpy_id = *self.functions.get("strcpy").unwrap();
        let strcpy_ref = self.module.declare_func_in_func(strcpy_id, self.builder.func);

        let strcat_id = *self.functions.get("strcat").unwrap();
        let strcat_ref = self.module.declare_func_in_func(strcat_id, self.builder.func);

        // Get lengths
        let len1_call = self.builder.ins().call(strlen_ref, &[lhs]);
        let len1 = self.builder.inst_results(len1_call)[0];

        let len2_call = self.builder.ins().call(strlen_ref, &[rhs]);
        let len2 = self.builder.inst_results(len2_call)[0];

        // total_len = len1 + len2 + 1
        let total = self.builder.ins().iadd(len1, len2);
        let one = self.builder.ins().iconst(types::I64, 1);
        let total_with_null = self.builder.ins().iadd(total, one);

        // Allocate memory
        let malloc_call = self.builder.ins().call(malloc_ref, &[total_with_null]);
        let result_ptr = self.builder.inst_results(malloc_call)[0];

        // strcpy + strcat
        self.builder.ins().call(strcpy_ref, &[result_ptr, lhs]);
        self.builder.ins().call(strcat_ref, &[result_ptr, rhs]);

        Ok(result_ptr)
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

    pub fn emit_print(&mut self, val: Value, ty: &Type) -> Result<()> {
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
            Type::Int | Type::Unknown => "%lld",
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

    pub fn to_bool(&mut self, val: Value) -> Value {
        let ty = self.builder.func.dfg.value_type(val);
        if ty == types::I8 {
            val
        } else {
            let zero = self.builder.ins().iconst(ty, 0);
            self.builder.ins().icmp(IntCC::NotEqual, val, zero)
        }
    }

    pub fn default_value(&mut self, ty: &Type) -> Value {
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
