//! ABI to Rust code generation.
//!
//! Parses Ethereum contract ABIs and generates Rust structs for events,
//! complete with decoding logic using graph-as-runtime helper methods.

use alloy_json_abi::{Event, EventParam, Function, JsonAbi, Param};
use anyhow::{Context, Result};
use heck::ToUpperCamelCase;
use std::collections::HashSet;
use std::fmt::Write;
use std::path::Path;

/// Generate Rust bindings for a contract ABI.
pub fn generate_abi_bindings(abi_path: &Path, contract_name: &str) -> Result<String> {
    let abi_json = std::fs::read_to_string(abi_path)
        .with_context(|| format!("Failed to read ABI file: {}", abi_path.display()))?;

    let abi: JsonAbi = serde_json::from_str(&abi_json)
        .with_context(|| format!("Failed to parse ABI JSON: {}", abi_path.display()))?;

    let mut output = String::new();

    // File header
    writeln!(
        output,
        "//! Generated event bindings for {} contract.",
        contract_name
    )?;
    writeln!(output, "//!")?;
    writeln!(
        output,
        "//! DO NOT EDIT — regenerate with `graphite codegen`"
    )?;
    writeln!(output)?;
    writeln!(output, "#![allow(dead_code)]")?;
    writeln!(output, "#![allow(unused_imports)]")?;
    writeln!(output)?;
    writeln!(output, "extern crate alloc;")?;
    writeln!(output)?;
    writeln!(output, "use alloc::string::String;")?;
    writeln!(output, "use alloc::vec::Vec;")?;
    writeln!(output)?;

    // Generate event structs
    for event in abi.events() {
        let event_code = generate_event_struct(event, contract_name)?;
        output.push_str(&event_code);
        output.push('\n');
    }

    // Generate call structs for all ABI functions
    for function in abi.functions() {
        let call_code = generate_call_struct(function, contract_name)?;
        output.push_str(&call_code);
        output.push('\n');
    }

    Ok(output)
}

/// Generate a Rust struct for an event using raw graph-as-runtime types.
fn generate_event_struct(event: &Event, contract_name: &str) -> Result<String> {
    let mut output = String::new();
    let struct_name = format!("{}{}Event", contract_name, event.name.to_upper_camel_case());

    // Doc comment
    writeln!(output, "/// Generated from `{}` event.", event.name)?;
    writeln!(output, "pub struct {} {{", struct_name)?;

    // Event-specific parameters
    let mut seen_names = HashSet::new();
    for (i, param) in event.inputs.iter().enumerate() {
        let field_name = param_to_field_name(param, i, &mut seen_names);
        let rust_type = solidity_to_rust_type(&param.ty);
        writeln!(output, "    pub {}: {},", field_name, rust_type)?;
    }

    // Context fields from RawEthereumEvent
    writeln!(output, "    pub block_number: Vec<u8>,")?;
    writeln!(output, "    pub block_timestamp: Vec<u8>,")?;
    writeln!(output, "    pub tx_hash: [u8; 32],")?;
    writeln!(output, "    pub log_index: Vec<u8>,")?;
    writeln!(output, "    pub address: [u8; 20],")?;

    writeln!(output, "}}")?;
    writeln!(output)?;

    // Generate FromRawEvent implementation
    let raw_impl = generate_from_raw_event_impl(event, &struct_name)?;
    output.push_str(&raw_impl);

    Ok(output)
}

/// Generate `FromRawEvent` impl using `find_*` helpers.
fn generate_from_raw_event_impl(event: &Event, struct_name: &str) -> Result<String> {
    let mut output = String::new();

    writeln!(
        output,
        "impl graph_as_runtime::ethereum::FromRawEvent for {} {{",
        struct_name
    )?;
    writeln!(
        output,
        "    fn from_raw_event(raw: &graph_as_runtime::ethereum::RawEthereumEvent) -> Result<Self, &'static str> {{"
    )?;

    let mut seen_names = HashSet::new();
    for (i, param) in event.inputs.iter().enumerate() {
        let field_name = param_to_field_name(param, i, &mut seen_names);
        let param_name_str = if param.name.is_empty() {
            format!("param_{}", i)
        } else {
            param.name.clone()
        };
        let find_expr = raw_find_call(&param.ty, &param_name_str);
        writeln!(output, "        let {} = {};", field_name, find_expr)?;
    }

    writeln!(output)?;
    writeln!(output, "        Ok(Self {{")?;

    seen_names.clear();
    for (i, param) in event.inputs.iter().enumerate() {
        let field_name = param_to_field_name(param, i, &mut seen_names);
        writeln!(output, "            {},", field_name)?;
    }

    writeln!(
        output,
        "            block_number: raw.block_number.clone(),"
    )?;
    writeln!(
        output,
        "            block_timestamp: raw.block_timestamp.clone(),"
    )?;
    writeln!(output, "            tx_hash: raw.tx_hash,")?;
    writeln!(output, "            log_index: raw.log_index.clone(),")?;
    writeln!(output, "            address: raw.address,")?;

    writeln!(output, "        }})")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;

    Ok(output)
}

