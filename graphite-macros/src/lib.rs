//! Proc macros for the Graphite subgraph SDK.
//!
//! Provides `#[derive(Entity)]` and `#[handler]` macros for ergonomic
//! subgraph development.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemFn};

/// Derive macro for entity types.
///
/// Generates `Store` trait implementation with `load()`, `save()`, and `remove()` methods.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Entity)]
/// pub struct Transfer {
///     #[id]
///     id: String,
///     from: Address,
///     to: Address,
///     value: BigInt,
/// }
/// ```
///
/// The struct must have exactly one field marked with `#[id]`.
#[proc_macro_derive(Entity, attributes(id, graphite))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let entity_type = name.to_string();

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => panic!("Entity derive only supports structs with named fields"),
        },
        _ => panic!("Entity derive only supports structs"),
    };

    // Find the #[id] field
    let id_field = fields
        .iter()
        .find(|f| f.attrs.iter().any(|a| a.path().is_ident("id")))
        .expect("Entity must have exactly one field marked with #[id]");
    let id_field_name = id_field.ident.as_ref().unwrap();

    // Generate field setters for to_entity
    let field_setters = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_name_str = to_camel_case(&field_name.to_string());
        quote! {
            entity.set(#field_name_str, self.#field_name.clone());
        }
    });

    // Generate field getters for from_entity
    let field_getters = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        let field_name_str = to_camel_case(&field_name.to_string());
        let field_type = &f.ty;
        quote! {
            #field_name: entity
                .get(#field_name_str)
                .and_then(|v| <#field_type as graphite::store::FromValue>::from_value(v.clone()))
                .ok_or_else(|| graphite::store::EntityError::MissingField(#field_name_str.into()))?
        }
    });

    // Generate Default-like field initializers for new()
    let field_defaults = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();
        if f.attrs.iter().any(|a| a.path().is_ident("id")) {
            quote! { #field_name: id.into() }
        } else {
            quote! { #field_name: Default::default() }
        }
    });

    let expanded = quote! {
        impl #name {
            /// Create a new instance with the given ID and default field values.
            pub fn new(id: impl Into<String>) -> Self {
                Self {
                    #(#field_defaults),*
                }
            }

            /// Load an entity from the store.
            pub fn load<H: graphite::host::HostFunctions>(host: &H, id: &str) -> Option<Self> {
                host.store_get(#entity_type, id)
                    .and_then(|e| Self::from_entity(e).ok())
            }

            /// Save this entity to the store.
            pub fn save<H: graphite::host::HostFunctions>(&self, host: &mut H) {
                host.store_set(#entity_type, &self.id(), self.to_entity());
            }

            /// Remove this entity from the store.
            pub fn remove<H: graphite::host::HostFunctions>(host: &mut H, id: &str) {
                host.store_remove(#entity_type, id);
            }
        }

        impl graphite::store::Store for #name {
            const ENTITY_TYPE: &'static str = #entity_type;

            fn id(&self) -> &str {
                &self.#id_field_name
            }

            fn to_entity(&self) -> graphite::store::Entity {
                let mut entity = graphite::store::Entity::new();
                #(#field_setters)*
                entity
            }

            fn from_entity(entity: graphite::store::Entity) -> Result<Self, graphite::store::EntityError> {
                Ok(Self {
                    #(#field_getters),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

/// Attribute macro for handler functions.
///
/// Generates the `extern "C"` wrapper that graph-node calls, handling
/// event deserialization and memory management.
///
/// # Example
///
/// ```rust,ignore
/// #[handler]
/// pub fn handle_transfer(event: TransferEvent) {
///     // Handler logic here
/// }
/// ```
#[proc_macro_attribute]
pub fn handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_body = &input.block;
    let fn_inputs = &input.sig.inputs;
    let fn_vis = &input.vis;

    // Extract the event parameter type
    let event_param = fn_inputs.first().expect("Handler must have an event parameter");
    let (param_name, param_type) = match event_param {
        syn::FnArg::Typed(pat_type) => {
            let name = match &*pat_type.pat {
                syn::Pat::Ident(ident) => &ident.ident,
                _ => panic!("Expected identifier pattern"),
            };
            (name, &pat_type.ty)
        }
        _ => panic!("Handler cannot have self parameter"),
    };

    // Generate the wrapper - named after the original function for graph-node to call
    // e.g., handle_transfer becomes handle_transfer (the extern "C" entry point)
    let impl_name = syn::Ident::new(&format!("__{}_impl", fn_name), fn_name.span());

    let expanded = quote! {
        // The implementation function (for native testing with MockHost)
        #fn_vis fn #impl_name<H: graphite::host::HostFunctions>(
            host: &mut H,
            #param_name: &#param_type
        ) #fn_body

        // Native (non-WASM) version - just calls impl with provided host
        #[cfg(not(target_arch = "wasm32"))]
        #fn_vis fn #fn_name<H: graphite::host::HostFunctions>(
            host: &mut H,
            #param_name: &#param_type
        ) {
            #impl_name(host, #param_name)
        }

        // The extern "C" wrapper for WASM - this is what graph-node calls
        #[cfg(target_arch = "wasm32")]
        #[unsafe(no_mangle)]
        pub extern "C" fn #fn_name(event_ptr: u32, event_len: u32) -> u32 {
            // Install panic hook (idempotent — only runs once)
            graphite::wasm::panic::install();

            // SAFETY: graph-node passes valid ptr+len pointing to serialized event
            let bytes = unsafe {
                core::slice::from_raw_parts(event_ptr as *const u8, event_len as usize)
            };

            // Deserialize the event from graph-node's TLV format
            let #param_name = match <#param_type as graphite::decode::FromWasmBytes>::from_wasm_bytes(bytes) {
                Ok(e) => e,
                Err(e) => {
                    // Log the decode error so it shows up in graph-node logs
                    let msg = alloc::format!(
                        "Failed to decode {} in {}: {}",
                        stringify!(#param_type),
                        stringify!(#fn_name),
                        e
                    );
                    let msg_ptr = msg.as_ptr() as u32;
                    let msg_len = msg.len() as u32;
                    unsafe {
                        graphite::wasm::ffi::log_log(3, msg_ptr, msg_len); // 3 = Error
                    }
                    return 1;
                }
            };

            // Create the WASM host for calling back into graph-node
            let mut host = graphite::wasm::WasmHost::new();

            // Call the actual handler implementation
            #impl_name(&mut host, &#param_name);

            0 // Success
        }
    };

    TokenStream::from(expanded)
}

/// Convert snake_case to camelCase for GraphQL field names.
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}
