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

    // Pre-scan to decide which imports are needed
    let needs_vec = abi
        .events()
        .any(|e| e.inputs.iter().any(|p| p.ty.contains('[')));

    let mut output = String::new();

    // File header
    writeln!(output, "//! Generated bindings for {} contract.", contract_name)?;
    writeln!(output, "//!")?;
    writeln!(output, "//! DO NOT EDIT — regenerate with `graphite codegen`")?;
    writeln!(output)?;
    writeln!(output, "#![allow(dead_code)]")?;
    writeln!(output, "#![allow(clippy::too_many_arguments)]")?;
    writeln!(output)?;
    writeln!(output, "extern crate alloc;")?;
    writeln!(output)?;
    writeln!(output, "use alloc::format;")?;
    writeln!(output, "use alloc::string::String;")?;
    if needs_vec {
        writeln!(output, "use alloc::vec::Vec;")?;
    }
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

    // from_raw_log constructor
    writeln!(output)?;
    writeln!(output, "    /// Decode from a raw log.")?;
    writeln!(output, "    pub fn from_raw_log(log: &RawLog) -> Result<Self, DecodeError> {{")?;
    writeln!(output, "        <Self as EventDecode>::decode(&log.topics, log.data.as_slice())")?;
    writeln!(output, "            .map(|mut e| {{")?;
    writeln!(output, "                e.tx_hash = log.tx_hash;")?;
    writeln!(output, "                e.log_index = BigInt::from(log.log_index);")?;
    writeln!(output, "                e.block_number = BigInt::from(log.block_number);")?;
    writeln!(output, "                e.block_timestamp = BigInt::from(log.block_timestamp);")?;
    writeln!(output, "                e.address = log.address;")?;
    writeln!(output, "                e")?;
    writeln!(output, "            }})")?;
    writeln!(output, "    }}")?;

    writeln!(output, "}}")?;
    writeln!(output)?;

    // Generate EventDecode implementation
    let decode_impl = generate_event_decode_impl(event, &struct_name)?;
    output.push_str(&decode_impl);

    // Generate FromWasmBytes implementation for WASM deserialization
    let wasm_impl = generate_from_wasm_bytes_impl(event, &struct_name)?;
    output.push_str(&wasm_impl);

    // Generate FromRawEvent implementation for AS-ABI event reading
    let raw_impl = generate_from_raw_event_impl(event, &struct_name)?;
    output.push_str(&raw_impl);

    Ok(output)
}

/// Generate EventDecode trait implementation for an event.
fn generate_event_decode_impl(event: &Event, struct_name: &str) -> Result<String> {
    let mut output = String::new();
    let selector = event.selector();

    // Separate indexed params (non-indexed are decoded from data)
    let indexed_params: Vec<_> = event.inputs.iter().filter(|p| p.indexed).collect();

    // Expected number of topics: 1 (selector) + indexed params
    let expected_topics = 1 + indexed_params.len();

    // If all params are indexed, the `data` argument will be unused
    let has_non_indexed = event.inputs.iter().any(|p| !p.indexed);
    let data_param = if has_non_indexed { "data" } else { "_data" };

    writeln!(output, "impl EventDecode for {} {{", struct_name)?;
    writeln!(output, "    const SELECTOR: [u8; 32] = {:?};", selector.0)?;
    writeln!(output)?;
    writeln!(output, "    fn decode(topics: &[B256], {}: &[u8]) -> Result<Self, DecodeError> {{", data_param)?;

    // Check selector
    writeln!(output, "        // Verify selector")?;
    writeln!(output, "        if topics.is_empty() || topics[0].0 != Self::SELECTOR {{")?;
    writeln!(output, "            return Err(DecodeError::SelectorMismatch {{")?;
    writeln!(output, "                expected: Self::SELECTOR,")?;
    writeln!(output, "                got: topics.first().map(|t| t.0).unwrap_or([0; 32]),")?;
    writeln!(output, "            }});")?;
    writeln!(output, "        }}")?;
    writeln!(output)?;

    // Check topic count
    writeln!(output, "        // Verify topic count")?;
    writeln!(output, "        if topics.len() < {} {{", expected_topics)?;
    writeln!(output, "            return Err(DecodeError::NotEnoughTopics {{")?;
    writeln!(output, "                expected: {},", expected_topics)?;
    writeln!(output, "                got: topics.len(),")?;
    writeln!(output, "            }});")?;
    writeln!(output, "        }}")?;
    writeln!(output)?;

    // Decode indexed params from topics
    let mut seen_names = HashSet::new();
    let mut topic_idx = 1; // Start after selector
    for (i, param) in event.inputs.iter().enumerate() {
        if !param.indexed {
            continue;
        }
        let field_name = param_to_field_name(param, i, &mut seen_names);
        let decode_expr = topic_decode_expr(&param.ty, topic_idx);
        writeln!(output, "        let {} = {};", field_name, decode_expr)?;
        topic_idx += 1;
    }

    // Decode non-indexed params from data
    let mut data_offset = 0;
    seen_names.clear();
    for (i, param) in event.inputs.iter().enumerate() {
        if param.indexed {
            continue;
        }
        let field_name = param_to_field_name(param, i, &mut seen_names);
        let (decode_expr, size) = data_decode_expr(&param.ty, data_offset);
        writeln!(output, "        let {} = {}?;", field_name, decode_expr)?;
        data_offset += size;
    }

    writeln!(output)?;
    writeln!(output, "        Ok(Self {{")?;
    writeln!(output, "            tx_hash: B256::default(),")?;
    writeln!(output, "            log_index: BigInt::zero(),")?;
    writeln!(output, "            block_number: BigInt::zero(),")?;
    writeln!(output, "            block_timestamp: BigInt::zero(),")?;
    writeln!(output, "            address: Address::ZERO,")?;

    // Output all fields
    seen_names.clear();
    for (i, param) in event.inputs.iter().enumerate() {
        let field_name = param_to_field_name(param, i, &mut seen_names);
        writeln!(output, "            {},", field_name)?;
    }

    writeln!(output, "        }})")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;

    Ok(output)
}

