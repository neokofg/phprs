//! Class registry for OOP support
//!
//! Tracks class definitions, inheritance hierarchies, vtable layouts,
//! and property offsets for code generation.

#![allow(
    clippy::items_after_statements,
    clippy::if_not_else,
    clippy::option_if_let_else,
    clippy::self_only_used_in_recursion
)]

use std::collections::HashMap;

use crate::ast::{ClassDef, QualifiedName, TraitDef, Type, Visibility};

/// Registry of all classes in the program
#[derive(Debug, Default)]
pub struct ClassRegistry {
    classes: HashMap<String, ClassInfo>,
    traits: HashMap<String, TraitInfo>,
}

/// Complete information about a trait
#[derive(Debug, Clone)]
pub struct TraitInfo {
    /// Simple trait name
    pub name: String,
    /// Fully qualified name
    pub qualified_name: Option<QualifiedName>,
    /// Trait properties
    pub properties: Vec<PropertyInfo>,
    /// Trait methods
    pub methods: Vec<TraitMethodInfo>,
}

/// Information about a trait method
#[derive(Debug, Clone)]
pub struct TraitMethodInfo {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_abstract: bool,
}

/// Complete information about a class
#[derive(Debug, Clone)]
pub struct ClassInfo {
    /// Simple class name
    pub name: String,
    /// Fully qualified name
    pub qualified_name: Option<QualifiedName>,
    pub parent: Option<String>,
    pub properties: Vec<PropertyInfo>,
    pub methods: Vec<MethodInfo>,
    /// Method names in vtable order (for virtual dispatch)
    pub vtable_layout: Vec<String>,
    /// Total size of object in bytes (including vtable pointer)
    pub object_size: usize,
    pub is_abstract: bool,
    pub is_final: bool,
}

/// Information about a class property
#[derive(Debug, Clone)]
pub struct PropertyInfo {
    pub name: String,
    pub ty: Type,
    pub visibility: Visibility,
    pub is_static: bool,
    /// Offset in bytes from object start (after vtable pointer)
    pub offset: usize,
}

/// Information about a class method
#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub visibility: Visibility,
    pub is_static: bool,
    #[allow(dead_code)]
    pub is_abstract: bool,
    #[allow(dead_code)]
    pub is_final: bool,
    /// Index in vtable (None for static methods)
    pub vtable_index: Option<usize>,
    /// Mangled function name for codegen
    pub mangled_name: String,
}

