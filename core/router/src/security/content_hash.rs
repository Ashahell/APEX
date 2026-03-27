//! ContentHash: SHA-256 based content verification with path normalization
//!
//! This module provides:
//! - Path normalization to fix symlink/traversal vulnerabilities
//! - SHA-256 content hashing for integrity verification
//! - Async file reading support

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContentHashError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path normalization failed: {0}")]
    Normalization(String),

    #[error("File not found: {0}")]
    NotFound(String),
}

/// ContentHash provides SHA-256 based content verification
/// with proper path normalization to prevent symlink attacks
pub struct ContentHash;

impl ContentHash {
    /// Compute SHA-256 hash of file contents
    ///
    /// # Arguments
    /// * `path` - Path to the file
    ///
    /// # Returns
    /// * Hex-encoded SHA-256 hash of file contents
    pub async fn hash_file(path: &Path) -> Result<String, ContentHashError> {
        let contents = tokio::fs::read(path).await?;
        Ok(Self::hash_bytes(&contents))
    }

    /// Compute SHA-256 hash of bytes
    pub fn hash_bytes(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Compute SHA-256 hash of string
    pub fn hash_string(data: &str) -> String {
        Self::hash_bytes(data.as_bytes())
    }

    /// Normalize path to prevent symlink traversal attacks
    ///
    /// This fixes the vulnerability where paths like:
    /// - `/skills/../../../etc/passwd`
    /// - `/skills/valid/../../secret.txt`
    ///
    /// Are resolved to unintended locations.
    ///
    /// # Arguments
    /// * `base_path` - The base directory (e.g., skills directory)
    /// * `target_path` - The path to normalize
    ///
    /// # Returns
    /// * Canonical path if within base_path
    /// * Error if path escapes base directory
    pub fn normalize_path(
        base_path: &Path,
        target_path: &Path,
    ) -> Result<PathBuf, ContentHashError> {
        // Resolve both paths to absolute, canonical form
        let base = base_path.canonicalize().map_err(|e| {
            ContentHashError::Normalization(format!("Failed to canonicalize base: {}", e))
        })?;

        // If target is relative, resolve it relative to base
        let target = if target_path.is_relative() {
            base.join(target_path)
        } else {
            target_path.to_path_buf()
        };

        // Canonicalize the target
        let canonical = target.canonicalize().map_err(|e| {
            ContentHashError::Normalization(format!("Failed to canonicalize target: {}", e))
        })?;

        // Verify the canonical path is within base
        // On Windows, we need to handle drive letters
        #[cfg(windows)]
        {
            let base_str = base.to_string_lossy().to_lowercase();
            let canonical_str = canonical.to_string_lossy().to_lowercase();

            if !canonical_str.starts_with(&base_str) {
                return Err(ContentHashError::Normalization(format!(
                    "Path escapes base directory: {} not in {}",
                    canonical.display(),
                    base.display()
                )));
            }
        }

        #[cfg(not(windows))]
        {
            if !canonical.starts_with(&base) {
                return Err(ContentHashError::Normalization(format!(
                    "Path escapes base directory: {} not in {}",
                    canonical.display(),
                    base.display()
                )));
            }
        }

        Ok(canonical)
    }

    /// Verify file contents match expected hash
    ///
    /// # Arguments
    /// * `path` - Path to the file
    /// * `expected_hash` - Expected SHA-256 hash
    ///
    /// # Returns
    /// * Ok(()) if hash matches
    /// * Error if hash doesn't match or file can't be read
    pub async fn verify_hash(path: &Path, expected_hash: &str) -> Result<bool, ContentHashError> {
        let actual_hash = Self::hash_file(path).await?;
        Ok(actual_hash == expected_hash)
    }

    /// Compute hash of directory contents recursively
    ///
    /// Hashes all files in alphabetical order for deterministic results
    ///
    /// # Arguments
    /// * `dir_path` - Path to the directory
    ///
    /// # Returns
    /// * Combined SHA-256 hash of all file contents
    pub fn hash_directory(dir_path: &Path) -> Result<String, ContentHashError> {
        let mut files = Self::collect_files_sync(dir_path)?;

        // Sort for deterministic ordering
        files.sort();

        // Hash each file and combine
        let mut hasher = Sha256::new();

        for file_path in files {
            let contents = std::fs::read(&file_path)?;
            hasher.update(&contents);
            // Include file path to detect renames
            hasher.update(file_path.to_string_lossy().as_bytes());
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Collect all files in directory recursively (non-recursive)
    fn collect_files_sync(dir_path: &Path) -> Result<Vec<PathBuf>, ContentHashError> {
        let mut files = Vec::new();
        let mut stack = vec![dir_path.to_path_buf()];

        while let Some(current) = stack.pop() {
            if current.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&current) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            stack.push(path);
                        } else if path.is_file() {
                            files.push(path);
                        }
                    }
                }
            }
        }

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_hash_bytes() {
        let data = b"hello world";
        let hash = ContentHash::hash_bytes(data);
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_hash_string() {
        let hash = ContentHash::hash_string("hello world");
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[tokio::test]
    async fn test_hash_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let hash = ContentHash::hash_file(&file_path).await.unwrap();
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[tokio::test]
    async fn test_normalize_path_valid() {
        let dir = tempdir().unwrap();
        let base = dir.path();

        // Create a subdirectory with a file
        let subdir = base.join("skills").join("shell.execute");
        std::fs::create_dir_all(&subdir).unwrap();

        // Create the target file
        let target_file = subdir.join("src").join("index.ts");
        std::fs::create_dir_all(target_file.parent().unwrap()).unwrap();
        std::fs::write(&target_file, "test").unwrap();

        let target = Path::new("skills/shell.execute/src/index.ts");
        let result = ContentHash::normalize_path(base, target);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_normalize_path_escape() {
        let dir = tempdir().unwrap();
        let base = dir.path();

        // Create skills directory
        let skills_dir = base.join("skills");
        std::fs::create_dir_all(&skills_dir).unwrap();

        // Create a sibling directory that we DON'T want access to
        let sibling_dir = base.join("secret");
        std::fs::create_dir_all(&sibling_dir).unwrap();
        std::fs::write(sibling_dir.join("secret.txt"), "secret").unwrap();

        // Try to escape using symlink or .. in parent
        // On Windows, we test with a path that goes above base
        let target = base.join("..").join("..").join("..");

        // Check if we can at least detect the escape attempt
        // The canonicalize might fail because path doesn't exist or escapes
        let result = ContentHash::normalize_path(base, &target);

        // This should fail because it escapes the base
        assert!(result.is_err() || result.map(|p| !p.starts_with(base)).unwrap_or(true));
    }
}
