//! Cache busting utilities
//!
//! Generates content-based hashes for static assets to ensure browsers
//! don't serve stale cached versions.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Cache busting strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// MD5 hash of file content
    Md5,
    /// SHA256 hash of file content
    Sha256,
    /// Unix timestamp
    Timestamp,
}

impl Strategy {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sha256" => Self::Sha256,
            "timestamp" => Self::Timestamp,
            _ => Self::Md5, // Default
        }
    }
}

/// Generate a hash for a file
pub fn hash_file(path: &Path, strategy: Strategy, length: usize) -> Result<String> {
    match strategy {
        Strategy::Md5 => hash_file_md5(path, length),
        Strategy::Sha256 => hash_file_sha256(path, length),
        Strategy::Timestamp => Ok(generate_timestamp()),
    }
}

/// Generate MD5 hash of file content
fn hash_file_md5(path: &Path, length: usize) -> Result<String> {
    let content = fs::read(path)
        .with_context(|| format!("Failed to read file for hashing: {}", path.display()))?;
    
    let hash = format!("{:x}", md5::compute(&content));
    Ok(hash[..length.min(hash.len())].to_string())
}

/// Generate SHA256 hash of file content
fn hash_file_sha256(path: &Path, length: usize) -> Result<String> {
    use sha2::{Sha256, Digest};
    
    let content = fs::read(path)
        .with_context(|| format!("Failed to read file for hashing: {}", path.display()))?;
    
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();
    let hash = format!("{:x}", result);
    Ok(hash[..length.min(hash.len())].to_string())
}

/// Generate timestamp-based version
fn generate_timestamp() -> String {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

/// Rename a file with cache busting hash
pub fn rename_with_hash(
    path: &Path,
    strategy: Strategy,
    length: usize,
) -> Result<PathBuf> {
    let hash = hash_file(path, strategy, length)?;
    
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .context("Invalid file name")?;
    
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    
    let new_name = if extension.is_empty() {
        format!("{}.{}", file_stem, hash)
    } else {
        format!("{}.{}.{}", file_stem, hash, extension)
    };
    
    let new_path = path.with_file_name(new_name);
    fs::rename(path, &new_path)
        .with_context(|| format!("Failed to rename {} to {}", path.display(), new_path.display()))?;
    
    Ok(new_path)
}

/// Apply cache busting to a file and return the mapping
pub fn apply_cache_busting(
    path: &Path,
    strategy: Strategy,
    length: usize,
) -> Result<(PathBuf, String)> {
    let original_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .context("Invalid file name")?
        .to_string();
    
    let new_path = rename_with_hash(path, strategy, length)?;
    
    Ok((new_path, original_name))
}

/// Update HTML file references to use hashed filenames
pub fn update_html_references(
    html_path: &Path,
    mappings: &HashMap<String, String>,
) -> Result<()> {
    let content = fs::read_to_string(html_path)
        .with_context(|| format!("Failed to read HTML file: {}", html_path.display()))?;
    
    let mut updated_content = content;
    
    // Replace references in src, href attributes
    for (original, hashed) in mappings {
        // Match various reference patterns
        let patterns = vec![
            format!(r#"src="{}""#, original),
            format!(r#"src='{}'"#, original),
            format!(r#"href="{}""#, original),
            format!(r#"href='{}'"#, original),
            // Relative paths
            format!(r#"src="./{}""#, original),
            format!(r#"href="./{}""#, original),
            // Absolute paths
            format!(r#"src="/{}""#, original),
            format!(r#"href="/{}""#, original),
        ];
        
        let replacements = vec![
            format!(r#"src="{}""#, hashed),
            format!(r#"src='{}'"#, hashed),
            format!(r#"href="{}""#, hashed),
            format!(r#"href='{}'"#, hashed),
            format!(r#"src="./{}""#, hashed),
            format!(r#"href="./{}""#, hashed),
            format!(r#"src="/{}""#, hashed),
            format!(r#"href="/{}""#, hashed),
        ];
        
        for (pattern, replacement) in patterns.iter().zip(replacements.iter()) {
            updated_content = updated_content.replace(pattern, replacement);
        }
    }
    
    fs::write(html_path, updated_content)
        .with_context(|| format!("Failed to write updated HTML: {}", html_path.display()))?;
    
    Ok(())
}

/// Process directory for cache busting
pub fn process_directory(
    dir: &Path,
    strategy: Strategy,
    length: usize,
    css: bool,
    js: bool,
    _assets: bool,
) -> Result<HashMap<String, String>> {
    let mut mappings = HashMap::new();
    
    if !dir.exists() {
        return Ok(mappings);
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let should_hash = if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                match ext {
                    "css" if css => true,
                    "js" | "wasm" if js => true,
                    _ => false,
                }
            } else {
                false
            };
            
            if should_hash {
                match apply_cache_busting(&path, strategy, length) {
                    Ok((new_path, original_name)) => {
                        let new_name = new_path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_string();
                        mappings.insert(original_name, new_name);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to hash {}: {}", path.display(), e);
                    }
                }
            }
        }
    }
    
    Ok(mappings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_hash_file_md5() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.css");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"body { color: red; }").unwrap();
        
        let hash = hash_file_md5(&file_path, 8).unwrap();
        assert_eq!(hash.len(), 8);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_timestamp() {
        let ts = generate_timestamp();
        assert!(!ts.is_empty());
        assert!(ts.parse::<u64>().is_ok());
    }

    #[test]
    fn test_rename_with_hash() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("styles.css");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"body { color: red; }").unwrap();
        drop(file);
        
        let new_path = rename_with_hash(&file_path, Strategy::Md5, 8).unwrap();
        assert!(new_path.exists());
        assert!(!file_path.exists());
        assert!(new_path.to_string_lossy().contains("styles."));
        assert!(new_path.to_string_lossy().ends_with(".css"));
    }
}

