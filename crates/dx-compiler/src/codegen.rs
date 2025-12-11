//! # Codegen Module - The Rust Writer
//!
//! Writes temporary Rust code that powers the WASM.
//! Takes the "Binding Map" and writes a Rust `struct`.
//! Implements the `dirty_mask` logic automatically.

use anyhow::{Context, Result};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::splitter::{Binding, StateSchema, Template};

/// Generate Rust code from templates and bindings
pub fn generate_rust(
    templates: Vec<Template>,
    bindings: Vec<Binding>,
    schemas: Vec<StateSchema>,
    verbose: bool,
) -> Result<String> {
    if verbose {
        println!("  Generating Rust code...");
    }

    let mut generated = TokenStream::new();

    // Generate imports
    generated.extend(generate_imports());

    // Generate template registration
    generated.extend(generate_template_registration(&templates));

    // Generate component structs
    for schema in &schemas {
        generated.extend(generate_component_struct(schema, verbose)?);
    }

    // Generate update implementations
    for schema in &schemas {
        let component_bindings: Vec<_> = bindings
            .iter()
            .filter(|b| b.component == schema.component)
            .collect();
        generated.extend(generate_update_impl(schema, &component_bindings)?);
    }

    Ok(generated.to_string())
}

/// Generate imports for generated code
fn generate_imports() -> TokenStream {
    quote! {
        use wasm_bindgen::prelude::*;
        use web_sys::{Document, Element, HtmlElement, Node, window};

        // TODO: Import dx-dom, dx-morph, dx-sched from runtime
        // For now, inline minimal bindings

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = console)]
            fn log(s: &str);
        }

        const BIT_0: u64 = 1 << 0;
        const BIT_1: u64 = 1 << 1;
        const BIT_2: u64 = 1 << 2;
        const BIT_3: u64 = 1 << 3;
        const BIT_4: u64 = 1 << 4;
        const BIT_5: u64 = 1 << 5;
        const BIT_6: u64 = 1 << 6;
        const BIT_7: u64 = 1 << 7;
    }
}

/// Generate template registration code
fn generate_template_registration(templates: &[Template]) -> TokenStream {
    let template_count = templates.len();
    let template_html: Vec<_> = templates.iter().map(|t| &t.html).collect();

    quote! {
        #[wasm_bindgen(start)]
        pub fn init_templates() {
            // Register templates in the browser
            let window = window().expect("no global window");
            let document = window.document().expect("no document");

            #(
                {
                    let template = document.create_element("template").unwrap();
                    template.set_inner_html(#template_html);
                    // Store in global template cache
                }
            )*

            log(&format!("Registered {} templates", #template_count));
        }
    }
}

/// Generate component struct
fn generate_component_struct(schema: &StateSchema, verbose: bool) -> Result<TokenStream> {
    if verbose {
        println!("    Generating struct for {}", schema.component);
    }

    let component_name = Ident::new(&schema.component, Span::call_site());

    let field_names: Vec<_> = schema
        .fields
        .iter()
        .map(|f| Ident::new(&f.name, Span::call_site()))
        .collect();

    let field_types: Vec<_> = schema
        .fields
        .iter()
        .map(|f| {
            // Convert TypeScript types to Rust types
            let rust_type = match f.type_name.as_str() {
                "number" => "i32",
                "string" => "String",
                "boolean" => "bool",
                _ => "i32", // Default fallback
            };
            Ident::new(rust_type, Span::call_site())
        })
        .collect();

    let initial_values: Vec<_> = schema
        .fields
        .iter()
        .map(|f| {
            // Parse initial values
            let val_str = if f.type_name == "string" {
                format!("String::from(\"{}\")", f.initial_value)
            } else {
                f.initial_value.clone()
            };
            
            use std::str::FromStr;
            TokenStream::from_str(&val_str).unwrap_or_else(|_| quote!(0))
        })
        .collect();

    Ok(quote! {
        #[wasm_bindgen]
        pub struct #component_name {
            dirty_mask: u64,
            #(pub #field_names: #field_types,)*
        }

        #[wasm_bindgen]
        impl #component_name {
            #[wasm_bindgen(constructor)]
            pub fn new() -> Self {
                Self {
                    dirty_mask: 0,
                    #(#field_names: #initial_values,)*
                }
            }

            pub fn mark_dirty(&mut self, bit: u8) {
                self.dirty_mask |= 1u64 << bit;
            }

            pub fn clear_dirty(&mut self) {
                self.dirty_mask = 0;
            }

            pub fn is_dirty(&self, bit: u8) -> bool {
                (self.dirty_mask & (1u64 << bit)) != 0
            }
        }
    })
}

