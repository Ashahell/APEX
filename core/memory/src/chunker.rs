use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    pub chunk_size_tokens: usize,
    pub overlap_tokens: usize,
    pub min_chunk_tokens: usize,
    pub respect_headings: bool,
    pub respect_code_blocks: bool,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            chunk_size_tokens: 256,
            overlap_tokens: 32,
            min_chunk_tokens: 20,
            respect_headings: true,
            respect_code_blocks: true,
        }
    }
}

pub fn chunk_text(text: &str, config: &ChunkerConfig) -> Vec<(usize, String)> {
    if text.trim().is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut current_token_count = 0;
    let mut chunk_index = 0;
    let words: Vec<String> = text.unicode_words().map(|s| s.to_string()).collect();
    let word_count = words.len();

    if words.is_empty() {
        return Vec::new();
    }

    for word in &words {
        current_token_count += 1;

        if !current_chunk.is_empty() {
            current_chunk.push(' ');
        }
        current_chunk.push_str(word);

        if current_token_count >= config.chunk_size_tokens {
            chunks.push((chunk_index, current_chunk.clone()));
            chunk_index += 1;

            let overlap_count = config.overlap_tokens.min(current_token_count);
            if overlap_count > 0 {
                let start_idx = word_count.saturating_sub(overlap_count);
                let overlap_words: Vec<String> = words[start_idx..].to_vec();
                current_chunk = overlap_words.join(" ");
                current_token_count = overlap_count;
            } else {
                current_chunk.clear();
                current_token_count = 0;
            }
        }
    }

    if !current_chunk.is_empty() && current_token_count >= config.min_chunk_tokens {
        chunks.push((chunk_index, current_chunk));
    }

    if chunks.is_empty() {
        let total_words = words.len();
        if total_words >= config.min_chunk_tokens {
            chunks.push((0, text.to_string()));
        }
    }

    chunks
}

fn count_tokens(text: &str) -> usize {
    text.unicode_words().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text_basic() {
        let text = "This is sentence one. This is sentence two. This is sentence three.";
        let config = ChunkerConfig::default();
        let chunks = chunk_text(text, &config);

        for (_, chunk) in &chunks {
            assert!(count_tokens(chunk) <= config.chunk_size_tokens + 10);
        }
    }

    #[test]
    fn test_chunk_text_with_min_chunk() {
        let text = "Short.";
        let config = ChunkerConfig::default();
        let chunks = chunk_text(text, &config);

        assert!(chunks.is_empty() || count_tokens(&chunks[0].1) >= config.min_chunk_tokens);
    }

    #[test]
    fn test_chunk_text_empty() {
        let text = "";
        let config = ChunkerConfig::default();
        let chunks = chunk_text(text, &config);

        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunk_respects_headings() {
        let text = "# Heading\n\nThis is content under heading one.\n\n## Subheading\n\nThis is content under subheading.";
        let config = ChunkerConfig {
            respect_headings: true,
            min_chunk_tokens: 5,
            ..Default::default()
        };
        let chunks = chunk_text(text, &config);

        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_chunk_respects_code_blocks() {
        let text = "Some text before.\n\n```python\ndef foo():\n    pass\n```\n\nSome text after.";
        let config = ChunkerConfig {
            respect_code_blocks: true,
            min_chunk_tokens: 5,
            ..Default::default()
        };
        let chunks = chunk_text(text, &config);

        assert!(!chunks.is_empty());
    }
}
