//! ABI to Rust code generation.
//!
//! Parses Ethereum contract ABIs and generates Rust structs for events,
//! complete with decoding logic.

use alloy_json_abi::{Event, EventParam, JsonAbi};
use anyhow::{Context, Result};
use heck::{ToSnakeCase, ToUpperCamelCase};
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
    writeln!(output, "//! Generated bindings for {} contract.", contract_name)?;
    writeln!(output, "//!")?;
    writeln!(output, "//! DO NOT EDIT — regenerate with `graphite codegen`")?;
    writeln!(output)?;
    writeln!(output, "#![allow(dead_code)]")?;
    writeln!(output, "#![allow(clippy::too_many_arguments)]")?;
    writeln!(output)?;
    writeln!(output, "use graphite::prelude::*;")?;
    writeln!(output)?;

    // Track imports we need
    let mut needs_log = false;

    // Generate event structs
    for event in abi.events() {
        let event_code = generate_event_struct(event, contract_name)?;
        output.push_str(&event_code);
        output.push('\n');
        needs_log = true;
    }

    // Generate the event enum for dispatch
    if needs_log {
        let enum_code = generate_event_enum(&abi, contract_name)?;
        output.push_str(&enum_code);
    }

    Ok(output)
}

/// Generate a Rust struct for an event.
fn generate_event_struct(event: &Event, contract_name: &str) -> Result<String> {
    let mut output = String::new();
    let struct_name = format!("{}{}Event", contract_name, event.name.to_upper_camel_case());

    // Doc comment
    writeln!(output, "/// Event: `{}`", event.signature())?;
    writeln!(output, "#[derive(Debug, Clone, PartialEq)]")?;
    writeln!(output, "pub struct {} {{", struct_name)?;

    // Common event metadata fields
    writeln!(output, "    /// Transaction hash")?;
    writeln!(output, "    pub tx_hash: B256,")?;
    writeln!(output, "    /// Log index within the transaction")?;
    writeln!(output, "    pub log_index: BigInt,")?;
    writeln!(output, "    /// Block number")?;
    writeln!(output, "    pub block_number: BigInt,")?;
    writeln!(output, "    /// Block timestamp")?;
    writeln!(output, "    pub block_timestamp: BigInt,")?;
    writeln!(output, "    /// Contract address that emitted the event")?;
    writeln!(output, "    pub address: Address,")?;

    // Event-specific parameters
    let mut seen_names = HashSet::new();
    for (i, param) in event.inputs.iter().enumerate() {
        let field_name = param_to_field_name(param, i, &mut seen_names);
        let rust_type = solidity_to_rust_type(&param.ty);
        let indexed = if param.indexed { " (indexed)" } else { "" };

        writeln!(output, "    /// `{}{}`", param.ty, indexed)?;
        writeln!(output, "    pub {}: {},", field_name, rust_type)?;
    }

    writeln!(output, "}}")?;
    writeln!(output)?;

    // Generate impl block with helper methods
    writeln!(output, "impl {} {{", struct_name)?;

    // Unique ID generation (tx_hash + log_index)
    writeln!(output, "    /// Generate a unique ID for this event.")?;
    writeln!(output, "    pub fn id(&self) -> String {{")?;
    writeln!(
        output,
        "        format!(\"{{:?}}-{{}}\", self.tx_hash, self.log_index)"
    )?;
    writeln!(output, "    }}")?;

    // Event topic (selector)
    let selector = event.selector();
    writeln!(output)?;
    writeln!(output, "    /// The event topic (keccak256 of signature).")?;
    writeln!(output, "    pub const SELECTOR: [u8; 32] = {:?};", selector.0)?;

    writeln!(output, "}}")?;

    Ok(output)
}

