//! GraphQL schema to Rust entity code generation.
//!
//! Parses schema.graphql files and generates Rust entity structs with
//! builder-pattern setters and a `save()` method backed by graph-as-runtime.

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
    writeln!(output, "#![allow(unused_imports)]")?;
    writeln!(output, "#![allow(unused_mut)]")?;
    writeln!(output)?;
    writeln!(output, "extern crate alloc;")?;
    writeln!(output)?;
    writeln!(output, "use alloc::string::String;")?;
    writeln!(output, "use alloc::vec::Vec;")?;
    writeln!(output)?;

    // Process all @entity type definitions
    for definition in &document.definitions {
        if let Definition::TypeDefinition(type_def) = definition {
            if let TypeDefinition::Object(obj) = type_def {
                // Skip built-in types
                if obj.name.starts_with("__")
                    || obj.name == "Query"
                    || obj.name == "Mutation"
                    || obj.name == "Subscription"
                {
                    continue;
                }

                // Only process @entity types
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

/// Generate a Rust struct for a GraphQL entity type with raw types and EntityBuilder.
fn generate_entity_struct<'a>(name: &str, fields: &'a [Field<'a, String>]) -> Result<String> {
    let mut output = String::new();

    // Validate id field exists
    let _id_field = fields
        .iter()
        .find(|f| f.name == "id")
        .ok_or_else(|| anyhow::anyhow!("Entity {} must have an 'id' field", name))?;

    writeln!(output, "/// Generated from `{}` entity in schema.graphql.", name)?;
    writeln!(output, "pub struct {} {{", name)?;

    // Emit fields
    for field in fields {
        let field_name = field.name.to_snake_case();
        let (rust_type, optional) = graphql_type_to_rust(&field.field_type);

        if field.name == "id" {
            writeln!(output, "    id: alloc::string::String,")?;
        } else if optional {
            writeln!(output, "    {}: Option<{}>,", field_name, rust_type)?;
        } else {
            writeln!(output, "    {}: Option<{}>,", field_name, rust_type)?;
        }
    }

    writeln!(output, "}}")?;
    writeln!(output)?;

    // impl block
    writeln!(output, "impl {} {{", name)?;

    // new()
    writeln!(output, "    pub fn new(id: &str) -> Self {{")?;
    writeln!(output, "        Self {{")?;
    writeln!(output, "            id: id.into(),")?;
    for field in fields {
        if field.name == "id" {
            continue;
        }
        let field_name = field.name.to_snake_case();
        writeln!(output, "            {}: None,", field_name)?;
    }
    writeln!(output, "        }}")?;
    writeln!(output, "    }}")?;
    writeln!(output)?;

    // set_* builder methods
    for field in fields {
        if field.name == "id" {
            continue;
        }
        let field_name = field.name.to_snake_case();
        let (rust_type, _) = graphql_type_to_rust(&field.field_type);
        let setter_name = format!("set_{}", field_name);

        writeln!(output, "    pub fn {}(mut self, v: {}) -> Self {{", setter_name, rust_type)?;
        writeln!(output, "        self.{} = Some(v);", field_name)?;
        writeln!(output, "        self")?;
        writeln!(output, "    }}")?;
        writeln!(output)?;
    }

    // save() — only available on WASM (needs AS allocator + FFI)
    writeln!(output, "    #[cfg(target_arch = \"wasm32\")]")?;
    writeln!(output, "    pub fn save(&self) {{")?;
    writeln!(
        output,
        "        let mut b = graph_as_runtime::entity::EntityBuilder::new();"
    )?;
    writeln!(output, "        b.set_string(\"id\", &self.id);")?;

    for field in fields {
        if field.name == "id" {
            continue;
        }
        let field_name = field.name.to_snake_case();
        let gql_name = &field.name;
        let builder_call = graphql_field_to_builder_call(&field.field_type, &field_name, gql_name);
        output.push_str(&builder_call);
    }

    writeln!(output, "        let entity_ptr = graph_as_runtime::as_types::new_asc_string({:?});", name)?;
    writeln!(output, "        let id_ptr = graph_as_runtime::as_types::new_asc_string(&self.id);")?;
    writeln!(output, "        unsafe {{")?;
    writeln!(
        output,
        "            graph_as_runtime::ffi::store_set(entity_ptr, id_ptr, b.build());"
    )?;
    writeln!(output, "        }}")?;
    writeln!(output, "    }}")?;

    writeln!(output, "}}")?;

    Ok(output)
}

/// Generate the `b.set_*` call inside `save()` for a given field.
fn graphql_field_to_builder_call<'a>(ty: &'a Type<'a, String>, field_name: &str, gql_name: &str) -> String {
    let base_scalar = get_base_scalar(ty);

    // List types are not directly serialisable as a single store field; skip them.
    if matches!(ty, Type::ListType(_))
        || matches!(ty, Type::NonNullType(inner) if matches!(inner.as_ref(), Type::ListType(_)))
    {
        return format!(
            "        // skipped list field `{}` (not directly storable)\n",
            gql_name
        );
    }

    match base_scalar {
        "ID" | "String" => {
            format!(
                "        if let Some(ref v) = self.{} {{ b.set_string({:?}, v); }}\n",
                field_name, gql_name
            )
        }
        "Boolean" => {
            format!(
                "        if let Some(v) = self.{} {{ b.set_bool({:?}, v); }}\n",
                field_name, gql_name
            )
        }
        "Int" => {
            format!(
                "        if let Some(v) = self.{} {{ b.set_i32({:?}, v); }}\n",
                field_name, gql_name
            )
        }
        "BigInt" | "BigDecimal" => {
            format!(
                "        if let Some(ref v) = self.{} {{ b.set_bigint({:?}, v); }}\n",
                field_name, gql_name
            )
        }
        "Bytes" | "Address" => {
            format!(
                "        if let Some(ref v) = self.{} {{ b.set_bytes({:?}, v); }}\n",
                field_name, gql_name
            )
        }
        _ => {
            // Entity reference — stored as a string ID
            format!(
                "        if let Some(ref v) = self.{} {{ b.set_string({:?}, v); }}\n",
                field_name, gql_name
            )
        }
    }
}

/// Extract the base scalar name from a possibly wrapped GraphQL type.
fn get_base_scalar<'a>(ty: &'a Type<'a, String>) -> &'a str
where
    String: 'a,
{
    match ty {
        Type::NamedType(name) => name.as_str(),
        Type::NonNullType(inner) => get_base_scalar(inner),
        Type::ListType(_) => "Bytes", // lists → bytes fallback
    }
}

/// Convert a GraphQL type to Rust type + whether it's optional.
/// Returns (rust_type_string, is_optional).
///
/// All fields on an entity are stored as Option<T> in the struct to support
/// the builder pattern, regardless of non-null in the schema.
fn graphql_type_to_rust(ty: &Type<'_, String>) -> (String, bool) {
    let is_nullable = !matches!(ty, Type::NonNullType(_));
    let rust_type = graphql_type_to_rust_inner(ty);
    (rust_type, is_nullable)
}

fn graphql_type_to_rust_inner(ty: &Type<'_, String>) -> String {
    match ty {
        Type::NamedType(name) => scalar_to_rust(name),
        Type::NonNullType(inner) => graphql_type_to_rust_inner(inner),
        Type::ListType(inner) => {
            format!("Vec<{}>", graphql_type_to_rust_inner(inner))
        }
    }
}

/// Convert a GraphQL scalar name to its raw Rust type.
fn scalar_to_rust(name: &str) -> String {
    match name {
        "ID" | "String" => "alloc::string::String".to_string(),
        "Int" => "i32".to_string(),
        "Float" => "f64".to_string(),
        "Boolean" => "bool".to_string(),
        "BigInt" | "BigDecimal" => "Vec<u8>".to_string(),
        "Bytes" | "Address" => "Vec<u8>".to_string(),
        // Entity references are stored by their ID string
        _other => "alloc::string::String".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_mapping() {
        assert_eq!(scalar_to_rust("ID"), "alloc::string::String");
        assert_eq!(scalar_to_rust("String"), "alloc::string::String");
        assert_eq!(scalar_to_rust("BigInt"), "Vec<u8>");
        assert_eq!(scalar_to_rust("Bytes"), "Vec<u8>");
        assert_eq!(scalar_to_rust("Boolean"), "bool");
    }
}
