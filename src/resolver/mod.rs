//! Module resolver for namespace and use declarations
//!
//! Resolves imports, loads dependencies, and detects circular dependencies.

mod dependency_graph;
mod file_resolver;
mod symbol_table;

pub use dependency_graph::DependencyGraph;
pub use file_resolver::FileResolver;
pub use symbol_table::{Symbol, SymbolTable};

use std::collections::HashMap;
use std::path::PathBuf;

use crate::ast::{CompilationUnit, Program, QualifiedName, Span, UseKind};
use crate::errors::CompileError;
use crate::lexer;
use crate::parser;
use miette::Result;

/// Module resolver that handles namespace resolution and file loading
#[derive(Debug)]
pub struct ModuleResolver {
    /// File resolver for finding PHP files
    file_resolver: FileResolver,
    /// Dependency graph for cycle detection
    dependency_graph: DependencyGraph,
    /// Loaded compilation units by file path
    loaded_units: HashMap<PathBuf, CompilationUnit>,
    /// Symbol tables by file path
    symbol_tables: HashMap<PathBuf, SymbolTable>,
    /// Global class registry: qualified name -> file path
    class_locations: HashMap<String, PathBuf>,
    /// Global function registry: qualified name -> file path
    function_locations: HashMap<String, PathBuf>,
    /// Global trait registry: qualified name -> file path
    trait_locations: HashMap<String, PathBuf>,
}

impl ModuleResolver {
    /// Create a new module resolver with given root directories
    #[must_use]
    pub fn new(roots: Vec<PathBuf>) -> Self {
        Self {
            file_resolver: FileResolver::new(roots),
            dependency_graph: DependencyGraph::new(),
            loaded_units: HashMap::new(),
            symbol_tables: HashMap::new(),
            class_locations: HashMap::new(),
            function_locations: HashMap::new(),
            trait_locations: HashMap::new(),
        }
    }

    /// Add a root directory for file resolution
    pub fn add_root(&mut self, root: PathBuf) {
        self.file_resolver.add_root(root);
    }

    /// Resolve a compilation unit and all its dependencies
    pub fn resolve(&mut self, entry_path: PathBuf, unit: CompilationUnit) -> Result<Program> {
        // Load the entry unit
        self.load_unit(entry_path.clone(), unit)?;

        // Process all pending imports until no more dependencies
        self.resolve_all_imports()?;

        // Build the final program
        self.build_program()
    }

    /// Load a compilation unit and register its symbols
    fn load_unit(&mut self, path: PathBuf, mut unit: CompilationUnit) -> Result<()> {
        // Skip if already loaded
        if self.loaded_units.contains_key(&path) {
            return Ok(());
        }

        // Add to dependency graph
        self.dependency_graph.add_node(path.clone());

        // Build symbol table for this unit
        let mut symbol_table = SymbolTable::new();

        // Set namespace
        if let Some(ns) = &unit.namespace {
            symbol_table.set_namespace(ns.name.clone());
        }

        // Register imports
        for use_decl in &unit.uses {
            for item in &use_decl.items {
                let alias = item.imported_name().to_string();
                symbol_table.add_import(alias, item.path.clone(), item.kind);
            }
        }

        // Register and qualify classes
        for class in &mut unit.classes {
            let qualified = self.qualify_name(&symbol_table, &class.name);
            class.qualified_name = Some(qualified.clone());
            symbol_table.define_class(&class.name, qualified.clone());
            self.class_locations
                .insert(qualified.full_path(), path.clone());

            // Qualify parent class with current namespace if it's a simple name
            if let Some(parent_qn) = &class.parent_qualified {
                if parent_qn.segments.len() == 1 && !parent_qn.is_absolute {
                    // Simple name - qualify with current namespace
                    let resolved = symbol_table.resolve_qualified(parent_qn, UseKind::Class);
                    class.parent_qualified = Some(resolved.clone());
                    class.parent = Some(resolved.full_path());
                }
            }

            // Qualify interfaces with current namespace if they're simple names
            let mut resolved_interfaces = Vec::new();
            for iface_qn in &class.interfaces_qualified {
                if iface_qn.segments.len() == 1 && !iface_qn.is_absolute {
                    let resolved = symbol_table.resolve_qualified(iface_qn, UseKind::Class);
                    resolved_interfaces.push(resolved);
                } else {
                    resolved_interfaces.push(iface_qn.clone());
                }
            }
            if !resolved_interfaces.is_empty() {
                class.interfaces_qualified = resolved_interfaces.clone();
                class.interfaces = resolved_interfaces.iter().map(|qn| qn.full_path()).collect();
            }
        }

        // Register and qualify functions
        for func in &unit.functions {
            let qualified = self.qualify_name(&symbol_table, &func.name);
            symbol_table.define_function(&func.name, qualified.clone());
            self.function_locations
                .insert(qualified.full_path(), path.clone());
        }

        // Register and qualify traits
        for trait_def in &mut unit.traits {
            let qualified = self.qualify_name(&symbol_table, &trait_def.name);
            trait_def.qualified_name = Some(qualified.clone());
            symbol_table.define_trait(&trait_def.name, qualified.clone());
            self.trait_locations
                .insert(qualified.full_path(), path.clone());
        }

        // Store
        unit.file_path = Some(path.clone());
        self.symbol_tables.insert(path.clone(), symbol_table);
        self.loaded_units.insert(path, unit);

        Ok(())
    }