// ============================================================================
// Call handler struct generation
// ============================================================================

/// Generate a Rust struct for a contract function call (call handler).
///
/// Emits a `{Contract}{FunctionName}Call` struct with the function's input
/// parameters as fields, plus block/tx context fields.  Also emits a
/// `FromRawCall` implementation that decodes the inputs by name from
/// `RawEthereumCall`.
fn generate_call_struct(function: &Function, contract_name: &str) -> Result<String> {
    let mut output = String::new();
    let fn_camel = function.name.to_upper_camel_case();
    let struct_name = format!("{}{}Call", contract_name, fn_camel);

    writeln!(output, "/// Generated from `{}` function call.", function.name)?;
    writeln!(output, "pub struct {} {{", struct_name)?;

    // Input parameters
    let mut seen_names = HashSet::new();
    for (i, param) in function.inputs.iter().enumerate() {
        let field_name = call_param_to_field_name(param, i, &mut seen_names);
        let rust_type = solidity_to_rust_type(&param.ty);
        writeln!(output, "    pub {}: {},", field_name, rust_type)?;
    }

    // Context fields
    writeln!(output, "    pub block_number: Vec<u8>,")?;
    writeln!(output, "    pub block_timestamp: Vec<u8>,")?;
    writeln!(output, "    pub tx_hash: [u8; 32],")?;
    writeln!(output, "    pub address: [u8; 20],")?;
    writeln!(output, "    pub from: [u8; 20],")?;
    writeln!(output, "}}")?;
    writeln!(output)?;

    // FromRawCall impl
    let impl_code = generate_from_raw_call_impl(function, &struct_name)?;
    output.push_str(&impl_code);

    Ok(output)
}

/// Generate `FromRawCall` impl for a call struct.
fn generate_from_raw_call_impl(function: &Function, struct_name: &str) -> Result<String> {
    let mut output = String::new();

    writeln!(
        output,
        "impl graph_as_runtime::ethereum::FromRawCall for {} {{",
        struct_name
    )?;
    writeln!(
        output,
        "    fn from_raw_call(raw: &graph_as_runtime::ethereum::RawEthereumCall) -> Result<Self, &'static str> {{"
    )?;

    let mut seen_names = HashSet::new();
    for (i, param) in function.inputs.iter().enumerate() {
        let field_name = call_param_to_field_name(param, i, &mut seen_names);
        let param_name_str = if param.name.is_empty() {
            format!("param_{}", i)
        } else {
            param.name.clone()
        };
        // Reuse raw_find_call logic but against raw.inputs via find_input_*
        let find_expr = raw_call_input_expr(&param.ty, &param_name_str);
        writeln!(output, "        let {} = {};", field_name, find_expr)?;
    }

    writeln!(output)?;
    writeln!(output, "        Ok(Self {{")?;

    seen_names.clear();
    for (i, param) in function.inputs.iter().enumerate() {
        let field_name = call_param_to_field_name(param, i, &mut seen_names);
        writeln!(output, "            {},", field_name)?;
    }

    writeln!(output, "            block_number: raw.block_number.clone(),")?;
    writeln!(output, "            block_timestamp: raw.block_timestamp.clone(),")?;
    writeln!(output, "            tx_hash: raw.tx_hash,")?;
    writeln!(output, "            address: raw.address,")?;
    writeln!(output, "            from: raw.from,")?;
    writeln!(output, "        }})")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;

    Ok(output)
}

/// Return the RHS expression for decoding a call input parameter from `RawEthereumCall`.
fn raw_call_input_expr(sol_type: &str, param_name: &str) -> String {
    // Array types
    if let Some(inner) = sol_type.strip_suffix("[]") {
        let extractor = array_element_extractor(inner);
        return format!(
            "raw.inputs.iter().find(|p| p.name == {:?}).and_then(|p| p.value.as_array()).map(|arr| arr.iter().filter_map(|v| {}).collect::<alloc::vec::Vec<_>>()).ok_or(\"input param not found\")?",
            param_name, extractor
        );
    }
    if sol_type.ends_with(']') {
        if let Some(bracket) = sol_type.rfind('[') {
            let inner = &sol_type[..bracket];
            let extractor = array_element_extractor(inner);
            return format!(
                "raw.inputs.iter().find(|p| p.name == {:?}).and_then(|p| p.value.as_array()).map(|arr| arr.iter().filter_map(|v| {}).collect::<alloc::vec::Vec<_>>()).ok_or(\"input param not found\")?",
                param_name, extractor
            );
        }
    }
    if sol_type.starts_with('(') || sol_type == "tuple" || sol_type.starts_with("tuple(") {
        return format!(
            "raw.inputs.iter().find(|p| p.name == {:?}).and_then(|p| p.value.as_tuple()).map(|t| t.to_vec()).ok_or(\"input param not found\")?",
            param_name
        );
    }
    // Scalar
    let method = call_input_find_method(sol_type);
    format!("raw.{}({:?}).map_err(|_| \"input param not found\")?", method, param_name)
}

