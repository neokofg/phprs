//! Dependency graph for detecting circular dependencies

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Dependency graph for detecting circular imports
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// Edges: file -> files it depends on
    edges: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl DependencyGraph {
    /// Create a new dependency graph
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, file: PathBuf) {
        self.edges.entry(file).or_default();
    }

    /// Add a dependency edge: `from` depends on `to`
    pub fn add_dependency(&mut self, from: PathBuf, to: PathBuf) {
        self.edges.entry(from).or_default().insert(to);
    }

    /// Check if adding a dependency would create a cycle
    /// Returns Some(cycle) if a cycle would be created, None otherwise
    #[must_use]
    pub fn would_create_cycle(&self, from: &PathBuf, to: &PathBuf) -> Option<Vec<PathBuf>> {
        // Check if there's a path from `to` back to `from`
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        if self.has_path_dfs(to, from, &mut visited, &mut path) {
            // Build cycle path
            path.push(to.clone());
            path.push(from.clone());
            path.reverse();
            Some(path)
        } else {
            None
        }
    }

    /// DFS to check if there's a path from src to dst
    fn has_path_dfs(
        &self,
        src: &PathBuf,
        dst: &PathBuf,
        visited: &mut HashSet<PathBuf>,
        path: &mut Vec<PathBuf>,
    ) -> bool {
        if src == dst {
            return true;
        }

        if visited.contains(src) {
            return false;
        }

        visited.insert(src.clone());
        path.push(src.clone());

        if let Some(deps) = self.edges.get(src) {
            for dep in deps {
                if self.has_path_dfs(dep, dst, visited, path) {
                    return true;
                }
            }
        }

        path.pop();
        false
    }

    /// Find all cycles in the graph
    #[must_use]
    #[allow(dead_code)]
    pub fn find_cycles(&self) -> Vec<Vec<PathBuf>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node in self.edges.keys() {
            if !visited.contains(node) {
                self.find_cycles_dfs(node, &mut visited, &mut rec_stack, &mut path, &mut cycles);
            }
        }

        cycles
    }

    fn find_cycles_dfs(
        &self,
        node: &PathBuf,
        visited: &mut HashSet<PathBuf>,
        rec_stack: &mut HashSet<PathBuf>,
        path: &mut Vec<PathBuf>,
        cycles: &mut Vec<Vec<PathBuf>>,
    ) {
        visited.insert(node.clone());
        rec_stack.insert(node.clone());
        path.push(node.clone());

        if let Some(deps) = self.edges.get(node) {
            for dep in deps {
                if !visited.contains(dep) {
                    self.find_cycles_dfs(dep, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(dep) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|p| p == dep).unwrap_or(0);
                    let mut cycle: Vec<PathBuf> = path[cycle_start..].to_vec();
                    cycle.push(dep.clone());
                    cycles.push(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    /// Get all dependencies for a file
    #[must_use]
    #[allow(dead_code)]
    pub fn dependencies(&self, file: &PathBuf) -> Option<&HashSet<PathBuf>> {
        self.edges.get(file)
    }

    /// Get topological order of files (dependencies first)
    #[must_use]
    pub fn topological_order(&self) -> Option<Vec<PathBuf>> {
        let mut in_degree: HashMap<PathBuf, usize> = HashMap::new();
        let mut reverse_edges: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

        // Initialize in-degrees
        for node in self.edges.keys() {
            in_degree.entry(node.clone()).or_insert(0);
        }

        // Calculate in-degrees and reverse edges
        for (from, deps) in &self.edges {
            for to in deps {
                *in_degree.entry(to.clone()).or_insert(0) += 1;
                reverse_edges
                    .entry(to.clone())
                    .or_default()
                    .push(from.clone());
            }
        }

        // Kahn's algorithm
        let mut queue: Vec<PathBuf> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(node, _)| node.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(node) = queue.pop() {
            result.push(node.clone());

            if let Some(deps) = self.edges.get(&node) {
                for dep in deps {
                    if let Some(deg) = in_degree.get_mut(dep) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(dep.clone());
                        }
                    }
                }
            }
        }

        if result.len() == self.edges.len() {
            // Reverse to get dependencies first
            result.reverse();
            Some(result)
        } else {
            // Cycle detected
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(PathBuf::from("a.php"), PathBuf::from("b.php"));
        graph.add_dependency(PathBuf::from("b.php"), PathBuf::from("c.php"));

        assert!(graph
            .would_create_cycle(&PathBuf::from("a.php"), &PathBuf::from("b.php"))
            .is_none());
    }

    #[test]
    fn test_would_create_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(PathBuf::from("a.php"), PathBuf::from("b.php"));
        graph.add_dependency(PathBuf::from("b.php"), PathBuf::from("c.php"));

        // Adding c -> a would create a cycle
        let cycle = graph.would_create_cycle(&PathBuf::from("c.php"), &PathBuf::from("a.php"));
        assert!(cycle.is_some());
    }

    #[test]
    fn test_topological_order() {
        let mut graph = DependencyGraph::new();
        graph.add_node(PathBuf::from("a.php"));
        graph.add_dependency(PathBuf::from("a.php"), PathBuf::from("b.php"));
        graph.add_dependency(PathBuf::from("b.php"), PathBuf::from("c.php"));
        graph.add_node(PathBuf::from("c.php"));

        let order = graph.topological_order();
        assert!(order.is_some());
    }
}
