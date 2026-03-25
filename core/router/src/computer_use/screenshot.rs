use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotConfig {
    pub display: u32,
    pub quality: u8,
    pub max_width: u32,
    pub max_height: u32,
    pub cache_ttl_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotManager {
    pub config: ScreenshotConfig,
    // Simple in-memory cache placeholder
    // In real implementation this would be a thread-safe cache
    // For skeleton we omit actual data
}

impl ScreenshotManager {
    pub fn new() -> Self {
        ScreenshotManager {
            config: ScreenshotConfig { display: 0, quality: 70, max_width: 1024, max_height: 768, cache_ttl_ms: 60000 },
        }
    }
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        ScreenshotConfig {
            display: 0,
            quality: 70,
            max_width: 1024,
            max_height: 768,
            cache_ttl_ms: 60000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedScreenshot {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub timestamp: i64,
    pub display: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotError {
    pub message: String,
}

impl ScreenshotManager {
    pub async fn capture(&self, _display: u32) -> Result<CapturedScreenshot, ScreenshotError> {
        Ok(CapturedScreenshot {
            data: vec![],
            width: 0,
            height: 0,
            format: ImageFormat::Png,
            timestamp: 0,
            display: _display,
        })
    }
}