/// Map a Solidity scalar type to the appropriate `find_input_*` method on `RawEthereumCall`.
fn call_input_find_method(sol_type: &str) -> &'static str {
    match sol_type {
        "address" => "find_input_address",
        "bool" => "find_input_bool",
        _ if sol_type.starts_with("uint") => "find_input_uint",
        _ => "find_input_bytes",
    }
}

/// Convert a function `Param` to a valid Rust field name, handling duplicates.
fn call_param_to_field_name(param: &Param, index: usize, seen: &mut HashSet<String>) -> String {
    let base_name = if param.name.is_empty() {
        format!("param_{}", index)
    } else {
        to_snake_case(&param.name)
    };
    let name = match base_name.as_str() {
        "type" => "type_".to_string(),
        "ref" => "ref_".to_string(),
        "self" => "self_".to_string(),
        "mod" => "mod_".to_string(),
        "fn" => "fn_".to_string(),
        other => other.to_string(),
    };
    if seen.contains(&name) {
        let mut counter = 2;
        loop {
            let candidate = format!("{}_{}", name, counter);
            if !seen.contains(&candidate) {
                seen.insert(candidate.clone());
                return candidate;
            }
            counter += 1;
        }
    }
    seen.insert(name.clone());
    name
}

/// Return the complete RHS expression for decoding a parameter.
///
/// For scalar types this is `raw.find_*(name)?`.
/// For array/fixed-array types this maps each element through the appropriate
/// accessor and collects into a `Vec`.
/// For tuple types this clones the decoded `Vec<EthereumValue>`.
fn raw_find_call(sol_type: &str, param_name: &str) -> String {
    // Dynamic array: `T[]`
    if let Some(inner) = sol_type.strip_suffix("[]") {
        let extractor = array_element_extractor(inner);
        return format!(
            "raw.find_array({:?}).map(|arr| arr.iter().filter_map(|v| {}).collect::<alloc::vec::Vec<_>>())?",
            param_name, extractor
        );
    }

    // Fixed-size array: `T[N]`
    if sol_type.ends_with(']') {
        if let Some(bracket) = sol_type.rfind('[') {
            let inner = &sol_type[..bracket];
            let extractor = array_element_extractor(inner);
            return format!(
                "raw.find_array({:?}).map(|arr| arr.iter().filter_map(|v| {}).collect::<alloc::vec::Vec<_>>())?",
                param_name, extractor
            );
        }
    }

    // Tuple: `(T1,T2,...)` or keyword `tuple`
    if sol_type.starts_with('(') || sol_type == "tuple" || sol_type.starts_with("tuple(") {
        return format!("raw.find_tuple({:?}).map(|t| t.to_vec())?", param_name);
    }

    // Scalar types
    let method = solidity_find_method(sol_type);
    format!("raw.{}({:?})?", method, param_name)
}

/// Return the `EthereumValue` accessor expression for an array element type.
fn array_element_extractor(inner_type: &str) -> &'static str {
    match inner_type {
        "address" => "v.as_address()",
        "bool" => "v.as_bool()",
        "string" => "v.as_string().map(|s| s.to_string())",
        s if s.starts_with("uint") => "v.as_uint()",
        s if s.starts_with("int") => "v.as_int()",
        _ => "v.as_bytes()",
    }
}

/// Map a scalar Solidity type to the appropriate `find_*` method on `RawEthereumEvent`.
fn solidity_find_method(sol_type: &str) -> &'static str {
    match sol_type {
        "address" => "find_address",
        "bool" => "find_bool",
        "string" => "find_string",
        s if s.starts_with("uint") => "find_uint",
        s if s.starts_with("int") => "find_int",
        _ => "find_bytes",
    }
}

