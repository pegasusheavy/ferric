//! WASM chunking and code splitting utilities
//!
//! Enables splitting WASM applications into multiple chunks for on-demand loading.

use anyhow::{Context, Result};
use console::style;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// WASM chunking configuration
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// Main chunk (always loaded)
    pub main: ChunkDefinition,
    /// Lazy-loaded chunks
    pub lazy_chunks: Vec<ChunkDefinition>,
    /// Output directory
    pub out_dir: PathBuf,
    /// Enable release mode
    pub release: bool,
}

/// Definition of a WASM chunk
#[derive(Debug, Clone)]
pub struct ChunkDefinition {
    /// Chunk name (e.g., "main", "admin", "dashboard")
    pub name: String,
    /// Rust crate or module path
    pub module_path: String,
    /// Priority (higher loads first)
    pub priority: u8,
    /// Preload strategy
    pub preload: PreloadStrategy,
}

/// Preload strategy for lazy chunks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreloadStrategy {
    /// Load immediately after main chunk
    Immediate,
    /// Load when idle
    Idle,
    /// Load on route activation
    OnRoute,
    /// Load on user interaction
    OnInteraction,
    /// Manual loading only
    Manual,
}

impl ChunkConfig {
    /// Create a new chunk configuration
    pub fn new(out_dir: PathBuf, release: bool) -> Self {
        Self {
            main: ChunkDefinition {
                name: "main".to_string(),
                module_path: "src/lib.rs".to_string(),
                priority: 0,
                preload: PreloadStrategy::Immediate,
            },
            lazy_chunks: Vec::new(),
            out_dir,
            release,
        }
    }

    /// Add a lazy chunk
    pub fn add_lazy_chunk(&mut self, chunk: ChunkDefinition) {
        self.lazy_chunks.push(chunk);
    }
}

/// Build WASM chunks
pub async fn build_chunks(config: &ChunkConfig) -> Result<ChunkManifest> {
    println!(
        "  {} Building {} WASM chunks...",
        style("→").cyan(),
        1 + config.lazy_chunks.len()
    );

    let mut manifest = ChunkManifest::new();

    // Build main chunk
    let main_chunk = build_chunk(&config.main, &config.out_dir, config.release).await?;
    manifest.add_chunk("main", main_chunk);
    println!(
        "    {} Main chunk: {} ({} KB)",
        style("✓").green(),
        manifest.chunks["main"].wasm_file,
        manifest.chunks["main"].size_kb
    );

    // Build lazy chunks
    for (i, chunk_def) in config.lazy_chunks.iter().enumerate() {
        println!(
            "    {} Building lazy chunk {}/{}...",
            style("→").cyan(),
            i + 1,
            config.lazy_chunks.len()
        );

        let chunk = build_chunk(chunk_def, &config.out_dir, config.release).await?;
        let size_kb = chunk.size_kb;
        let wasm_file = chunk.wasm_file.clone();

        manifest.add_chunk(&chunk_def.name, chunk);

        println!(
            "      {} {}: {} ({} KB)",
            style("✓").green(),
            chunk_def.name,
            wasm_file,
            size_kb
        );
    }

    // Generate preload manifest
    manifest.save(&config.out_dir.join("chunk-manifest.json"))?;

    Ok(manifest)
}

/// Build a single WASM chunk
async fn build_chunk(
    chunk: &ChunkDefinition,
    out_dir: &Path,
    release: bool,
) -> Result<ChunkInfo> {
    let chunk_out_dir = out_dir.join("pkg").join(&chunk.name);
    fs::create_dir_all(&chunk_out_dir)?;

    let mut args = vec!["build", "--target", "web"];

    if release {
        args.push("--release");
    }

    let out_arg = format!("--out-dir={}", chunk_out_dir.display());
    args.push(&out_arg);

    let out_name = format!("--out-name={}", chunk.name);
    args.push(&out_name);

    // Run wasm-pack
    let status = Command::new("wasm-pack")
        .args(&args)
        .status()
        .context("Failed to run wasm-pack")?;

    if !status.success() {
        anyhow::bail!("wasm-pack build failed for chunk: {}", chunk.name);
    }

    // Get chunk info
    let wasm_file = format!("{}_bg.wasm", chunk.name);
    let wasm_path = chunk_out_dir.join(&wasm_file);
    let js_file = format!("{}.js", chunk.name);

    let size = fs::metadata(&wasm_path)?.len();
    let size_kb = size / 1024;

    Ok(ChunkInfo {
        name: chunk.name.clone(),
        wasm_file,
        js_file,
        size: size as usize,
        size_kb: size_kb as usize,
        preload: chunk.preload,
        priority: chunk.priority,
    })
}

