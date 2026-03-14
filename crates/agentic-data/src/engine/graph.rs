//! Graph engine — relationship traversal, impact analysis, and discovery.
//!
//! Parity with AgenticMemory's graph traversal, BFS, and impact analysis.

use std::collections::{HashMap, HashSet, VecDeque};
use crate::types::*;
use super::store::DataStore;

/// Direction for graph traversal.
#[derive(Debug, Clone, Copy)]
pub enum TraversalDirection { Forward, Backward, Both }

/// Result of a graph traversal.
#[derive(Debug, Clone)]
pub struct TraversalResult {
    pub visited: Vec<String>,
    pub edges_followed: usize,
    pub depth_reached: usize,
}

/// Result of impact analysis.
#[derive(Debug, Clone)]
pub struct ImpactResult {
    pub affected: Vec<String>,
    pub depth: usize,
    pub total_downstream: usize,
}

/// Result of relationship discovery.
#[derive(Debug, Clone)]
pub struct RelationshipCandidate {
    pub from_node: String,
    pub from_field: String,
    pub to_node: String,
    pub to_field: String,
    pub confidence: f64,
    pub reason: String,
}

/// Graph operations on schema relationships and data lineage.
pub struct GraphEngine;

impl GraphEngine {
    /// BFS traversal from a starting node through schema edges.
    pub fn traverse_schema(
        schema: &UniversalSchema,
        start_node: &str,
        direction: TraversalDirection,
        max_depth: usize,
    ) -> TraversalResult {
        let mut visited = Vec::new();
        let mut seen = HashSet::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        let mut edges_followed = 0;
        let mut max_d = 0;

        queue.push_back((start_node.to_string(), 0));
        seen.insert(start_node.to_string());

        while let Some((node, depth)) = queue.pop_front() {
            if depth > max_depth { continue; }
            visited.push(node.clone());
            max_d = max_d.max(depth);

            for edge in &schema.edges {
                let next = match direction {
                    TraversalDirection::Forward if edge.from.starts_with(&node) => Some(&edge.to),
                    TraversalDirection::Backward if edge.to.starts_with(&node) => Some(&edge.from),
                    TraversalDirection::Both => {
                        if edge.from.starts_with(&node) { Some(&edge.to) }
                        else if edge.to.starts_with(&node) { Some(&edge.from) }
                        else { None }
                    }
                    _ => None,
                };
                if let Some(target) = next {
                    let target_node = target.split('.').next().unwrap_or(target).to_string();
                    if !seen.contains(&target_node) {
                        seen.insert(target_node.clone());
                        queue.push_back((target_node, depth + 1));
                        edges_followed += 1;
                    }
                }
            }
        }

        TraversalResult { visited, edges_followed, depth_reached: max_d }
    }

    /// Impact analysis: if a field changes, what else is affected?
    pub fn impact_analysis(
        store: &DataStore,
        source_field: &str,
    ) -> ImpactResult {
        let mut affected = Vec::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        let mut seen = HashSet::new();
        let mut max_depth = 0;

        queue.push_back((source_field.to_string(), 0));
        seen.insert(source_field.to_string());

        // Traverse lineage forward deps
        while let Some((field, depth)) = queue.pop_front() {
            if depth > 10 { continue; }
            max_depth = max_depth.max(depth);

            if let Some(chain) = store.get_lineage(&field) {
                for entry in &chain.entries {
                    if !seen.contains(&entry.source) {
                        seen.insert(entry.source.clone());
                        affected.push(entry.source.clone());
                        queue.push_back((entry.source.clone(), depth + 1));
                    }
                }
            }
        }

        ImpactResult { total_downstream: affected.len(), depth: max_depth, affected }
    }

    /// Discover potential relationships between schema nodes by analyzing field names and data.
    pub fn discover_relationships(
        store: &DataStore,
    ) -> Vec<RelationshipCandidate> {
        let mut candidates = Vec::new();
        let schemas = store.all_schemas();

        for schema in &schemas {
            let nodes = &schema.nodes;
            for i in 0..nodes.len() {
                for j in (i + 1)..nodes.len() {
                    // Check for FK-like name patterns
                    for fi in &nodes[i].fields {
                        for fj in &nodes[j].fields {
                            if let Some(candidate) = check_relationship(
                                &nodes[i].name, &fi.name, &fi.field_type,
                                &nodes[j].name, &fj.name, &fj.field_type,
                            ) {
                                candidates.push(candidate);
                            }
                        }
                    }
                }
            }
        }
        candidates
    }

    /// Find connected components in schema graph.
    pub fn connected_components(schema: &UniversalSchema) -> Vec<Vec<String>> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();

