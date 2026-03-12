//! GraphQL schema to Rust entity code generation.
//!
//! Parses schema.graphql files and generates Rust structs with `#[derive(Entity)]`.

use anyhow::{Context, Result};
use graphql_parser::schema::{Definition, Document, Field, Type, TypeDefinition};
use heck::ToSnakeCase;
use std::fmt::Write;
use std::path::Path;

/// Generate Rust entity structs from a GraphQL schema.
pub fn generate_schema_entities(schema_path: &Path) -> Result<String> {
    let schema_str = std::fs::read_to_string(schema_path)
        .with_context(|| format!("Failed to read schema: {}", schema_path.display()))?;

    let document: Document<'_, String> = graphql_parser::parse_schema(&schema_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse schema: {}", e))?;

    let mut output = String::new();

    // File header
    writeln!(output, "//! Generated entity types from schema.graphql.")?;
    writeln!(output, "//!")?;
    writeln!(output, "//! DO NOT EDIT — regenerate with `graphite codegen`")?;
    writeln!(output)?;
    writeln!(output, "#![allow(dead_code)]")?;
    writeln!(output)?;
    writeln!(output, "use graphite::prelude::*;")?;
    writeln!(output)?;

    // Process all type definitions
    for definition in &document.definitions {
        if let Definition::TypeDefinition(type_def) = definition {
            if let TypeDefinition::Object(obj) = type_def {
                // Skip built-in types and Query/Mutation/Subscription
                if obj.name.starts_with("__")
                    || obj.name == "Query"
                    || obj.name == "Mutation"
                    || obj.name == "Subscription"
                {
                    continue;
                }

                // Check if it has @entity directive
                let is_entity = obj.directives.iter().any(|d| d.name == "entity");
                if !is_entity {
                    continue;
                }

                let entity_code = generate_entity_struct(&obj.name, &obj.fields)?;
                output.push_str(&entity_code);
                output.push('\n');
            }
        }
    }

    Ok(output)
}

/// Generate a Rust struct for a GraphQL entity type.
fn generate_entity_struct(name: &str, fields: &[Field<'_, String>]) -> Result<String> {
    let mut output = String::new();

    // Find the ID field
    let id_field = fields
        .iter()
        .find(|f| f.name == "id")
        .ok_or_else(|| anyhow::anyhow!("Entity {} must have an 'id' field", name))?;

    // Validate ID is correct type
    let id_type = graphql_type_to_rust(&id_field.field_type);
    if id_type != "String" && id_type != "Bytes" {
        anyhow::bail!(
            "Entity {} id field must be ID!, String!, or Bytes!, got {}",
            name,
            format_graphql_type(&id_field.field_type)
        );
    }

    // Doc comment with field list
    writeln!(output, "/// Entity: `{}`", name)?;
    writeln!(output, "///")?;
    writeln!(output, "/// Fields:")?;
    for field in fields {
        let required = is_required(&field.field_type);
        let marker = if required { "" } else { " (optional)" };
        writeln!(
            output,
            "/// - `{}`: `{}`{}",
            field.name,
            format_graphql_type(&field.field_type),
            marker
        )?;
    }

    writeln!(output, "#[derive(Entity, Debug, Clone, PartialEq)]")?;
    writeln!(output, "pub struct {} {{", name)?;

    for field in fields {
        let field_name = field.name.to_snake_case();
        let rust_type = graphql_type_to_rust(&field.field_type);

        // Add #[id] attribute to the id field
        if field.name == "id" {
            writeln!(output, "    #[id]")?;
        }

        writeln!(output, "    pub {}: {},", field_name, rust_type)?;
    }

    writeln!(output, "}}")?;

    Ok(output)
}

/// Convert a GraphQL type to its Rust equivalent.
fn graphql_type_to_rust(ty: &Type<'_, String>) -> String {
    match ty {
        Type::NamedType(name) => scalar_to_rust(name),

        Type::NonNullType(inner) => {
            // Non-null just uses the inner type directly (no Option wrapper)
            graphql_type_to_rust(inner)
        }

        Type::ListType(inner) => {
            let inner_type = graphql_type_to_rust(inner);
            format!("Vec<{}>", inner_type)
        }
    }
}

/// Convert a GraphQL scalar/type name to Rust.
fn scalar_to_rust(name: &str) -> String {
    match name {
        // Built-in GraphQL scalars
        "ID" | "String" => "String".to_string(),
        "Int" => "i32".to_string(),
        "Float" => "f64".to_string(),
        "Boolean" => "bool".to_string(),

        // The Graph custom scalars
        "BigInt" => "BigInt".to_string(),
        "BigDecimal" => "BigDecimal".to_string(),
        "Bytes" => "Bytes".to_string(),
        "Address" => "Address".to_string(),

        // Assume other types are entity references (stored as ID strings)
        other => format!("String /* {} */", other),
    }
}

/// Check if a GraphQL type is required (non-null at the top level).
fn is_required(ty: &Type<'_, String>) -> bool {
    matches!(ty, Type::NonNullType(_))
}

/// Format a GraphQL type for documentation.
fn format_graphql_type(ty: &Type<'_, String>) -> String {
    match ty {
        Type::NamedType(name) => name.clone(),
        Type::NonNullType(inner) => format!("{}!", format_graphql_type(inner)),
        Type::ListType(inner) => format!("[{}]", format_graphql_type(inner)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_mapping() {
        assert_eq!(scalar_to_rust("ID"), "String");
        assert_eq!(scalar_to_rust("String"), "String");
        assert_eq!(scalar_to_rust("BigInt"), "BigInt");
        assert_eq!(scalar_to_rust("Bytes"), "Bytes");
        assert_eq!(scalar_to_rust("Boolean"), "bool");
    }

    #[test]
    fn type_to_rust() {
        // Simple named type (nullable by default in GraphQL)
        let named = Type::NamedType("String".to_string());
        assert_eq!(graphql_type_to_rust(&named), "String");

        // Non-null type
        let non_null = Type::NonNullType(Box::new(Type::NamedType("BigInt".to_string())));
        assert_eq!(graphql_type_to_rust(&non_null), "BigInt");

        // List type
        let list = Type::ListType(Box::new(Type::NamedType("Address".to_string())));
        assert_eq!(graphql_type_to_rust(&list), "Vec<Address>");
    }
}