/// Information about a built chunk
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChunkInfo {
    pub name: String,
    pub wasm_file: String,
    pub js_file: String,
    pub size: usize,
    pub size_kb: usize,
    #[serde(skip)]
    pub preload: PreloadStrategy,
    pub priority: u8,
}

/// Manifest of all chunks
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChunkManifest {
    pub chunks: HashMap<String, ChunkInfo>,
    pub total_size_kb: usize,
}

impl ChunkManifest {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            total_size_kb: 0,
        }
    }

    pub fn add_chunk(&mut self, name: &str, info: ChunkInfo) {
        self.total_size_kb += info.size_kb;
        self.chunks.insert(name.to_string(), info);
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

/// Generate HTML script tags for chunk loading
pub fn generate_chunk_loader(manifest: &ChunkManifest) -> String {
    let mut script = String::from(
        r#"
// Ferric WASM Chunk Loader
const FerricChunks = {
  loaded: new Set(),
  loading: new Map(),
  
  async loadChunk(name) {
    if (this.loaded.has(name)) {
      return true;
    }
    
    if (this.loading.has(name)) {
      return await this.loading.get(name);
    }
    
    const promise = this._loadChunkImpl(name);
    this.loading.set(name, promise);
    
    try {
      await promise;
      this.loaded.add(name);
      this.loading.delete(name);
      return true;
    } catch (err) {
      this.loading.delete(name);
      throw err;
    }
  },
  
  async _loadChunkImpl(name) {
    const script = document.createElement('script');
    script.type = 'module';
    script.src = `pkg/${name}/${name}.js`;
    
    return new Promise((resolve, reject) => {
      script.onload = resolve;
      script.onerror = reject;
      document.head.appendChild(script);
    });
  },
  
  preloadChunk(name) {
    const link = document.createElement('link');
    link.rel = 'modulepreload';
    link.href = `pkg/${name}/${name}.js`;
    document.head.appendChild(link);
  }
};

// Chunk manifest
const CHUNK_MANIFEST = "#,
    );

    script.push_str(&serde_json::to_string_pretty(manifest).unwrap_or_default());
    script.push_str(
        r#";

// Preload high-priority chunks
window.addEventListener('load', () => {
  requestIdleCallback(() => {
    // Preload chunks based on priority
    const chunks = Object.entries(CHUNK_MANIFEST.chunks)
      .filter(([name]) => name !== 'main')
      .sort((a, b) => (b[1].priority || 0) - (a[1].priority || 0));
    
    for (const [name] of chunks) {
      FerricChunks.preloadChunk(name);
    }
  });
});

window.FerricChunks = FerricChunks;
"#,
    );

    script
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_config_creation() {
        let config = ChunkConfig::new(PathBuf::from("dist"), true);
        assert_eq!(config.main.name, "main");
        assert!(config.lazy_chunks.is_empty());
    }

    #[test]
    fn test_chunk_manifest() {
        let mut manifest = ChunkManifest::new();
        manifest.add_chunk(
            "test",
            ChunkInfo {
                name: "test".to_string(),
                wasm_file: "test_bg.wasm".to_string(),
                js_file: "test.js".to_string(),
                size: 1024,
                size_kb: 1,
                preload: PreloadStrategy::Idle,
                priority: 5,
            },
        );

        assert_eq!(manifest.chunks.len(), 1);
        assert_eq!(manifest.total_size_kb, 1);
    }
}