    /// Qualify a simple name with the current namespace
    fn qualify_name(&self, symbol_table: &SymbolTable, name: &str) -> QualifiedName {
        if let Some(ns) = &symbol_table.namespace {
            let mut segments = ns.segments.clone();
            segments.push(name.to_string());
            QualifiedName::new(segments, false, Span::default())
        } else {
            QualifiedName::simple(name.to_string(), Span::default())
        }
    }

    /// Resolve all pending imports (loops until no more dependencies)
    fn resolve_all_imports(&mut self) -> Result<()> {
        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 100;

        loop {
            iteration += 1;
            if iteration > MAX_ITERATIONS {
                return Err(CompileError::ResolverError {
                    message: "Too many resolution iterations - possible circular dependency"
                        .to_string(),
                }
                .into());
            }

            // Collect all imports that need resolution
            let mut pending: Vec<(PathBuf, QualifiedName)> = Vec::new();
            let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

            // Check use imports
            for (path, symbol_table) in &self.symbol_tables {
                for (_, symbol) in symbol_table.imports() {
                    let qn = symbol.qualified_name();
                    let key = qn.full_path();
                    if !self.is_loaded(qn) && !seen.contains(&key) {
                        seen.insert(key);
                        pending.push((path.clone(), qn.clone()));
                    }
                }
            }

            // Check parent classes
            for (path, unit) in &self.loaded_units {
                for class in &unit.classes {
                    if let Some(parent_qn) = &class.parent_qualified {
                        let key = parent_qn.full_path();
                        if !self.is_loaded(parent_qn) && !seen.contains(&key) {
                            seen.insert(key);
                            pending.push((path.clone(), parent_qn.clone()));
                        }
                    }
                    // Check interfaces too
                    for iface_qn in &class.interfaces_qualified {
                        let key = iface_qn.full_path();
                        if !self.is_loaded(iface_qn) && !seen.contains(&key) {
                            seen.insert(key);
                            pending.push((path.clone(), iface_qn.clone()));
                        }
                    }
                }
            }

            if pending.is_empty() {
                break;
            }

            // Track if we actually loaded anything new
            let loaded_before = self.loaded_units.len();

            // Try to resolve each pending import
            for (from_path, qn) in &pending {
                self.resolve_import(from_path, qn)?;
            }

            // If we didn't load anything new, we can't resolve these imports
            if self.loaded_units.len() == loaded_before {
                // These imports couldn't be resolved - let type checker handle it
                break;
            }
        }

        Ok(())
    }

    /// Check if a qualified name is already loaded
    fn is_loaded(&self, qn: &QualifiedName) -> bool {
        let full_path = qn.full_path();
        self.class_locations.contains_key(&full_path)
            || self.function_locations.contains_key(&full_path)
            || self.trait_locations.contains_key(&full_path)
    }

