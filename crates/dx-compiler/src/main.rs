//! # Dx Compiler - The Factory
//!
//! The Transpiler-to-Binary Pipeline that converts `.tsx` into machine-executable `.dxb` and `.wasm`.
//!
//! ## Philosophy
//! "Separate Structure from Logic."
//! - Input: Developer writes `App.tsx`
//! - Parse: Use `swc` to parse the AST
//! - Split: Separate Static HTML from Dynamic Expressions
//! - Gen A: Serialize Static HTML to `layout.bin` (bincode)
//! - Gen B: Transpile Dynamic Logic to Rust (`generated.rs`)
//! - Compile: Invoke `rustc` to compile `generated.rs` into `logic.wasm`
//!
//! This gives you the performance of Rust WASM with the syntax of TypeScript.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Instant;

mod codegen;
mod dev_server;
mod packer;
mod parser;
mod splitter;

#[derive(Parser)]
#[command(name = "dx")]
#[command(about = "Dx Compiler - Transpiler-to-Binary Pipeline", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the project into optimized .dxb artifacts
    Build {
        /// Entry point file (default: src/main.dx)
        #[arg(short, long, default_value = "src/main.dx")]
        entry: PathBuf,

        /// Output directory (default: dist/)
        #[arg(short, long, default_value = "dist")]
        output: PathBuf,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,

        /// Skip WASM optimization (faster builds)
        #[arg(long)]
        skip_optimize: bool,
    },

    /// Start development mode with hot-swap
    Dev {
        /// Entry point file (default: src/main.dx)
        #[arg(short, long, default_value = "src/main.dx")]
        entry: PathBuf,

        /// Port for dev server (default: 3000)
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },

    /// Create a new Dx project
    New {
        /// Project name
        name: String,

        /// Use template (minimal, counter, todomvc)
        #[arg(short, long, default_value = "minimal")]
        template: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            entry,
            output,
            verbose,
            skip_optimize,
        } => {
            build_project(entry, output, verbose, skip_optimize).await?;
        },
        Commands::Dev {
            entry,
            port,
            verbose,
        } => {
            run_dev_server(entry, port, verbose).await?;
        },
        Commands::New { name, template } => {
            create_new_project(name, template)?;
        },
    }

    Ok(())
}

/// Build the project into optimized artifacts
async fn build_project(
    entry: PathBuf,
    output: PathBuf,
    verbose: bool,
    skip_optimize: bool,
) -> Result<()> {
    let start_time = Instant::now();

    println!("{}", style("🏭 Dx Compiler - Building...").bold().cyan());
    println!();

    // Create progress bar
    let pb = ProgressBar::new(6);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>2}/{len:2} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    // Step 1: Parse
    pb.set_message("Parsing .tsx files...");
    let parsed_ast = parser::parse_entry(&entry, verbose).context("Failed to parse entry file")?;
    pb.inc(1);

    // Step 2: Tree Shake
    pb.set_message("Tree shaking imports...");
    let shaken = parser::tree_shake(parsed_ast, verbose)?;
    pb.inc(1);

    // Step 3: Split
    pb.set_message("Splitting structure from logic...");
    let (templates, bindings, state_schema) = splitter::split_components(shaken, verbose)?;
    pb.inc(1);

    // Step 4: Generate HTIP Binary (NO RUST/WASM - pure data!)
    pb.set_message("Generating HTIP binary...");
    let (htip_stream, _string_table) =
        codegen::generate_htip(&templates, &bindings, &state_schema, verbose)?;
    pb.inc(1);

    // Step 5: Pack .dxb (templates + HTIP stream, NO WASM!)
    pb.set_message("Packing .dxb artifact...");
    packer::pack_dxb_htip(&output, &templates, &htip_stream, verbose)?;
    pb.inc(1);
    pb.inc(1); // Skip extra step

    pb.finish_with_message("Build complete!");

    let elapsed = start_time.elapsed();
    println!();
    println!("{} Built in {:.2}s", style("✓").green().bold(), elapsed.as_secs_f32());
    println!("  {} {}", style("Output:").dim(), output.display());
    println!();

    Ok(())
}

/// Run development server with hot-swap
async fn run_dev_server(entry: PathBuf, port: u16, verbose: bool) -> Result<()> {
    println!("{}", style("🔥 Dx Dev Server - Hot Module Replacement").bold().cyan());
    println!();
    println!("  {} http://localhost:{}", style("Local:").dim(), port);
    println!();

    dev_server::start(entry, port, verbose).await?;

    Ok(())
}

/// Create a new Dx project from template
fn create_new_project(name: String, template: String) -> Result<()> {
    println!("{}", style(format!("📦 Creating new project: {}", name)).bold().cyan());
    println!("  Template: {}", template);
    println!();

    // TODO: Implement project scaffolding
    println!("{}", style("⚠️  Project creation not yet implemented").yellow());
    println!("For now, copy the examples/hello-world template manually.");

    Ok(())
}