/// Generate decode expression for an indexed parameter from a topic.
fn topic_decode_expr(sol_type: &str, topic_idx: usize) -> String {
    match sol_type {
        "address" => format!(
            "graphite::decode::decode_address_from_topic(&topics[{}])",
            topic_idx
        ),
        "bool" => format!(
            "graphite::decode::decode_bool_from_topic(&topics[{}])",
            topic_idx
        ),
        "bytes32" => format!(
            "graphite::decode::decode_bytes32_from_topic(&topics[{}])",
            topic_idx
        ),
        _ if sol_type.starts_with("uint") || sol_type.starts_with("int") => format!(
            "graphite::decode::decode_uint256_from_topic(&topics[{}])",
            topic_idx
        ),
        // Default: treat as bytes32
        _ => format!("topics[{}]", topic_idx),
    }
}

/// Generate decode expression for a non-indexed parameter from data.
/// Returns (expression, size in bytes consumed).
fn data_decode_expr(sol_type: &str, offset: usize) -> (String, usize) {
    match sol_type {
        "address" => (
            format!("graphite::decode::decode_address(data, {})", offset),
            32,
        ),
        "bool" => (
            format!("graphite::decode::decode_bool(data, {})", offset),
            32,
        ),
        "bytes32" => (
            format!("graphite::decode::decode_bytes32(data, {})", offset),
            32,
        ),
        "string" => (
            format!("graphite::decode::decode_string(data, {})", offset),
            32, // Pointer takes 32 bytes
        ),
        "bytes" => (
            format!("graphite::decode::decode_bytes(data, {})", offset),
            32, // Pointer takes 32 bytes
        ),
        _ if sol_type.starts_with("uint") || sol_type.starts_with("int") => (
            format!("graphite::decode::decode_uint256(data, {})", offset),
            32,
        ),
        // Default fallback
        _ => (
            format!("graphite::decode::decode_bytes32(data, {})", offset),
            32,
        ),
    }
}

/// Generate FromWasmBytes implementation for deserializing from graph-node's format.
///
/// Graph-node serializes events using RustLogTrigger format (compact binary).
/// We parse it as RawLog first, then use the existing from_raw_log method.
fn generate_from_wasm_bytes_impl(_event: &Event, struct_name: &str) -> Result<String> {
    let mut output = String::new();

    writeln!(output, "impl FromWasmBytes for {} {{", struct_name)?;
    writeln!(
        output,
        "    fn from_wasm_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {{"
    )?;
    writeln!(output, "        // Parse as RawLog first (graph-node sends RustLogTrigger format)")?;
    writeln!(output, "        let raw_log = RawLog::from_wasm_bytes(bytes)?;")?;
    writeln!(output, "        Self::from_raw_log(&raw_log)")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;

    Ok(output)
}

/// Generate a `FromRawEvent` implementation for AS-ABI event reading.
///
/// This is used by the `#[handler]` macro's WASM entry point to decode
/// event parameters from the `RawEthereumEvent` produced by
/// `graph_as_runtime::ethereum::read_ethereum_event`.
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

    // Generate param lookups for event-specific inputs
    let mut seen_names = HashSet::new();
    for (i, param) in event.inputs.iter().enumerate() {
        let field_name = param_to_field_name(param, i, &mut seen_names);
        let param_name_str = if param.name.is_empty() {
            format!("param_{}", i)
        } else {
            param.name.clone()
        };
        let extract = raw_event_extract_expr(&param.ty, &param_name_str, &field_name);
        writeln!(output, "        {}", extract)?;
    }

    writeln!(output)?;
    writeln!(output, "        Ok(Self {{")?;
    // Block/tx fields come from the raw event metadata
    writeln!(output, "            tx_hash:         B256(raw.tx_hash),")?;
    writeln!(
        output,
        "            log_index:       graphite::primitives::BigInt::from_signed_bytes_le(&raw.log_index),"
    )?;
    writeln!(
        output,
        "            block_number:    graphite::primitives::BigInt::from_signed_bytes_le(&raw.block_number),"
    )?;
    writeln!(
        output,
        "            block_timestamp: graphite::primitives::BigInt::from_signed_bytes_le(&raw.block_timestamp),"
    )?;
    writeln!(
        output,
        "            address:         Address::from(raw.address),"
    )?;
    // Event-specific params
    seen_names.clear();
    for (i, param) in event.inputs.iter().enumerate() {
        let field_name = param_to_field_name(param, i, &mut seen_names);
        writeln!(output, "            {},", field_name)?;
    }
    writeln!(output, "        }})")?;
    writeln!(output, "    }}")?;
    writeln!(output, "}}")?;
    writeln!(output)?;

    Ok(output)
}