    /// Resolve a single import
    fn resolve_import(&mut self, from_path: &PathBuf, qn: &QualifiedName) -> Result<()> {
        // Try to find the file for this import
        if let Some(file_path) = self.file_resolver.resolve(qn) {
            // Check for circular dependency
            if let Some(cycle) = self
                .dependency_graph
                .would_create_cycle(from_path, &file_path)
            {
                return Err(CompileError::ResolverError {
                    message: format!(
                        "Circular dependency detected: {}",
                        cycle
                            .iter()
                            .map(|p| p.display().to_string())
                            .collect::<Vec<_>>()
                            .join(" -> ")
                    ),
                }
                .into());
            }

            // Add dependency edge
            self.dependency_graph
                .add_dependency(from_path.clone(), file_path.clone());

            // Load the file if not already loaded
            if !self.loaded_units.contains_key(&file_path) {
                let unit = self.load_file(&file_path)?;
                self.load_unit(file_path, unit)?;
            }
        }
        // If file not found, we'll let the type checker handle the error

        Ok(())
    }

    /// Load and parse a PHP file
    fn load_file(&self, path: &PathBuf) -> Result<CompilationUnit> {
        let source = std::fs::read_to_string(path).map_err(|e| CompileError::ResolverError {
            message: format!("Failed to read file {}: {}", path.display(), e),
        })?;

        let tokens = lexer::tokenize(&source)?;
        parser::parse_unit(tokens)
    }

    /// Build the final program from all loaded units
    fn build_program(&mut self) -> Result<Program> {
        // Get topological order (dependencies first)
        let order = self.dependency_graph.topological_order().unwrap_or_else(|| {
            // If there's a cycle, just use arbitrary order
            self.loaded_units.keys().cloned().collect()
        });

        let mut all_functions = Vec::new();
        let mut all_classes = Vec::new();
        let mut all_traits = Vec::new();
        let mut all_units = Vec::new();

        for path in order {
            if let Some(unit) = self.loaded_units.remove(&path) {
                // Resolve qualified names in classes
                let resolved_classes = self.resolve_class_references(&path, unit.classes)?;
                let resolved_functions = unit.functions;
                let resolved_traits = unit.traits;

                // Keep copies for type checker context resolution
                all_units.push(CompilationUnit {
                    namespace: unit.namespace,
                    uses: unit.uses,
                    functions: resolved_functions.clone(),
                    classes: resolved_classes.clone(),
                    traits: resolved_traits.clone(),
                    file_path: unit.file_path,
                });

                all_classes.extend(resolved_classes);
                all_functions.extend(resolved_functions);
                all_traits.extend(resolved_traits);
            }
        }

        Ok(Program {
            units: all_units,
            functions: all_functions,
            classes: all_classes,
            traits: all_traits,
        })
    }

    /// Resolve class references (parent, interfaces) to qualified names
    fn resolve_class_references(
        &self,
        path: &PathBuf,
        classes: Vec<crate::ast::ClassDef>,
    ) -> Result<Vec<crate::ast::ClassDef>> {
        let symbol_table = self.symbol_tables.get(path);

        classes
            .into_iter()
            .map(|mut class| {
                // Resolve parent class
                if let Some(parent_qn) = &class.parent_qualified {
                    if let Some(st) = symbol_table {
                        let resolved = st.resolve_qualified(parent_qn, UseKind::Class);
                        class.parent = Some(resolved.full_path());
                        class.parent_qualified = Some(resolved);
                    }
                }

                // Resolve interfaces
                let mut resolved_interfaces = Vec::new();
                for iface_qn in &class.interfaces_qualified {
                    if let Some(st) = symbol_table {
                        let resolved = st.resolve_qualified(iface_qn, UseKind::Class);
                        resolved_interfaces.push(resolved);
                    }
                }
                if !resolved_interfaces.is_empty() {
                    class.interfaces = resolved_interfaces
                        .iter()
                        .map(|qn| qn.full_path())
                        .collect();
                    class.interfaces_qualified = resolved_interfaces;
                }

                Ok(class)
            })
            .collect()
    }

    /// Get all loaded units
    pub fn units(&self) -> impl Iterator<Item = (&PathBuf, &CompilationUnit)> {
        self.loaded_units.iter()
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
