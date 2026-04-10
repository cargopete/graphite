//! GraphQL schema to Rust entity code generation.
//!
//! Parses schema.graphql files and generates Rust entity structs with
//! builder-pattern setters and a `save()` method backed by graph-as-runtime.

use anyhow::{Context, Result};
use graphql_parser::query::Value as GqlValue;
use graphql_parser::schema::{Definition, Document, Field, Type, TypeDefinition};
use heck::ToSnakeCase;
use std::fmt::Write;
use std::path::Path;

/// Generate Rust entity structs from a GraphQL schema.
pub fn generate_schema_entities(schema_path: &Path) -> Result<String> {
    let schema_str = std::fs::read_to_string(schema_path)
        .with_context(|| format!("Failed to read schema: {}", schema_path.display()))?;
    generate_schema_entities_from_str(&schema_str)
}

fn generate_schema_entities_from_str(schema_str: &str) -> Result<String> {
    let document: Document<'_, String> = graphql_parser::parse_schema(schema_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse schema: {}", e))?;

    let mut output = String::new();

    // File header
    writeln!(output, "//! Generated entity types from schema.graphql.")?;
    writeln!(output, "//!")?;
    writeln!(
        output,
        "//! DO NOT EDIT — regenerate with `graphite codegen`"
    )?;
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
                // Skip built-in and graph-node special types.
                // _Schema_ holds @fulltext directives — graph-node handles it at deploy time.
                if obj.name.starts_with("__")
                    || obj.name == "_Schema_"
                    || obj.name == "Query"
                    || obj.name == "Mutation"
                    || obj.name == "Subscription"
                {
                    continue;
                }

                // Skip @aggregation types — graph-node auto-computes them from timeseries
                // entities; handlers never write to them directly.
                if obj.directives.iter().any(|d| d.name == "aggregation") {
                    continue;
                }

                // Only process @entity types
                let entity_directive = obj.directives.iter().find(|d| d.name == "entity");
                if entity_directive.is_none() {
                    continue;
                }
                let is_immutable = entity_directive
                    .unwrap()
                    .arguments
                    .iter()
                    .any(|(k, v)| k == "immutable" && matches!(v, GqlValue::Boolean(true)));

                // @entity(timeseries: true) entities have auto-managed IDs and timestamps;
                // they are otherwise generated identically to regular entities.
                let entity_code =
                    generate_entity_struct(&obj.name, &obj.fields, is_immutable)?;
                output.push_str(&entity_code);
                output.push('\n');
            }
        }
    }

    Ok(output)
}

