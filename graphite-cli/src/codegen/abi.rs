//! ABI to Rust code generation.
//!
//! Parses Ethereum contract ABIs and generates Rust structs for events,
//! complete with decoding logic using graph-as-runtime helper methods.

use alloy_json_abi::{Event, EventParam, JsonAbi};
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
        let find_call = raw_find_call(&param.ty, &param_name_str);
        writeln!(output, "        let {} = raw.{}?;", field_name, find_call)?;
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

/// Return the `find_*` call expression for a given Solidity type.
fn raw_find_call(sol_type: &str, param_name: &str) -> String {
    let method = solidity_find_method(sol_type);
    format!("{}({:?})", method, param_name)
}

/// Map a Solidity type to the appropriate `find_*` method on `RawEthereumEvent`.
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
}
