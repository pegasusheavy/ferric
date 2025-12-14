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
    
    // Replace references in src, href, and other attributes
    for (original, hashed) in mappings {
        updated_content = replace_asset_references(&updated_content, original, hashed);
    }
    
    fs::write(html_path, updated_content)
        .with_context(|| format!("Failed to write updated HTML: {}", html_path.display()))?;
    
    Ok(())
}

/// Replace all asset references in HTML content
fn replace_asset_references(content: &str, original: &str, hashed: &str) -> String {
    let mut result = content.to_string();
    
    // Extract filename for path-aware matching
    let original_filename = Path::new(original)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(original);
    
    // Pattern 1: Direct references (src="file.css", href="file.js")
    let patterns = vec![
        // Quoted attributes with various paths
        (format!(r#"src="{}""#, original), format!(r#"src="{}""#, hashed)),
        (format!(r#"src='{}'"#, original), format!(r#"src='{}'"#, hashed)),
        (format!(r#"href="{}""#, original), format!(r#"href="{}""#, hashed)),
        (format!(r#"href='{}'"#, original), format!(r#"href='{}'"#, hashed)),
        
        // With ./ prefix
        (format!(r#"src="./{}""#, original), format!(r#"src="./{}""#, hashed)),
        (format!(r#"src='./{}'"#, original), format!(r#"src='./{}'"#, hashed)),
        (format!(r#"href="./{}""#, original), format!(r#"href="./{}""#, hashed)),
        (format!(r#"href='./{}'"#, original), format!(r#"href='./{}'"#, hashed)),
        
        // With / prefix (absolute)
        (format!(r#"src="/{}""#, original), format!(r#"src="/{}""#, hashed)),
        (format!(r#"src='/{}'"#, original), format!(r#"src='/{}'"#, hashed)),
        (format!(r#"href="/{}""#, original), format!(r#"href="/{}""#, hashed)),
        (format!(r#"href='/{}'"#, original), format!(r#"href='/{}'"#, hashed)),
        
        // In directories (pkg/file.js, styles/file.css)
        (format!(r#"src="pkg/{}""#, original_filename), format!(r#"src="pkg/{}""#, hashed)),
        (format!(r#"src='pkg/{}'"#, original_filename), format!(r#"src='pkg/{}'"#, hashed)),
        (format!(r#"href="pkg/{}""#, original_filename), format!(r#"href="pkg/{}""#, hashed)),
        (format!(r#"href='pkg/{}'"#, original_filename), format!(r#"href='pkg/{}'"#, hashed)),
        
        (format!(r#"src="styles/{}""#, original_filename), format!(r#"src="styles/{}""#, hashed)),
        (format!(r#"src='styles/{}'"#, original_filename), format!(r#"src='styles/{}'"#, hashed)),
        (format!(r#"href="styles/{}""#, original_filename), format!(r#"href="styles/{}""#, hashed)),
        (format!(r#"href='styles/{}'"#, original_filename), format!(r#"href='styles/{}'"#, hashed)),
        
        (format!(r#"src="dist/{}""#, original_filename), format!(r#"src="dist/{}""#, hashed)),
        (format!(r#"href="dist/{}""#, original_filename), format!(r#"href="dist/{}""#, hashed)),
        
        // Import statements in inline scripts
        (format!(r#"import("./{}""#, original), format!(r#"import("./{}""#, hashed)),
        (format!(r#"import('./{}'"#, original), format!(r#"import('./{}'"#, hashed)),
        (format!(r#"from "{}""#, original), format!(r#"from "{}""#, hashed)),
        (format!(r#"from '{}'"#, original), format!(r#"from '{}'"#, hashed)),
        
        // Link preload/prefetch
        (format!(r#"<link rel="preload" href="{}""#, original), format!(r#"<link rel="preload" href="{}""#, hashed)),
        (format!(r#"<link rel="prefetch" href="{}""#, original), format!(r#"<link rel="prefetch" href="{}""#, hashed)),
        (format!(r#"<link rel="modulepreload" href="{}""#, original), format!(r#"<link rel="modulepreload" href="{}""#, hashed)),
    ];
    
    for (pattern, replacement) in patterns {
        result = result.replace(&pattern, &replacement);
    }
    
    result
}

/// Process directory for cache busting
pub fn process_directory(
    dir: &Path,
    strategy: Strategy,
    length: usize,
    css: bool,
    js: bool,
    assets: bool,
) -> Result<HashMap<String, String>> {
    let mut mappings = HashMap::new();
    
    if !dir.exists() {
        return Ok(mappings);
    }
    
    // Recursively process directory
    process_directory_recursive(dir, dir, strategy, length, css, js, assets, &mut mappings)?;
    
    Ok(mappings)
}

/// Recursively process directory and subdirectories
fn process_directory_recursive(
    base_dir: &Path,
    current_dir: &Path,
    strategy: Strategy,
    length: usize,
    css: bool,
    js: bool,
    assets: bool,
    mappings: &mut HashMap<String, String>,
) -> Result<()> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Recursively process subdirectories
            process_directory_recursive(
                base_dir,
                &path,
                strategy,
                length,
                css,
                js,
                assets,
                mappings,
            )?;
        } else if path.is_file() {
            let should_hash = if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                match ext {
                    "css" if css => true,
                    "js" | "wasm" if js => true,
                    "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" if assets => true,
                    "woff" | "woff2" | "ttf" | "eot" | "otf" if assets => true,
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
                        
                        // Store both filename and relative path for better matching
                        mappings.insert(original_name.clone(), new_name.clone());
                        
                        // Also store relative path from base dir
                        if let Ok(rel_path) = path.parent().unwrap_or(base_dir).strip_prefix(base_dir) {
                            if !rel_path.as_os_str().is_empty() {
                                let rel_original = rel_path.join(&original_name);
                                let rel_new = rel_path.join(&new_name);
                                
                                if let (Some(o), Some(n)) = (rel_original.to_str(), rel_new.to_str()) {
                                    mappings.insert(o.to_string(), n.to_string());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to hash {}: {}", path.display(), e);
                    }
                }
            }
        }
    }
    
    Ok(())
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