/// Generate a Rust struct for a GraphQL entity type with raw types and EntityBuilder.
fn generate_entity_struct<'a>(
    name: &str,
    fields: &'a [Field<'a, String>],
    immutable: bool,
) -> Result<String> {
    let mut output = String::new();

    // Validate id field exists
    let _id_field = fields
        .iter()
        .find(|f| f.name == "id")
        .ok_or_else(|| anyhow::anyhow!("Entity {} must have an 'id' field", name))?;

    // Collect names of @derivedFrom fields — they are computed by graph-node at
    // query time and must not appear in the stored struct or save/load logic.
    let derived: std::collections::HashSet<&str> = fields
        .iter()
        .filter(|f| f.directives.iter().any(|d| d.name == "derivedFrom"))
        .map(|f| f.name.as_str())
        .collect();

    writeln!(
        output,
        "/// Generated from `{}` entity in schema.graphql.",
        name
    )?;
    if immutable {
        writeln!(output, "/// This entity is immutable (`@entity(immutable: true)`). Use `save()` once at creation; updates and `remove()` are not supported.")?;
    }
    writeln!(output, "pub struct {} {{", name)?;

    // Emit fields — non-nullable fields use `T`, nullable fields use `Option<T>`.
    // @derivedFrom fields are skipped — they're computed by graph-node, not stored.
    for field in fields {
        if derived.contains(field.name.as_str()) {
            continue;
        }
        let field_name = field.name.to_snake_case();
        let (rust_type, nullable) = graphql_type_to_rust(&field.field_type);

        if field.name == "id" {
            writeln!(output, "    id: alloc::string::String,")?;
        } else if nullable {
            writeln!(output, "    {}: Option<{}>,", field_name, rust_type)?;
        } else {
            writeln!(output, "    {}: {},", field_name, rust_type)?;
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
        if field.name == "id" || derived.contains(field.name.as_str()) {
            continue;
        }
        let field_name = field.name.to_snake_case();
        let (_, nullable) = graphql_type_to_rust(&field.field_type);
        if nullable {
            writeln!(output, "            {}: None,", field_name)?;
        } else {
            writeln!(output, "            {}: Default::default(),", field_name)?;
        }
    }
    writeln!(output, "        }}")?;
    writeln!(output, "    }}")?;
    writeln!(output)?;

    // get_* accessor methods (returns reference or Option<&T>)
    for field in fields {
        if field.name == "id" || derived.contains(field.name.as_str()) {
            continue;
        }
        let field_name = field.name.to_snake_case();
        let (rust_type, nullable) = graphql_type_to_rust(&field.field_type);
        if nullable {
            writeln!(
                output,
                "    pub fn {}(&self) -> Option<&{}> {{ self.{}.as_ref() }}",
                field_name, rust_type, field_name
            )?;
        } else {
            writeln!(
                output,
                "    pub fn {}(&self) -> &{} {{ &self.{} }}",
                field_name, rust_type, field_name
            )?;
        }
        writeln!(output)?;
    }

    // set_* builder methods
    for field in fields {
        if field.name == "id" || derived.contains(field.name.as_str()) {
            continue;
        }
        let field_name = field.name.to_snake_case();
        let (rust_type, nullable) = graphql_type_to_rust(&field.field_type);
        let setter_name = format!("set_{}", field_name);

        writeln!(
            output,
            "    pub fn {}(mut self, v: {}) -> Self {{",
            setter_name, rust_type
        )?;
        if nullable {
            writeln!(output, "        self.{} = Some(v);", field_name)?;
        } else {
            writeln!(output, "        self.{} = v;", field_name)?;
        }
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
        if field.name == "id" || derived.contains(field.name.as_str()) {
            continue;
        }
        let field_name = field.name.to_snake_case();
        let gql_name = &field.name;
        let (_, nullable) = graphql_type_to_rust(&field.field_type);
        let builder_call =
            graphql_field_to_builder_call(&field.field_type, &field_name, gql_name, nullable);
        output.push_str(&builder_call);
    }

    writeln!(
        output,
        "        let entity_ptr = graph_as_runtime::as_types::new_asc_string({:?});",
        name
    )?;
    writeln!(
        output,
        "        let id_ptr = graph_as_runtime::as_types::new_asc_string(&self.id);"
    )?;
    writeln!(output, "        unsafe {{")?;
    writeln!(
        output,
        "            graph_as_runtime::ffi::store_set(entity_ptr, id_ptr, b.build());"
    )?;
    writeln!(output, "        }}")?;
    writeln!(output, "    }}")?;
    writeln!(output)?;

    // save() — native (non-wasm32) implementation for unit testing
    writeln!(output, "    #[cfg(not(target_arch = \"wasm32\"))]")?;
    writeln!(output, "    pub fn save(&self) {{")?;
    writeln!(output, "        use std::collections::HashMap;")?;
    writeln!(
        output,
        "        use graph_as_runtime::native_store::{{FieldValue, STORE}};"
    )?;
    writeln!(output, "        let mut fields = HashMap::new();")?;
    writeln!(
        output,
        "        fields.insert(\"id\".to_string(), FieldValue::String(self.id.clone()));"
    )?;

    for field in fields {
        if field.name == "id" || derived.contains(field.name.as_str()) {
            continue;
        }
        let field_name = field.name.to_snake_case();
        let gql_name = &field.name;
        let (_, nullable) = graphql_type_to_rust(&field.field_type);
        let native_call =
            graphql_field_to_native_store_call(&field.field_type, &field_name, gql_name, nullable);
        output.push_str(&native_call);
    }

    writeln!(
        output,
        "        STORE.with(|s| s.borrow_mut().set_entity({:?}, &self.id, fields));",
        name
    )?;
    writeln!(output, "    }}")?;
    writeln!(output)?;

    // load() — WASM: reads from graph-node via store.get + AS memory walk
    writeln!(output, "    #[cfg(target_arch = \"wasm32\")]")?;
    writeln!(output, "    pub fn load(id: &str) -> Option<Self> {{")?;
    writeln!(
        output,
        "        let entity_ptr = graph_as_runtime::as_types::new_asc_string({:?});",
        name
    )?;
    writeln!(
        output,
        "        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);"
    )?;
    writeln!(
        output,
        "        let map_ptr = unsafe {{ graph_as_runtime::ffi::store_get(entity_ptr, id_ptr) }};"
    )?;
    writeln!(output, "        if map_ptr == 0 {{")?;
    writeln!(output, "            return None;")?;
    writeln!(output, "        }}")?;
    writeln!(
        output,
        "        let fields = unsafe {{ graph_as_runtime::store_read::read_typed_map(map_ptr) }};"
    )?;
    writeln!(
        output,
        "        let get = |k: &str| fields.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());"
    )?;
    writeln!(output, "        Some(Self {{")?;
    writeln!(output, "            id: id.into(),")?;
    for field in fields {
        if field.name == "id" || derived.contains(field.name.as_str()) {
            continue;
        }
        let field_name = field.name.to_snake_case();
        let gql_name = &field.name;
        let (_, nullable) = graphql_type_to_rust(&field.field_type);
        let decode = wasm_load_field_decode(&field.field_type, &field_name, gql_name, nullable);
        output.push_str(&decode);
    }
    writeln!(output, "        }})")?;
    writeln!(output, "    }}")?;
    writeln!(output)?;

    // load() — native: reads from the thread-local NativeStore
    writeln!(output, "    #[cfg(not(target_arch = \"wasm32\"))]")?;
    writeln!(output, "    pub fn load(id: &str) -> Option<Self> {{")?;
    writeln!(
        output,
        "        use graph_as_runtime::native_store::{{FieldValue, STORE}};"
    )?;
    writeln!(
        output,
        "        let fields = STORE.with(|s| s.borrow().get_entity({:?}, id).cloned())?;",
        name
    )?;
    writeln!(output, "        Some(Self {{")?;
    writeln!(output, "            id: id.into(),")?;
    for field in fields {
        if field.name == "id" || derived.contains(field.name.as_str()) {
            continue;
        }
        let field_name = field.name.to_snake_case();
        let gql_name = &field.name;
        let (_, nullable) = graphql_type_to_rust(&field.field_type);
        let decode = native_load_field_decode(&field.field_type, &field_name, gql_name, nullable);
        output.push_str(&decode);
    }
    writeln!(output, "        }})")?;
    writeln!(output, "    }}")?;

    if !immutable {
        writeln!(output)?;
        // remove() — WASM
        writeln!(output, "    #[cfg(target_arch = \"wasm32\")]")?;
        writeln!(output, "    pub fn remove(id: &str) {{")?;
        writeln!(
            output,
            "        let entity_ptr = graph_as_runtime::as_types::new_asc_string({:?});",
            name
        )?;
        writeln!(
            output,
            "        let id_ptr = graph_as_runtime::as_types::new_asc_string(id);"
        )?;
        writeln!(output, "        unsafe {{")?;
        writeln!(
            output,
            "            graph_as_runtime::ffi::store_remove(entity_ptr, id_ptr);"
        )?;
        writeln!(output, "        }}")?;
        writeln!(output, "    }}")?;
        writeln!(output)?;
        // remove() — native
        writeln!(output, "    #[cfg(not(target_arch = \"wasm32\"))]")?;
        writeln!(output, "    pub fn remove(id: &str) {{")?;
        writeln!(
            output,
            "        use graph_as_runtime::native_store::STORE;"
        )?;
        writeln!(
            output,
            "        STORE.with(|s| s.borrow_mut().remove_entity({:?}, id));",
            name
        )?;
        writeln!(output, "    }}")?;
    }

    writeln!(output, "}}")?;

    Ok(output)
}

/// Generate the field decode line for WASM `load()` using `StoreValue`.
fn wasm_load_field_decode<'a>(
    ty: &'a Type<'a, String>,
    field_name: &str,
    gql_name: &str,
    nullable: bool,
) -> String {
    if matches!(ty, Type::ListType(_))
        || matches!(ty, Type::NonNullType(inner) if matches!(inner.as_ref(), Type::ListType(_)))
    {
        return if nullable {
            format!("            {}: None,\n", field_name)
        } else {
            format!("            {}: Default::default(),\n", field_name)
        };
    }
    let base = get_base_scalar(ty);
    let expr = match base {
        "ID" | "String" => format!(
            "get({:?}).and_then(|v| v.as_string().map(|s| s.to_string()))",
            gql_name
        ),
        "Boolean" => format!("get({:?}).and_then(|v| v.as_bool())", gql_name),
        "Int" => format!("get({:?}).and_then(|v| v.as_i32())", gql_name),
        "Timestamp" | "Int8" => format!("get({:?}).and_then(|v| v.as_i64())", gql_name),
        _ => format!("get({:?}).and_then(|v| v.as_string().map(|s| s.to_string()))", gql_name),
    };
    if nullable {
        format!("            {}: {},\n", field_name, expr)
    } else {
        format!(
            "            {}: {}.unwrap_or_default(),\n",
            field_name, expr
        )
    }
}

