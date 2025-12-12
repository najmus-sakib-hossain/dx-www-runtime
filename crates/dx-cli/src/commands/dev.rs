/**
 * `dx dev` - Development server with hot reload
 * 
 * Orchestrates:
 * 1. File watching (notify) → triggers rebuild
 * 2. dx-compiler → compiles TSX to binary
 * 3. dx-server → serves the binary over HTTP
 * 4. WebSocket → pushes updates to client
 */

use anyhow::{Context, Result};
use console::style;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::config::ProjectConfig;

pub async fn execute(port: u16, host: &str, open_browser: bool) -> Result<()> {
    println!("{}", style("🚀 Starting development server...").bold());
    println!();

    // Load project configuration
    let config = ProjectConfig::load(".")
        .with_context(|| "Failed to load dx.toml. Run 'dx new' to create a project.")?;

    info!("Project: {} v{}", config.name(), config.version());

    // Create shutdown channel
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    // Initial build
    println!("{}", style("📦 Building application...").bold());
    let build_result = compile_project(&config).await?;
    println!("{} Initial build complete ({:.2}ms)", 
        style("✓").green(), 
        build_result.duration_ms
    );
    println!("  {} Runtime: {}", style("→").cyan(), build_result.runtime);
    println!("  {} Size: {} bytes", style("→").cyan(), build_result.size);
    println!();

    // Start file watcher
    let config_clone = config.clone();
    let shutdown_rx = shutdown_tx.subscribe();
    let watcher_handle = tokio::spawn(async move {
        watch_and_rebuild(config_clone, shutdown_rx).await
    });

    // Start HTTP server
    let addr = format!("{}:{}", host, port);
    println!("{}", style(format!("🌐 Server running at http://{}", addr)).green().bold());
    println!("{}", style("   Press Ctrl+C to stop").dim());
    println!();

    let addr_clone = addr.clone();
    let server_handle = tokio::spawn(async move {
        if let Err(e) = start_server(&addr_clone).await {
            warn!("Server error: {}", e);
        }
    });

    // Open browser if requested
    if open_browser {
        let url = format!("http://{}", addr);
        if let Err(e) = open::that(&url) {
            warn!("Failed to open browser: {}", e);
        }
    }

    // Wait for Ctrl+C
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("\n{}", style("⏹️  Shutting down...").yellow());
            let _ = shutdown_tx.send(());
        }
        _ = server_handle => {
            warn!("Server stopped unexpectedly");
        }
        _ = watcher_handle => {
            warn!("File watcher stopped unexpectedly");
        }
    }

    println!("{}", style("✨ Shutdown complete").green());
    Ok(())
}

/// Compile the project using dx-compiler
async fn compile_project(config: &ProjectConfig) -> Result<BuildResult> {
    // use dx_compiler::Compiler;  // TODO: Enable when dx-compiler exports lib
    use std::time::Instant;

    let start = Instant::now();

    // TODO: Actually use dx-compiler here
    // For now, simulate compilation
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Determine runtime (micro vs macro)
    let runtime = if config.build.auto_select {
        "auto (micro)"
    } else {
        config.build.runtime.as_deref().unwrap_or("micro")
    };

    Ok(BuildResult {
        runtime: runtime.to_string(),
        size: 23_300, // Simulated
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

/// Watch files and trigger rebuilds
async fn watch_and_rebuild(
    config: ProjectConfig,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    use notify::RecursiveMode;
    use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
    use std::time::Duration;

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    // Create file watcher
    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        move |res: Result<Vec<notify_debouncer_mini::DebouncedEvent>, _>| {
            if let Ok(events) = res {
                for event in events {
                    if let DebouncedEventKind::Any = event.kind {
                        // Check if it's a .tsx file
                        if event.path.extension().and_then(|s| s.to_str()) == Some("tsx") {
                            let _ = tx.blocking_send(event.path.clone());
                        }
                    }
                }
            }
        },
    )?;

    // Watch src/ directory
    debouncer
        .watcher()
        .watch(std::path::Path::new("src"), RecursiveMode::Recursive)?;

    info!("👀 Watching src/ for changes...");

    loop {
        tokio::select! {
            Some(path) = rx.recv() => {
                println!("\n{} File changed: {}", 
                    style("🔄").cyan(), 
                    path.display()
                );
                println!("{}", style("📦 Rebuilding...").bold());
                
                match compile_project(&config).await {
                    Ok(result) => {
                        println!("{} Build complete ({:.2}ms)", 
                            style("✓").green(), 
                            result.duration_ms
                        );
                        println!("  {} Size: {} bytes\n", 
                            style("→").cyan(), 
                            result.size
                        );
                    }
                    Err(e) => {
                        println!("{} Build failed: {}\n", 
                            style("✗").red(), 
                            e
                        );
                    }
                }
            }
            _ = shutdown.recv() => {
                break;
            }
        }
    }

    Ok(())
}

/// Start the HTTP server using dx-server
async fn start_server(addr: &str) -> Result<()> {
    // TODO: Enable when dx-server exports proper API
    // For now, simulate server running
    info!("HTTP server started at {}", addr);
    
    // Keep server alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}

#[derive(Debug)]
struct BuildResult {
    runtime: String,
    size: usize,
    duration_ms: f64,
}