        for node in &schema.nodes {
            if visited.contains(&node.name) { continue; }
            let result = Self::traverse_schema(schema, &node.name, TraversalDirection::Both, 100);
            for n in &result.visited { visited.insert(n.clone()); }
            components.push(result.visited);
        }
        components
    }

    /// Compute node importance (degree centrality).
    pub fn node_centrality(schema: &UniversalSchema) -> HashMap<String, f64> {
        let mut degrees: HashMap<String, f64> = HashMap::new();
        let total_edges = schema.edges.len().max(1) as f64;

        for node in &schema.nodes {
            degrees.insert(node.name.clone(), 0.0);
        }
        for edge in &schema.edges {
            let from_node = edge.from.split('.').next().unwrap_or(&edge.from);
            let to_node = edge.to.split('.').next().unwrap_or(&edge.to);
            *degrees.entry(from_node.to_string()).or_default() += 1.0 / total_edges;
            *degrees.entry(to_node.to_string()).or_default() += 1.0 / total_edges;
        }
        degrees
    }
}

fn check_relationship(
    node_a: &str, field_a: &str, type_a: &FieldType,
    node_b: &str, field_b: &str, type_b: &FieldType,
) -> Option<RelationshipCandidate> {
    // Pattern: "orders.user_id" matches "users.id"
    let a_lower = field_a.to_lowercase();
    let b_lower = field_b.to_lowercase();
    let node_b_singular = node_b.trim_end_matches('s').to_lowercase();

    if a_lower == format!("{}_id", node_b_singular) && b_lower == "id" && type_a == type_b {
        return Some(RelationshipCandidate {
            from_node: node_a.into(), from_field: field_a.into(),
            to_node: node_b.into(), to_field: field_b.into(),
            confidence: 0.90,
            reason: format!("{}.{} looks like FK to {}.{}", node_a, field_a, node_b, field_b),
        });
    }
    if b_lower == format!("{}_id", node_a.trim_end_matches('s').to_lowercase()) && a_lower == "id" && type_a == type_b {
        return Some(RelationshipCandidate {
            from_node: node_b.into(), from_field: field_b.into(),
            to_node: node_a.into(), to_field: field_a.into(),
            confidence: 0.90,
            reason: format!("{}.{} looks like FK to {}.{}", node_b, field_b, node_a, field_a),
        });
    }
    // Same field name + same type across tables
    if field_a == field_b && type_a == type_b && field_a != "id" && field_a != "name" {
        return Some(RelationshipCandidate {
            from_node: node_a.into(), from_field: field_a.into(),
            to_node: node_b.into(), to_field: field_b.into(),
            confidence: 0.60,
            reason: format!("Same field '{}' with same type in both tables", field_a),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_schema() -> UniversalSchema {
        let mut s = UniversalSchema::new("test");
        s.nodes.push(SchemaNode { name: "users".into(), source: "db".into(),
            fields: vec![SchemaField::inferred("id", FieldType::Integer, 1.0), SchemaField::inferred("name", FieldType::Text, 1.0)],
            record_count: Some(100) });
        s.nodes.push(SchemaNode { name: "orders".into(), source: "db".into(),
            fields: vec![SchemaField::inferred("id", FieldType::Integer, 1.0), SchemaField::inferred("user_id", FieldType::Integer, 1.0)],
            record_count: Some(500) });
        s.edges.push(SchemaEdge { from: "orders.user_id".into(), to: "users.id".into(),
            edge_type: SchemaEdgeType::References, confidence: 0.95 });
        s
    }

    #[test]
    fn test_bfs_traversal() {
        let schema = test_schema();
        let result = GraphEngine::traverse_schema(&schema, "users", TraversalDirection::Both, 3);
        assert!(result.visited.contains(&"users".to_string()));
        assert!(result.visited.contains(&"orders".to_string()));
    }

    #[test]
    fn test_connected_components() {
        let schema = test_schema();
        let components = GraphEngine::connected_components(&schema);
        assert_eq!(components.len(), 1); // All connected
    }

    #[test]
    fn test_centrality() {
        let schema = test_schema();
        let centrality = GraphEngine::node_centrality(&schema);
        assert!(centrality.get("users").unwrap() > &0.0);
        assert!(centrality.get("orders").unwrap() > &0.0);
    }

    #[test]
    fn test_discover_relationships() {
        let mut store = DataStore::new();
        let schema = test_schema();
        store.add_schema(schema);
        let candidates = GraphEngine::discover_relationships(&store);
        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|c| c.from_field == "user_id" && c.to_field == "id"));
    }
}