/// Generate the field decode line for native `load()` using `FieldValue`.
fn native_load_field_decode<'a>(
    ty: &'a Type<'a, String>,
    field_name: &str,
    gql_name: &str,
    nullable: bool,
) -> String {
    if matches!(ty, Type::ListType(_))
        || matches!(ty, Type::NonNullType(inner) if matches!(inner.as_ref(), Type::ListType(_)))
    {
        return if nullable {
            format!("            {}: None,\n", field_name)
        } else {
            format!("            {}: Default::default(),\n", field_name)
        };
    }
    let base = get_base_scalar(ty);
    let expr = match base {
        "ID" | "String" => format!(
            "fields.get({:?}).and_then(|v| if let FieldValue::String(s) = v {{ Some(s.clone()) }} else {{ None }})",
            gql_name
        ),
        "Boolean" => format!(
            "fields.get({:?}).and_then(|v| if let FieldValue::Bool(b) = v {{ Some(*b) }} else {{ None }})",
            gql_name
        ),
        "Int" => format!(
            "fields.get({:?}).and_then(|v| if let FieldValue::Int(n) = v {{ Some(*n) }} else {{ None }})",
            gql_name
        ),
        "Timestamp" | "Int8" => format!(
            "fields.get({:?}).and_then(|v| if let FieldValue::Int8(n) = v {{ Some(*n) }} else {{ None }})",
            gql_name
        ),
        "BigInt" | "BigDecimal" => format!(
            "fields.get({:?}).and_then(|v| if let FieldValue::BigInt(b) = v {{ Some(b.clone()) }} else {{ None }})",
            gql_name
        ),
        _ => format!(
            "fields.get({:?}).and_then(|v| if let FieldValue::String(s) = v {{ Some(s.clone()) }} else {{ None }})",
            gql_name
        ),
    };
    if nullable {
        format!("            {}: {},\n", field_name, expr)
    } else {
        format!("            {}: {}.unwrap_or_default(),\n", field_name, expr)
    }
}