impl ClassRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            traits: HashMap::new(),
        }
    }

    /// Register all traits from the program
    pub fn register_traits(&mut self, traits: &[TraitDef]) {
        for trait_def in traits {
            let key = trait_def
                .qualified_name
                .as_ref()
                .map_or_else(|| trait_def.name.clone(), QualifiedName::full_path);

            let properties: Vec<PropertyInfo> = trait_def
                .properties
                .iter()
                .map(|prop| PropertyInfo {
                    name: prop.name.clone(),
                    ty: prop.ty.clone(),
                    visibility: prop.visibility,
                    is_static: prop.is_static,
                    offset: 0, // Will be set when applied to class
                })
                .collect();

            let methods: Vec<TraitMethodInfo> = trait_def
                .methods
                .iter()
                .map(|method| TraitMethodInfo {
                    name: method.name.clone(),
                    params: method
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone()))
                        .collect(),
                    return_type: method.return_type.clone(),
                    visibility: method.visibility,
                    is_static: method.is_static,
                    is_abstract: method.is_abstract,
                })
                .collect();

            let info = TraitInfo {
                name: trait_def.name.clone(),
                qualified_name: trait_def.qualified_name.clone(),
                properties,
                methods,
            };

            self.traits.insert(key, info);
        }
    }

    /// Register all classes from the program
    pub fn register_classes(&mut self, classes: &[ClassDef]) {
        // First pass: register all class names
        for class in classes {
            // Use qualified name as key if available
            let key = class
                .qualified_name
                .as_ref()
                .map_or_else(|| class.name.clone(), |qn| qn.full_path());

            let info = ClassInfo {
                name: class.name.clone(),
                qualified_name: class.qualified_name.clone(),
                parent: class.parent.clone(),
                properties: Vec::new(),
                methods: Vec::new(),
                vtable_layout: Vec::new(),
                object_size: 8, // Start with vtable pointer size
                is_abstract: class.is_abstract,
                is_final: class.is_final,
            };
            self.classes.insert(key, info);
        }

        // Second pass: build vtables and compute layouts
        // Process in order that respects inheritance
        let ordered = self.topological_sort(classes);
        for class_key in ordered {
            // Find class by qualified name or simple name
            if let Some(class) = classes.iter().find(|c| {
                c.qualified_name
                    .as_ref()
                    .map_or(c.name == class_key, |qn| qn.full_path() == class_key)
            }) {
                self.build_class_info(class);
            }
        }
    }

    /// Get the registry key for a class
    fn get_class_key(class: &ClassDef) -> String {
        class
            .qualified_name
            .as_ref()
            .map_or_else(|| class.name.clone(), |qn| qn.full_path())
    }

    /// Topological sort of classes by inheritance
    fn topological_sort(&self, classes: &[ClassDef]) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();

        fn visit(
            key: &str,
            classes: &[ClassDef],
            visited: &mut std::collections::HashSet<String>,
            result: &mut Vec<String>,
        ) {
            if visited.contains(key) {
                return;
            }
            visited.insert(key.to_string());

            // Find class by key (qualified name or simple name)
            if let Some(class) = classes.iter().find(|c| {
                c.qualified_name
                    .as_ref()
                    .map_or(c.name == key, |qn| qn.full_path() == key)
            }) {
                if let Some(parent) = &class.parent {
                    visit(parent, classes, visited, result);
                }
            }
            result.push(key.to_string());
        }

        for class in classes {
            let key = Self::get_class_key(class);
            visit(&key, classes, &mut visited, &mut result);
        }

        result
    }

    /// Build complete class info including inherited members
    fn build_class_info(&mut self, class: &ClassDef) {
        let class_key = Self::get_class_key(class);
        let mangled_class_name = class
            .qualified_name
            .as_ref()
            .map_or_else(|| class.name.clone(), |qn| qn.mangle());

        let mut properties = Vec::new();
        let mut methods = Vec::new();
        let mut vtable_layout = Vec::new();
        let mut offset = 8; // After vtable pointer

        // Inherit from parent
        if let Some(parent_name) = &class.parent {
            // Try to find parent by qualified name first, then by simple name
            let parent_info = self.classes.get(parent_name).cloned();
            if let Some(parent_info) = parent_info {
                // Inherit properties
                for prop in &parent_info.properties {
                    if !prop.is_static {
                        properties.push(prop.clone());
                    }
                }
                offset = parent_info.object_size;

                // Inherit vtable layout
                vtable_layout.clone_from(&parent_info.vtable_layout);

                // Inherit methods (can be overridden)
                for method in &parent_info.methods {
                    methods.push(method.clone());
                }
            }
        }

        // Apply traits
        for trait_use in &class.trait_uses {
            for trait_qn in &trait_use.traits {
                let trait_key = trait_qn.full_path();
                if let Some(trait_info) = self.traits.get(&trait_key).cloned() {
                    // Add trait properties
                    for trait_prop in &trait_info.properties {
                        // Don't add if already exists (class can override)
                        if !properties.iter().any(|p| p.name == trait_prop.name) {
                            let size = self.type_size(&trait_prop.ty);
                            let prop_info = PropertyInfo {
                                name: trait_prop.name.clone(),
                                ty: trait_prop.ty.clone(),
                                visibility: trait_prop.visibility,
                                is_static: trait_prop.is_static,
                                offset: if trait_prop.is_static { 0 } else { offset },
                            };
                            if !trait_prop.is_static {
                                offset += size;
                            }
                            properties.push(prop_info);
                        }
                    }

                    // Add trait methods
                    for trait_method in &trait_info.methods {
                        // Don't add if already exists (class can override)
                        if !methods.iter().any(|m| m.name == trait_method.name) {
                            let mangled_name =
                                format!("{}_{}", mangled_class_name, trait_method.name);

                            if trait_method.is_static {
                                let method_info = MethodInfo {
                                    name: trait_method.name.clone(),
                                    params: trait_method.params.clone(),
                                    return_type: trait_method.return_type.clone(),
                                    visibility: trait_method.visibility,
                                    is_static: true,
                                    is_abstract: trait_method.is_abstract,
                                    is_final: false,
                                    vtable_index: None,
                                    mangled_name,
                                };
                                methods.push(method_info);
                            } else {
                                let vtable_idx = vtable_layout.len();
                                vtable_layout.push(trait_method.name.clone());

                                let method_info = MethodInfo {
                                    name: trait_method.name.clone(),
                                    params: trait_method.params.clone(),
                                    return_type: trait_method.return_type.clone(),
                                    visibility: trait_method.visibility,
                                    is_static: false,
                                    is_abstract: trait_method.is_abstract,
                                    is_final: false,
                                    vtable_index: Some(vtable_idx),
                                    mangled_name,
                                };
                                methods.push(method_info);
                            }
                        }
                    }
                }
            }
        }

        // Add own properties
        for prop in &class.properties {
            if prop.is_static {
                // Static properties don't take object space
                let prop_info = PropertyInfo {
                    name: prop.name.clone(),
                    ty: prop.ty.clone(),
                    visibility: prop.visibility,
                    is_static: true,
                    offset: 0,
                };
                properties.push(prop_info);
            } else {
                let size = self.type_size(&prop.ty);
                let prop_info = PropertyInfo {
                    name: prop.name.clone(),
                    ty: prop.ty.clone(),
                    visibility: prop.visibility,
                    is_static: prop.is_static,
                    offset,
                };
                properties.push(prop_info);
                offset += size;
            }
        }

        // Add own methods
        for method in &class.methods {
            let mangled_name = format!("{}_{}", mangled_class_name, method.name);

            if method.is_static {
                // Static methods don't go in vtable
                let method_info = MethodInfo {
                    name: method.name.clone(),
                    params: method
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone()))
                        .collect(),
                    return_type: method.return_type.clone(),
                    visibility: method.visibility,
                    is_static: true,
                    is_abstract: method.is_abstract,
                    is_final: method.is_final,
                    vtable_index: None,
                    mangled_name,
                };
                methods.push(method_info);
            } else {
                // Check if this overrides a parent method
                let existing_idx = vtable_layout.iter().position(|n| n == &method.name);

                let vtable_idx = if let Some(idx) = existing_idx {
                    // Override existing method
                    idx
                } else {
                    // New method, add to vtable
                    let idx = vtable_layout.len();
                    vtable_layout.push(method.name.clone());
                    idx
                };

                let method_info = MethodInfo {
                    name: method.name.clone(),
                    params: method
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone()))
                        .collect(),
                    return_type: method.return_type.clone(),
                    visibility: method.visibility,
                    is_static: false,
                    is_abstract: method.is_abstract,
                    is_final: method.is_final,
                    vtable_index: Some(vtable_idx),
                    mangled_name,
                };

                // Update or add method
                if let Some(existing) = methods.iter_mut().find(|m| m.name == method.name) {
                    *existing = method_info;
                } else {
                    methods.push(method_info);
                }
            }
        }

        // Align object size to 8 bytes
        let object_size = (offset + 7) & !7;

        // Update class info
        if let Some(info) = self.classes.get_mut(&class_key) {
            info.properties = properties;
            info.methods = methods;
            info.vtable_layout = vtable_layout;
            info.object_size = object_size;
        }
    }

    /// Get size of a type in bytes
    fn type_size(&self, ty: &Type) -> usize {
        match ty {
            Type::Int => 8,
            Type::Float => 8,
            Type::Bool => 1,
            Type::String => 8, // Pointer
            Type::Class(_) | Type::Interface(_) => 8, // Pointer
            Type::Array(_) => 8, // Pointer
            Type::Ref(_) | Type::RefMut(_) => 8, // Pointer
            Type::Nullable(inner) => self.type_size(inner),
            _ => 8,
        }
    }

    /// Get class info by name (tries qualified name first, then simple name)
    #[must_use]
    pub fn get_class(&self, name: &str) -> Option<&ClassInfo> {
        // First try exact match
        if let Some(info) = self.classes.get(name) {
            return Some(info);
        }
        // Then try to find by simple name
        self.classes.values().find(|info| info.name == name)
    }

    /// Get property info from a class (including inherited)
    #[must_use]
    pub fn get_property(&self, class_name: &str, prop_name: &str) -> Option<&PropertyInfo> {
        self.get_class(class_name)
            .and_then(|c| c.properties.iter().find(|p| p.name == prop_name))
    }

    /// Get method info from a class (including inherited)
    #[must_use]
    pub fn get_method(&self, class_name: &str, method_name: &str) -> Option<&MethodInfo> {
        self.get_class(class_name)
            .and_then(|c| c.methods.iter().find(|m| m.name == method_name))
    }

    /// Check if a class exists
    #[must_use]
    pub fn class_exists(&self, name: &str) -> bool {
        self.get_class(name).is_some()
    }

    /// Check if child class is a subtype of parent class
    #[must_use] 
    pub fn is_subclass(&self, child: &str, parent: &str) -> bool {
        if child == parent {
            return true;
        }

        if let Some(child_info) = self.classes.get(child) {
            if let Some(parent_name) = &child_info.parent {
                return self.is_subclass(parent_name, parent);
            }
        }

        false
    }

    /// Check if a member is accessible from a given context
    #[must_use] 
    pub fn is_accessible(
        &self,
        target_class: &str,
        member_visibility: Visibility,
        from_class: Option<&str>,
    ) -> bool {
        match member_visibility {
            Visibility::Public => true,
            Visibility::Protected => {
                if let Some(from) = from_class {
                    self.is_subclass(from, target_class) || self.is_subclass(target_class, from)
                } else {
                    false
                }
            }
            Visibility::Private => from_class == Some(target_class),
        }
    }

    /// Get all classes
    pub fn all_classes(&self) -> impl Iterator<Item = &ClassInfo> {
        self.classes.values()
    }

    /// Get trait info by name
    #[must_use]
    pub fn get_trait(&self, name: &str) -> Option<&TraitInfo> {
        // First try exact match
        if let Some(info) = self.traits.get(name) {
            return Some(info);
        }
        // Then try to find by simple name
        self.traits.values().find(|info| info.name == name)
    }

    /// Check if a trait exists
    #[must_use]
    pub fn trait_exists(&self, name: &str) -> bool {
        self.get_trait(name).is_some()
    }

    /// Get constructor info for a class
    #[must_use] 
    pub fn get_constructor(&self, class_name: &str) -> Option<&MethodInfo> {
        self.get_method(class_name, "__construct")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Span};

    fn make_class(name: &str, parent: Option<&str>) -> ClassDef {
        ClassDef {
            name: name.to_string(),
            qualified_name: None,
            parent: parent.map(|s| s.to_string()),
            parent_qualified: None,
            interfaces: vec![],
            interfaces_qualified: vec![],
            properties: vec![],
            methods: vec![],
            trait_uses: vec![],
            is_abstract: false,
            is_final: false,
            span: Span::default(),
        }
    }

    #[test]
    fn test_class_registry() {
        let mut registry = ClassRegistry::new();
        let classes = vec![
            make_class("Animal", None),
            make_class("Dog", Some("Animal")),
        ];
        registry.register_classes(&classes);

        assert!(registry.class_exists("Animal"));
        assert!(registry.class_exists("Dog"));
        assert!(registry.is_subclass("Dog", "Animal"));
        assert!(!registry.is_subclass("Animal", "Dog"));
    }
}