/// Generate the extraction expression for a single ABI param from `raw.params`.
///
/// Returns a `let field_name = ...;` statement string.
fn raw_event_extract_expr(sol_type: &str, param_name: &str, field_name: &str) -> String {
    let find = format!(
        "raw.params.iter().find(|p| p.name == {:?}).ok_or(\"missing param: {}\")?",
        param_name, param_name
    );

    let variant_match = match sol_type {
        "address" => format!(
            concat!(
                "let {field} = {{\n",
                "            let _p = {find};\n",
                "            match &_p.value {{\n",
                "                graph_as_runtime::ethereum::EthereumValue::Address(a) => Address::from(*a),\n",
                "                _ => return Err(\"wrong type for {param}\"),\n",
                "            }}\n",
                "        }};"
            ),
            field = field_name,
            find = find,
            param = param_name,
        ),
        "bool" => format!(
            concat!(
                "let {field} = {{\n",
                "            let _p = {find};\n",
                "            match &_p.value {{\n",
                "                graph_as_runtime::ethereum::EthereumValue::Bool(b) => *b,\n",
                "                _ => return Err(\"wrong type for {param}\"),\n",
                "            }}\n",
                "        }};"
            ),
            field = field_name,
            find = find,
            param = param_name,
        ),
        "string" => format!(
            concat!(
                "let {field} = {{\n",
                "            let _p = {find};\n",
                "            match &_p.value {{\n",
                "                graph_as_runtime::ethereum::EthereumValue::String(s) => s.clone(),\n",
                "                _ => return Err(\"wrong type for {param}\"),\n",
                "            }}\n",
                "        }};"
            ),
            field = field_name,
            find = find,
            param = param_name,
        ),
        "bytes32" => format!(
            concat!(
                "let {field} = {{\n",
                "            let _p = {find};\n",
                "            match &_p.value {{\n",
                "                graph_as_runtime::ethereum::EthereumValue::FixedBytes(b) => {{\n",
                "                    let mut arr = [0u8; 32];\n",
                "                    let n = b.len().min(32);\n",
                "                    arr[..n].copy_from_slice(&b[..n]);\n",
                "                    B256(arr)\n",
                "                }},\n",
                "                _ => return Err(\"wrong type for {param}\"),\n",
                "            }}\n",
                "        }};"
            ),
            field = field_name,
            find = find,
            param = param_name,
        ),
        s if s.starts_with("bytes") && s != "bytes" => {
            // Fixed bytesN (N < 32)
            format!(
                concat!(
                    "let {field} = {{\n",
                    "            let _p = {find};\n",
                    "            match &_p.value {{\n",
                    "                graph_as_runtime::ethereum::EthereumValue::FixedBytes(b) => b.clone(),\n",
                    "                _ => return Err(\"wrong type for {param}\"),\n",
                    "            }}\n",
                    "        }};"
                ),
                field = field_name,
                find = find,
                param = param_name,
            )
        }
        "bytes" => format!(
            concat!(
                "let {field} = {{\n",
                "            let _p = {find};\n",
                "            match &_p.value {{\n",
                "                graph_as_runtime::ethereum::EthereumValue::Bytes(b) => graphite::primitives::Bytes::from_slice(b),\n",
                "                _ => return Err(\"wrong type for {param}\"),\n",
                "            }}\n",
                "        }};"
            ),
            field = field_name,
            find = find,
            param = param_name,
        ),
        s if s.starts_with("uint") || s.starts_with("int") => format!(
            concat!(
                "let {field} = {{\n",
                "            let _p = {find};\n",
                "            match &_p.value {{\n",
                "                graph_as_runtime::ethereum::EthereumValue::Uint(b) =>\n",
                "                    graphite::primitives::BigInt::from_signed_bytes_le(b),\n",
                "                graph_as_runtime::ethereum::EthereumValue::Int(b) =>\n",
                "                    graphite::primitives::BigInt::from_signed_bytes_le(b),\n",
                "                _ => return Err(\"wrong type for {param}\"),\n",
                "            }}\n",
                "        }};"
            ),
            field = field_name,
            find = find,
            param = param_name,
        ),
        _ => format!(
            concat!(
                "let {field} = {{\n",
                "            let _p = {find};\n",
                "            match &_p.value {{\n",
                "                graph_as_runtime::ethereum::EthereumValue::Bytes(b) => graphite::primitives::Bytes::from_slice(b),\n",
                "                _ => return Err(\"wrong type for {param}\"),\n",
                "            }}\n",
                "        }};"
            ),
            field = field_name,
            find = find,
            param = param_name,
        ),
    };

    variant_match
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