/// Generate the native `fields.insert(...)` call for a given field.
fn graphql_field_to_native_store_call<'a>(
    ty: &'a Type<'a, String>,
    field_name: &str,
    gql_name: &str,
    nullable: bool,
) -> String {
    // Skip list types
    if matches!(ty, Type::ListType(_))
        || matches!(ty, Type::NonNullType(inner) if matches!(inner.as_ref(), Type::ListType(_)))
    {
        return format!(
            "        // skipped list field `{}` (not directly storable)\n",
            gql_name
        );
    }

    let base_scalar = get_base_scalar(ty);
    if nullable {
        match base_scalar {
            "ID" | "String" => format!(
                "        if let Some(ref v) = self.{} {{ fields.insert({:?}.to_string(), FieldValue::String(v.clone())); }}\n",
                field_name, gql_name
            ),
            "Boolean" => format!(
                "        if let Some(v) = self.{} {{ fields.insert({:?}.to_string(), FieldValue::Bool(v)); }}\n",
                field_name, gql_name
            ),
            "Int" => format!(
                "        if let Some(v) = self.{} {{ fields.insert({:?}.to_string(), FieldValue::Int(v)); }}\n",
                field_name, gql_name
            ),
            "Timestamp" | "Int8" => format!(
                "        if let Some(v) = self.{} {{ fields.insert({:?}.to_string(), FieldValue::Int8(v)); }}\n",
                field_name, gql_name
            ),
            "BigInt" | "BigDecimal" => format!(
                "        if let Some(ref v) = self.{} {{ fields.insert({:?}.to_string(), FieldValue::BigInt(v.clone())); }}\n",
                field_name, gql_name
            ),
            "Bytes" | "Address" => format!(
                "        if let Some(ref v) = self.{} {{ fields.insert({:?}.to_string(), FieldValue::Bytes(v.clone())); }}\n",
                field_name, gql_name
            ),
            _ => format!(
                "        if let Some(ref v) = self.{} {{ fields.insert({:?}.to_string(), FieldValue::String(v.clone())); }}\n",
                field_name, gql_name
            ),
        }
    } else {
        match base_scalar {
            "ID" | "String" => format!(
                "        fields.insert({:?}.to_string(), FieldValue::String(self.{}.clone()));\n",
                gql_name, field_name
            ),
            "Boolean" => format!(
                "        fields.insert({:?}.to_string(), FieldValue::Bool(self.{}));\n",
                gql_name, field_name
            ),
            "Int" => format!(
                "        fields.insert({:?}.to_string(), FieldValue::Int(self.{}));\n",
                gql_name, field_name
            ),
            "Timestamp" | "Int8" => format!(
                "        fields.insert({:?}.to_string(), FieldValue::Int8(self.{}));\n",
                gql_name, field_name
            ),
            "BigInt" | "BigDecimal" => format!(
                "        fields.insert({:?}.to_string(), FieldValue::BigInt(self.{}.clone()));\n",
                gql_name, field_name
            ),
            "Bytes" | "Address" => format!(
                "        fields.insert({:?}.to_string(), FieldValue::Bytes(self.{}.clone()));\n",
                gql_name, field_name
            ),
            _ => format!(
                "        fields.insert({:?}.to_string(), FieldValue::String(self.{}.clone()));\n",
                gql_name, field_name
            ),
        }
    }
}