/// Generate an enum that encompasses all events from a contract.
fn generate_event_enum(abi: &JsonAbi, contract_name: &str) -> Result<String> {
    let mut output = String::new();
    let enum_name = format!("{}Event", contract_name);

    writeln!(output, "/// All events emitted by the {} contract.", contract_name)?;
    writeln!(output, "#[derive(Debug, Clone, PartialEq)]")?;
    writeln!(output, "pub enum {} {{", enum_name)?;

    for event in abi.events() {
        let variant_name = event.name.to_upper_camel_case();
        let struct_name = format!("{}{}Event", contract_name, variant_name);
        writeln!(output, "    {}({}),", variant_name, struct_name)?;
    }

    writeln!(output, "}}")?;

    Ok(output)
}

/// Convert a Solidity type to its Rust equivalent.
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

    // Handle tuples (simplified — doesn't parse nested structure)
    if sol_type.starts_with('(') && sol_type.ends_with(')') {
        // For now, just use Bytes for complex tuples
        return "Bytes".to_string();
    }

    match sol_type {
        // Addresses
        "address" => "Address".to_string(),

        // Booleans
        "bool" => "bool".to_string(),

        // Strings
        "string" => "String".to_string(),

        // Bytes
        "bytes" => "Bytes".to_string(),
        "bytes1" => "[u8; 1]".to_string(),
        "bytes2" => "[u8; 2]".to_string(),
        "bytes3" => "[u8; 3]".to_string(),
        "bytes4" => "[u8; 4]".to_string(),
        "bytes8" => "[u8; 8]".to_string(),
        "bytes16" => "[u8; 16]".to_string(),
        "bytes20" => "[u8; 20]".to_string(),
        "bytes32" => "B256".to_string(),

        // Signed integers (all map to BigInt for simplicity and safety)
        s if s.starts_with("int") => "BigInt".to_string(),

        // Unsigned integers
        "uint8" | "uint16" | "uint32" => "i32".to_string(), // Fits in i32
        "uint64" => "i64".to_string(),
        s if s.starts_with("uint") => "BigInt".to_string(), // uint128, uint256, etc.

        // Unknown type — fall back to Bytes
        _ => "Bytes".to_string(),
    }
}

/// Convert an event parameter to a valid Rust field name.
fn param_to_field_name(param: &EventParam, index: usize, seen: &mut HashSet<String>) -> String {
    let base_name = if param.name.is_empty() {
        format!("param_{}", index)
    } else {
        param.name.to_snake_case()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solidity_type_mapping() {
        assert_eq!(solidity_to_rust_type("address"), "Address");
        assert_eq!(solidity_to_rust_type("uint256"), "BigInt");
        assert_eq!(solidity_to_rust_type("uint8"), "i32");
        assert_eq!(solidity_to_rust_type("bytes32"), "B256");
        assert_eq!(solidity_to_rust_type("string"), "String");
        assert_eq!(solidity_to_rust_type("bool"), "bool");
        assert_eq!(solidity_to_rust_type("address[]"), "Vec<Address>");
        assert_eq!(solidity_to_rust_type("uint256[3]"), "[BigInt; 3]");
    }

    #[test]
    fn param_name_handling() {
        let mut seen = HashSet::new();

        let param1 = EventParam {
            name: "from".to_string(),
            ty: "address".to_string(),
            indexed: true,
            components: vec![],
            internal_type: None,
        };
        assert_eq!(param_to_field_name(&param1, 0, &mut seen), "from");

        // Duplicate name gets suffixed
        let param2 = EventParam {
            name: "from".to_string(),
            ty: "address".to_string(),
            indexed: false,
            components: vec![],
            internal_type: None,
        };
        assert_eq!(param_to_field_name(&param2, 1, &mut seen), "from_2");

        // Reserved keyword
        let mut seen2 = HashSet::new();
        let param3 = EventParam {
            name: "type".to_string(),
            ty: "uint8".to_string(),
            indexed: false,
            components: vec![],
            internal_type: None,
        };
        assert_eq!(param_to_field_name(&param3, 0, &mut seen2), "type_");
    }
}
