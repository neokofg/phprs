//! File resolver for finding PHP files by namespace

use std::path::{Path, PathBuf};

use crate::ast::QualifiedName;

/// Resolves namespace paths to file paths (PSR-4 style)
#[derive(Debug, Default)]
pub struct FileResolver {
    /// Root directories to search for files
    roots: Vec<PathBuf>,
}

impl FileResolver {
    /// Create a new file resolver with given root directories
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(roots: Vec<PathBuf>) -> Self {
        Self { roots }
    }

    /// Add a root directory
    #[allow(dead_code)]
    pub fn add_root(&mut self, root: PathBuf) {
        if !self.roots.contains(&root) {
            self.roots.push(root);
        }
    }

    /// Resolve a qualified name to a file path
    /// Returns the first matching file found
    #[must_use]
    pub fn resolve(&self, name: &QualifiedName) -> Option<PathBuf> {
        for root in &self.roots {
            if let Some(path) = Self::try_resolve_in_root(root, name) {
                return Some(path);
            }
        }
        None
    }

    /// Try to resolve a name in a specific root directory
    fn try_resolve_in_root(root: &Path, name: &QualifiedName) -> Option<PathBuf> {
        // Convert namespace segments to path: App\Models\User -> App/Models/User.php
        let relative_path: PathBuf = name.segments.iter().collect();
        let with_ext = relative_path.with_extension("php");

        // Try exact case
        let path = root.join(&with_ext);
        if path.exists() {
            return Some(path);
        }

        // Try lowercase first segment (common convention)
        // App\Models\User -> app/Models/User.php
        if name.segments.len() > 1 {
            let mut segments = name.segments.clone();
            segments[0] = segments[0].to_lowercase();
            let relative_path: PathBuf = segments.iter().collect();
            let with_ext = relative_path.with_extension("php");
            let path = root.join(&with_ext);
            if path.exists() {
                return Some(path);
            }
        }

        // Try all lowercase
        // App\Models\User -> app/models/user.php
        let lowercase_segments: Vec<String> =
            name.segments.iter().map(|s| s.to_lowercase()).collect();
        let relative_path: PathBuf = lowercase_segments.iter().collect();
        let with_ext = relative_path.with_extension("php");
        let path = root.join(&with_ext);
        if path.exists() {
            return Some(path);
        }

        None
    }

    /// Get all root directories
    #[must_use]
    #[allow(dead_code)]
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Span;

    #[test]
    fn test_file_resolver_path_conversion() {
        let resolver = FileResolver::new(vec![]);
        let name = QualifiedName::new(
            vec!["App".to_string(), "Models".to_string(), "User".to_string()],
            false,
            Span::default(),
        );
        // Just test that it doesn't panic
        let _ = resolver.resolve(&name);
    }
}