/// Generate the `b.set_*` call inside `save()` for a given field.
fn graphql_field_to_builder_call<'a>(
    ty: &'a Type<'a, String>,
    field_name: &str,
    gql_name: &str,
    nullable: bool,
) -> String {
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

    if nullable {
        match base_scalar {
            "ID" | "String" => format!(
                "        if let Some(ref v) = self.{} {{ b.set_string({:?}, v); }}\n",
                field_name, gql_name
            ),
            "Boolean" => format!(
                "        if let Some(v) = self.{} {{ b.set_bool({:?}, v); }}\n",
                field_name, gql_name
            ),
            "Int" => format!(
                "        if let Some(v) = self.{} {{ b.set_i32({:?}, v); }}\n",
                field_name, gql_name
            ),
            "Timestamp" | "Int8" => format!(
                "        if let Some(v) = self.{} {{ b.set_i64({:?}, v); }}\n",
                field_name, gql_name
            ),
            "BigInt" | "BigDecimal" => format!(
                "        if let Some(ref v) = self.{} {{ b.set_bigint({:?}, v); }}\n",
                field_name, gql_name
            ),
            "Bytes" | "Address" => format!(
                "        if let Some(ref v) = self.{} {{ b.set_bytes({:?}, v); }}\n",
                field_name, gql_name
            ),
            _ => format!(
                "        if let Some(ref v) = self.{} {{ b.set_string({:?}, v); }}\n",
                field_name, gql_name
            ),
        }
    } else {
        match base_scalar {
            "ID" | "String" => format!(
                "        b.set_string({:?}, &self.{});\n",
                gql_name, field_name
            ),
            "Boolean" => format!("        b.set_bool({:?}, self.{});\n", gql_name, field_name),
            "Int" => format!("        b.set_i32({:?}, self.{});\n", gql_name, field_name),
            "Timestamp" | "Int8" => {
                format!("        b.set_i64({:?}, self.{});\n", gql_name, field_name)
            }
            "BigInt" | "BigDecimal" => {
                format!("        b.set_bigint({:?}, &self.{});\n", gql_name, field_name)
            }
            "Bytes" | "Address" => {
                format!("        b.set_bytes({:?}, &self.{});\n", gql_name, field_name)
            }
            _ => format!(
                "        b.set_string({:?}, &self.{});\n",
                gql_name, field_name
            ),
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

/// Convert a GraphQL type to a Rust type string and a nullability flag.
/// Returns `(rust_type_string, is_nullable)`.
///
/// Non-nullable fields (`!`) use `T` in the struct; nullable fields use `Option<T>`.
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
        // Timeseries scalars — graph-node stores these as 64-bit integers
        "Timestamp" | "Int8" => "i64".to_string(),
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

    #[test]
    fn mutable_entity_has_remove() {
        let schema = r#"type Transfer @entity { id: ID! from: Bytes! }"#;
        let doc: Document<'_, String> = graphql_parser::parse_schema(schema).unwrap();
        if let Definition::TypeDefinition(TypeDefinition::Object(obj)) = &doc.definitions[0] {
            let code = generate_entity_struct(&obj.name, &obj.fields, false).unwrap();
            assert!(code.contains("pub fn remove(id: &str)"), "mutable entity must have remove()");
        }
    }

    #[test]
    fn immutable_entity_no_remove() {
        let schema = r#"type Price @entity(immutable: true) { id: ID! value: BigInt! }"#;
        let doc: Document<'_, String> = graphql_parser::parse_schema(schema).unwrap();
        if let Definition::TypeDefinition(TypeDefinition::Object(obj)) = &doc.definitions[0] {
            let code = generate_entity_struct(&obj.name, &obj.fields, true).unwrap();
            assert!(!code.contains("pub fn remove(id: &str)"), "immutable entity must not have remove()");
            assert!(code.contains("immutable"), "immutable doc comment missing");
        }
    }

    #[test]
    fn immutable_flag_detected_from_schema() {
        let schema = r#"type Price @entity(immutable: true) { id: ID! value: BigInt! }"#;
        let result = generate_schema_entities_from_str(schema).unwrap();
        assert!(!result.contains("pub fn remove(id: &str)"));
    }

    #[test]
    fn mutable_flag_detected_from_schema() {
        let schema = r#"type Transfer @entity { id: ID! from: Bytes! }"#;
        let result = generate_schema_entities_from_str(schema).unwrap();
        assert!(result.contains("pub fn remove(id: &str)"));
    }

    #[test]
    fn derived_from_field_excluded_from_struct() {
        let schema = r#"
            type Token @entity {
                id: ID!
                name: String!
                transfers: [Transfer!]! @derivedFrom(field: "token")
            }
        "#;
        let result = generate_schema_entities_from_str(schema).unwrap();
        // The derived field should NOT appear as a struct field or setter.
        assert!(
            !result.contains("transfers"),
            "@derivedFrom field 'transfers' should not be in generated code, got:\n{}",
            result
        );
        // The real fields should still be there.
        assert!(result.contains("pub fn set_name"));
    }

    #[test]
    fn derived_from_field_does_not_affect_setters() {
        let schema = r#"
            type Pair @entity {
                id: ID!
                token0: String!
                swaps: [Swap!]! @derivedFrom(field: "pair")
            }
        "#;
        let result = generate_schema_entities_from_str(schema).unwrap();
        assert!(!result.contains("set_swaps"), "no setter should be generated for @derivedFrom field");
        assert!(result.contains("set_token0"), "real fields should still have setters");
    }
}
