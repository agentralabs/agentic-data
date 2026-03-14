//! Schema index — fast lookup of schemas, nodes, fields, and relationships.

use std::collections::HashMap;
use crate::types::*;

/// Index for fast schema lookups.
#[derive(Debug, Default)]
pub struct SchemaIndex {
    /// Schema ID → schema.
    schemas: HashMap<String, UniversalSchema>,
    /// Field qualified name → (schema_id, node_name, field_index).
    field_map: HashMap<String, (String, String, usize)>,
    /// Node name → list of schema IDs containing that node.
    node_to_schemas: HashMap<String, Vec<String>>,
}

impl SchemaIndex {
    pub fn new() -> Self { Self::default() }

    /// Index a schema.
    pub fn add(&mut self, schema: &UniversalSchema) {
        self.schemas.insert(schema.id.clone(), schema.clone());
        for node in &schema.nodes {
            self.node_to_schemas
                .entry(node.name.clone())
                .or_default()
                .push(schema.id.clone());
            for (i, field) in node.fields.iter().enumerate() {
                let key = format!("{}.{}", node.name, field.name);
                self.field_map.insert(key, (schema.id.clone(), node.name.clone(), i));
            }
        }
    }

    /// Look up a field by qualified name "node.field".
    pub fn find_field(&self, qualified: &str) -> Option<&SchemaField> {
        let (schema_id, node_name, idx) = self.field_map.get(qualified)?;
        let schema = self.schemas.get(schema_id)?;
        let node = schema.nodes.iter().find(|n| &n.name == node_name)?;
        node.fields.get(*idx)
    }

    /// Find all schemas containing a node.
    pub fn schemas_for_node(&self, node: &str) -> Vec<&UniversalSchema> {
        self.node_to_schemas.get(node)
            .map(|ids| ids.iter().filter_map(|id| self.schemas.get(id)).collect())
            .unwrap_or_default()
    }

    /// Find fields matching a type across all schemas.
    pub fn fields_of_type(&self, field_type: &FieldType) -> Vec<(String, &SchemaField)> {
        let mut results = Vec::new();
        for schema in self.schemas.values() {
            for node in &schema.nodes {
                for field in &node.fields {
                    if &field.field_type == field_type {
                        results.push((format!("{}.{}", node.name, field.name), field));
                    }
                }
            }
        }
        results
    }

    /// Total indexed field count.
    pub fn field_count(&self) -> usize { self.field_map.len() }

    /// Total indexed schema count.
    pub fn schema_count(&self) -> usize { self.schemas.len() }

    /// Search fields by name pattern (case-insensitive substring).
    pub fn search_fields(&self, pattern: &str) -> Vec<(String, &SchemaField)> {
        let lower = pattern.to_lowercase();
        let mut results = Vec::new();
        for (key, (schema_id, node_name, idx)) in &self.field_map {
            if key.to_lowercase().contains(&lower) {
                if let Some(schema) = self.schemas.get(schema_id) {
                    if let Some(node) = schema.nodes.iter().find(|n| &n.name == node_name) {
                        if let Some(field) = node.fields.get(*idx) {
                            results.push((key.clone(), field));
                        }
                    }
                }
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_schema() -> UniversalSchema {
        let mut s = UniversalSchema::new("test");
        s.nodes.push(SchemaNode {
            name: "users".into(), source: "db".into(),
            fields: vec![
                SchemaField::inferred("id", FieldType::Integer, 1.0),
                SchemaField::inferred("email", FieldType::Email, 0.95),
                SchemaField::inferred("name", FieldType::Text, 1.0),
            ],
            record_count: Some(100),
        });
        s
    }

    #[test]
    fn test_add_and_find() {
        let mut idx = SchemaIndex::new();
        idx.add(&test_schema());
        assert_eq!(idx.schema_count(), 1);
        assert_eq!(idx.field_count(), 3);
        let f = idx.find_field("users.email").unwrap();
        assert_eq!(f.field_type, FieldType::Email);
    }

    #[test]
    fn test_fields_of_type() {
        let mut idx = SchemaIndex::new();
        idx.add(&test_schema());
        let emails = idx.fields_of_type(&FieldType::Email);
        assert_eq!(emails.len(), 1);
        assert_eq!(emails[0].0, "users.email");
    }

    #[test]
    fn test_search_fields() {
        let mut idx = SchemaIndex::new();
        idx.add(&test_schema());
        let results = idx.search_fields("mail");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_schemas_for_node() {
        let mut idx = SchemaIndex::new();
        idx.add(&test_schema());
        assert_eq!(idx.schemas_for_node("users").len(), 1);
        assert_eq!(idx.schemas_for_node("missing").len(), 0);
    }
}
