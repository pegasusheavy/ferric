use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result};

/// Build CSS using TailwindCSS 4
pub fn build_css(input: PathBuf, output: PathBuf, minify: bool, watch: bool) -> Result<()> {
    println!("🎨 Building CSS with TailwindCSS 4...");

    let mut cmd = Command::new("npx");
    cmd.arg("@tailwindcss/cli@next")
        .arg("-i")
        .arg(&input)
        .arg("-o")
        .arg(&output);

    if minify {
        cmd.arg("--minify");
    }

    if watch {
        cmd.arg("--watch");
    }

    println!("📦 Input:  {}", input.display());
    println!("📦 Output: {}", output.display());

    if watch {
        println!("👀 Watching for changes...");
    }

    let status = cmd.status()
        .context("Failed to execute tailwindcss CLI")?;

    if !status.success() {
        anyhow::bail!("TailwindCSS build failed");
    }

    println!("✅ CSS build complete!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_build_css_creates_output() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.css");
        let output = temp_dir.path().join("output.css");

        // Create a minimal input file
        fs::write(&input, "@import 'tailwindcss';").unwrap();

        // This test requires npm/npx to be available
        // In CI, you might want to skip this or mock it
        if Command::new("npx").arg("--version").status().is_ok() {
            let result = build_css(input, output.clone(), false, false);
            assert!(result.is_ok() || output.exists());
        }
    }
}