/// Convert a Solidity type to its raw Rust equivalent (no graphite types).
fn solidity_to_rust_type(sol_type: &str) -> String {
    // Handle arrays
    if sol_type.ends_with("[]") {
        let inner = &sol_type[..sol_type.len() - 2];
        return format!("Vec<{}>", solidity_to_rust_type(inner));
    }

    // Handle fixed-size arrays
    if sol_type.ends_with(']') {
        if let Some(bracket_pos) = sol_type.rfind('[') {
            let inner = &sol_type[..bracket_pos];
            let size = &sol_type[bracket_pos + 1..sol_type.len() - 1];
            return format!("[{}; {}]", solidity_to_rust_type(inner), size);
        }
    }

    // Tuple: `(T1,T2,...)` or keyword `tuple`
    if sol_type.starts_with('(') || sol_type == "tuple" || sol_type.starts_with("tuple(") {
        return "alloc::vec::Vec<graph_as_runtime::ethereum::EthereumValue>".to_string();
    }

    match sol_type {
        "address" => "[u8; 20]".to_string(),
        "bool" => "bool".to_string(),
        "string" => "String".to_string(),
        "bytes" => "Vec<u8>".to_string(),
        s if s.starts_with("bytes") => {
            // bytesN → parse N
            if let Ok(n) = s[5..].parse::<usize>() {
                format!("[u8; {}]", n)
            } else {
                "Vec<u8>".to_string()
            }
        }
        s if s.starts_with("uint") || s.starts_with("int") => "Vec<u8>".to_string(),
        _ => "Vec<u8>".to_string(),
    }
}

/// Convert an event parameter to a valid Rust field name.
fn param_to_field_name(param: &EventParam, index: usize, seen: &mut HashSet<String>) -> String {
    let base_name = if param.name.is_empty() {
        format!("param_{}", index)
    } else {
        // Convert to snake_case manually: insert _ before uppercase letters
        to_snake_case(&param.name)
    };

    // Handle Rust reserved keywords
    let name = match base_name.as_str() {
        "type" => "type_".to_string(),
        "ref" => "ref_".to_string(),
        "self" => "self_".to_string(),
        "mod" => "mod_".to_string(),
        "fn" => "fn_".to_string(),
        "let" => "let_".to_string(),
        "mut" => "mut_".to_string(),
        "pub" => "pub_".to_string(),
        "use" => "use_".to_string(),
        "impl" => "impl_".to_string(),
        other => other.to_string(),
    };

    // Handle duplicate names
    if seen.contains(&name) {
        let mut counter = 2;
        loop {
            let candidate = format!("{}_{}", name, counter);
            if !seen.contains(&candidate) {
                seen.insert(candidate.clone());
                return candidate;
            }
            counter += 1;
        }
    }

    seen.insert(name.clone());
    name
}

/// Simple snake_case conversion.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solidity_type_mapping() {
        assert_eq!(solidity_to_rust_type("address"), "[u8; 20]");
        assert_eq!(solidity_to_rust_type("uint256"), "Vec<u8>");
        assert_eq!(solidity_to_rust_type("uint8"), "Vec<u8>");
        assert_eq!(solidity_to_rust_type("bytes32"), "[u8; 32]");
        assert_eq!(solidity_to_rust_type("string"), "String");
        assert_eq!(solidity_to_rust_type("bool"), "bool");
        assert_eq!(solidity_to_rust_type("bytes"), "Vec<u8>");
    }

    #[test]
    fn array_type_mapping() {
        assert_eq!(solidity_to_rust_type("uint256[]"), "Vec<Vec<u8>>");
        assert_eq!(solidity_to_rust_type("address[]"), "Vec<[u8; 20]>");
        assert_eq!(solidity_to_rust_type("bool[]"), "Vec<bool>");
        assert_eq!(solidity_to_rust_type("string[]"), "Vec<String>");
        assert_eq!(solidity_to_rust_type("uint256[3]"), "[Vec<u8>; 3]");
    }

    #[test]
    fn tuple_type_mapping() {
        assert!(solidity_to_rust_type("(uint256,address)").contains("EthereumValue"));
        assert!(solidity_to_rust_type("tuple").contains("EthereumValue"));
    }

    #[test]
    fn array_find_expression() {
        let expr = raw_find_call("uint256[]", "amounts");
        assert!(expr.contains("find_array"), "should use find_array");
        assert!(expr.contains("as_uint"), "should map elements with as_uint");
    }

    #[test]
    fn tuple_find_expression() {
        let expr = raw_find_call("(uint256,address)", "params");
        assert!(expr.contains("find_tuple"), "should use find_tuple");
    }

    #[test]
    fn scalar_find_expression() {
        let expr = raw_find_call("uint256", "value");
        assert_eq!(expr, r#"raw.find_uint("value")?"#);
        let expr = raw_find_call("address", "from");
        assert_eq!(expr, r#"raw.find_address("from")?"#);
    }
}
