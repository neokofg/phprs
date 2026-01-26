//! Class code generation - vtables, object allocation, method dispatch

use std::collections::HashMap;

use cranelift::prelude::*;
use cranelift_module::{DataDescription, DataId, FuncId, Linkage, Module};
use cranelift_object::ObjectModule;

use crate::ast::{ClassDef, Method, Type};
use crate::errors::CompileError;
use crate::types::ClassRegistry;
use miette::Result;

/// Holds class-related codegen state
pub struct ClassCodeGen {
    /// Vtable data IDs for each class
    pub vtables: HashMap<String, DataId>,
    /// Static property data IDs: "`ClassName_propName`" -> `DataId`
    pub static_properties: HashMap<String, DataId>,
    /// Method function IDs: "`ClassName_methodName`" -> `FuncId`
    pub methods: HashMap<String, FuncId>,
}

impl ClassCodeGen {
    pub fn new() -> Self {
        Self {
            vtables: HashMap::new(),
            static_properties: HashMap::new(),
            methods: HashMap::new(),
        }
    }

    /// Declare all class methods as functions
    pub fn declare_methods(
        &mut self,
        classes: &[ClassDef],
        _registry: &ClassRegistry,
        module: &mut ObjectModule,
    ) -> Result<()> {
        for class in classes {
            for method in &class.methods {
                self.declare_method(class, method, module)?;
            }
        }
        Ok(())
    }

    /// Declare a single method as a function
    fn declare_method(
        &mut self,
        class: &ClassDef,
        method: &Method,
        module: &mut ObjectModule,
    ) -> Result<()> {
        let mangled_name = format!("{}_{}", class.name, method.name);

        // Skip if already declared
        if self.methods.contains_key(&mangled_name) {
            return Ok(());
        }

        // Skip abstract methods (no body)
        if method.is_abstract {
            return Ok(());
        }

        let ptr_type = module.target_config().pointer_type();
        let mut sig = module.make_signature();

        // Non-static methods get $this as first parameter
        if !method.is_static {
            sig.params.push(AbiParam::new(ptr_type)); // $this
        }

        // Regular parameters
        for param in &method.params {
            let ty = Self::cranelift_type(&param.ty, ptr_type);
            sig.params.push(AbiParam::new(ty));
        }

        // Return type
        if method.return_type != Type::Void {
            let ret_ty = Self::cranelift_type(&method.return_type, ptr_type);
            sig.returns.push(AbiParam::new(ret_ty));
        }

        let func_id = module
            .declare_function(&mangled_name, Linkage::Local, &sig)
            .map_err(|e| CompileError::CodegenError {
                message: format!("Failed to declare method {mangled_name}: {e}"),
            })?;

        self.methods.insert(mangled_name, func_id);
        Ok(())
    }

    /// Create vtables for all classes (zeroinit, will be filled at runtime)
    pub fn create_vtables(
        &mut self,
        registry: &ClassRegistry,
        module: &mut ObjectModule,
    ) -> Result<()> {
        let ptr_type = module.target_config().pointer_type();
        let ptr_size = ptr_type.bytes() as usize;

        for class_info in registry.all_classes() {
            let vtable_name = format!("vtable_{}", class_info.name);

            // Vtable is an array of function pointers
            let vtable_size = class_info.vtable_layout.len() * ptr_size;
            let actual_size = if vtable_size == 0 { 8 } else { vtable_size };

            // Create writable vtable data (will be filled at runtime)
            let data_id = module
                .declare_data(&vtable_name, Linkage::Local, true, false) // writable=true
                .map_err(|e| CompileError::CodegenError {
                    message: format!("Failed to declare vtable {vtable_name}: {e}"),
                })?;

            let mut desc = DataDescription::new();
            desc.define_zeroinit(actual_size);

            module
                .define_data(data_id, &desc)
                .map_err(|e| CompileError::CodegenError {
                    message: format!("Failed to define vtable {vtable_name}: {e}"),
                })?;

            self.vtables.insert(class_info.name.clone(), data_id);
        }

        Ok(())
    }

    /// Find the actual method implementation, searching up the class hierarchy
    pub fn find_method_impl(
        &self,
        class_name: &str,
        method_name: &str,
        registry: &ClassRegistry,
    ) -> String {
        // Start from the most derived class and work up
        let mut current = class_name.to_string();

        loop {
            let mangled = format!("{current}_{method_name}");
            if self.methods.contains_key(&mangled) {
                return mangled;
            }

            // Try parent class
            if let Some(class_info) = registry.get_class(&current) {
                if let Some(parent) = &class_info.parent {
                    current = parent.clone();
                    continue;
                }
            }
            break;
        }

        // Default: use the class's own method name
        format!("{class_name}_{method_name}")
    }

    /// Create static properties as global data
    pub fn create_static_properties(
        &mut self,
        classes: &[ClassDef],
        module: &mut ObjectModule,
    ) -> Result<()> {
        let ptr_type = module.target_config().pointer_type();

        for class in classes {
            for prop in &class.properties {
                if !prop.is_static {
                    continue;
                }

                let data_name = format!("{}_{}", class.name, prop.name);
                let size = Self::type_size(&prop.ty, ptr_type);

                let data_id = module
                    .declare_data(&data_name, Linkage::Local, true, false)
                    .map_err(|e| CompileError::CodegenError {
                        message: format!("Failed to declare static property {data_name}: {e}"),
                    })?;

                let mut desc = DataDescription::new();
                desc.define_zeroinit(size);

                module
                    .define_data(data_id, &desc)
                    .map_err(|e| CompileError::CodegenError {
                        message: format!("Failed to define static property {data_name}: {e}"),
                    })?;

                self.static_properties.insert(data_name, data_id);
            }
        }

        Ok(())
    }

    /// Get the vtable `DataId` for a class
    pub fn get_vtable(&self, class_name: &str) -> Option<DataId> {
        self.vtables.get(class_name).copied()
    }

    /// Get the static property `DataId`
    pub fn get_static_property(&self, class_name: &str, prop_name: &str) -> Option<DataId> {
        let key = format!("{class_name}_{prop_name}");
        self.static_properties.get(&key).copied()
    }

    /// Convert AST type to Cranelift type
    pub fn cranelift_type(ty: &Type, ptr_type: types::Type) -> types::Type {
        match ty {
            Type::Int => types::I64,
            Type::Float => types::F64,
            Type::Bool => types::I8,
            Type::String => ptr_type,
            Type::Void => types::I64, // Placeholder
            Type::Ref(_) | Type::RefMut(_) => ptr_type,
            Type::Class(_) | Type::Interface(_) => ptr_type,
            Type::Nullable(inner) => Self::cranelift_type(inner, ptr_type),
            Type::Array(_) => ptr_type,
            Type::SelfType | Type::StaticType => ptr_type,
            Type::Unknown => types::I64,
        }
    }

    /// Get size of a type in bytes
    pub fn type_size(ty: &Type, ptr_type: types::Type) -> usize {
        match ty {
            Type::Int => 8,
            Type::Float => 8,
            Type::Bool => 1,
            Type::String => ptr_type.bytes() as usize,
            Type::Class(_) | Type::Interface(_) => ptr_type.bytes() as usize,
            Type::Array(_) => ptr_type.bytes() as usize,
            Type::Ref(_) | Type::RefMut(_) => ptr_type.bytes() as usize,
            Type::Nullable(inner) => Self::type_size(inner, ptr_type),
            _ => 8,
        }
    }
}

impl Default for ClassCodeGen {
    fn default() -> Self {
        Self::new()
    }
}