/// Generate update implementation
fn generate_update_impl(schema: &StateSchema, bindings: &[&Binding]) -> Result<TokenStream> {
    let component_name = Ident::new(&schema.component, Span::call_site());

    // Group bindings by dirty bit
    let mut bit_groups: HashMap<u8, Vec<&Binding>> = HashMap::new();
    for binding in bindings {
        bit_groups
            .entry(binding.dirty_bit)
            .or_insert_with(Vec::new)
            .push(binding);
    }

    let update_checks: Vec<_> = bit_groups
        .iter()
        .map(|(bit, group)| {
            let bit_lit = *bit;
            let updates: Vec<_> = group
                .iter()
                .map(|binding| {
                    let slot_id = binding.slot_id;
                    let expr = &binding.expression;
                    quote! {
                        // Update slot #slot_id with #expr
                        log(&format!("Updating slot {} with value: {:?}", #slot_id, #expr));
                        // TODO: Call dx_dom::update_text or update_attribute
                    }
                })
                .collect();

            quote! {
                if self.is_dirty(#bit_lit) {
                    #(#updates)*
                }
            }
        })
        .collect();

    Ok(quote! {
        #[wasm_bindgen]
        impl #component_name {
            pub fn update(&mut self) {
                if self.dirty_mask == 0 {
                    return; // Nothing to update
                }

                #(#update_checks)*

                self.clear_dirty();
            }

            pub fn render(&self) {
                log(&format!("Rendering {}", stringify!(#component_name)));
                // TODO: Clone template and populate slots
            }
        }
    })
}

/// Compile generated Rust code to WASM
pub fn compile_to_wasm(rust_code: String, skip_optimize: bool, verbose: bool) -> Result<Vec<u8>> {
    if verbose {
        println!("  Compiling to WASM...");
    }

    // Use fixed debug directory for easier inspection
    let temp_dir = std::env::current_dir()?.join("debug_out");
    if fs::exists(&temp_dir)? {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir_all(&temp_dir).context("Failed to create debug directory")?;

    if verbose {
        println!("    Debug build dir: {}", temp_dir.display());
    }

    // Write Cargo.toml
    let cargo_toml = r#"[package]
name = "dx-generated"
version = "0.1.0"
edition = "2021"

[workspace]

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["Window", "Document", "Element", "HtmlElement", "Node"] }

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
"#;

    fs::write(temp_dir.join("Cargo.toml"), cargo_toml)?;

    // Create src directory
    let src_dir = temp_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    // Write generated lib.rs
    fs::write(src_dir.join("lib.rs"), rust_code)?;

    // Run cargo build
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .arg("--release")
        .current_dir(&temp_dir);

    if verbose {
        println!("    Running: {:?}", cmd);
    }

    let output = cmd.output().context("Failed to run cargo build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Cargo build failed:\n{}", stderr));
    }

    // Read compiled WASM
    let wasm_path = temp_dir
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("dx_generated.wasm");

    let mut wasm_bytes = fs::read(&wasm_path).context("Failed to read compiled WASM")?;

    // Optimize with wasm-opt if requested
    if !skip_optimize {
        if verbose {
            println!("  Running wasm-opt...");
        }

        // Check if wasm-opt is available
        if Command::new("wasm-opt").arg("--version").output().is_ok() {
            let optimized_path = temp_dir.join("optimized.wasm");

            let status = Command::new("wasm-opt")
                .arg("-Oz")
                .arg(&wasm_path)
                .arg("-o")
                .arg(&optimized_path)
                .status()
                .context("Failed to run wasm-opt")?;

            if status.success() {
                wasm_bytes = fs::read(&optimized_path)?;
            } else if verbose {
                println!("    wasm-opt failed, using unoptimized WASM");
            }
        } else if verbose {
            println!("    wasm-opt not found, skipping optimization");
        }
    }

    // Cleanup temp directory
    if !verbose {
        let _ = fs::remove_dir_all(&temp_dir);
    }

    if verbose {
        println!("  WASM size: {} bytes", wasm_bytes.len());
    }

    Ok(wasm_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_generation() {
        let schema = StateSchema {
            component: "Counter".to_string(),
            fields: vec![StateField {
                name: "count".to_string(),
                type_name: "number".to_string(),
                initial_value: "0".to_string(),
                dirty_bit: 0,
            }],
        };

        let result = generate_component_struct(&schema, false);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let code = tokens.to_string();
        assert!(code.contains("pub struct Counter"));
        assert!(code.contains("dirty_mask"));
    }
}
