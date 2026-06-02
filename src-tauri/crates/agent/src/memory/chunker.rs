//! Text chunking (`agent/memory/chunker.py`).

#[derive(Debug, Clone)]
pub struct TextChunk {
    pub text: String,
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone)]
pub struct TextChunker {
    max_tokens: usize,
    overlap_tokens: usize,
    chars_per_token: usize,
}

impl TextChunker {
    pub fn new(max_tokens: usize, overlap_tokens: usize) -> Self {
        Self {
            max_tokens,
            overlap_tokens,
            chars_per_token: 4,
        }
    }

    pub fn chunk_text(&self, text: &str) -> Vec<TextChunk> {
        if text.trim().is_empty() {
            return vec![];
        }

        let lines: Vec<&str> = text.split('\n').collect();
        let max_chars = self.max_tokens * self.chars_per_token;
        let overlap_chars = self.overlap_tokens * self.chars_per_token;

        let mut chunks = Vec::new();
        let mut current_chunk: Vec<String> = Vec::new();
        let mut current_chars = 0usize;
        let mut start_line = 1u32;

        for (i, line) in lines.iter().enumerate() {
            let line_no = (i + 1) as u32;
            let line_chars = line.len();

            if line_chars > max_chars {
                if !current_chunk.is_empty() {
                    chunks.push(TextChunk {
                        text: current_chunk.join("\n"),
                        start_line,
                        end_line: line_no.saturating_sub(1),
                    });
                    current_chunk.clear();
                    current_chars = 0;
                }
                for sub in split_long_line(line, max_chars) {
                    chunks.push(TextChunk {
                        text: sub,
                        start_line: line_no,
                        end_line: line_no,
                    });
                }
                start_line = line_no + 1;
                continue;
            }

            if current_chars + line_chars > max_chars && !current_chunk.is_empty() {
                chunks.push(TextChunk {
                    text: current_chunk.join("\n"),
                    start_line,
                    end_line: line_no.saturating_sub(1),
                });
                let overlap_lines = overlap_lines_owned(&current_chunk, overlap_chars);
                let overlap_len = overlap_lines.len();
                current_chunk = overlap_lines;
                current_chunk.push(line.to_string());
                current_chars = current_chunk.iter().map(|l| l.len()).sum();
                start_line = line_no.saturating_sub(overlap_len as u32);
            } else {
                current_chunk.push(line.to_string());
                current_chars += line_chars;
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(TextChunk {
                text: current_chunk.join("\n"),
                start_line,
                end_line: lines.len() as u32,
            });
        }

        chunks
    }
}

fn split_long_line(line: &str, max_chars: usize) -> Vec<String> {
    line.as_bytes()
        .chunks(max_chars)
        .filter_map(|chunk| std::str::from_utf8(chunk).ok())
        .map(|s| s.to_string())
        .collect()
}

fn overlap_lines_owned(lines: &[String], target_chars: usize) -> Vec<String> {
    let mut overlap = Vec::new();
    let mut chars = 0usize;
    for line in lines.iter().rev() {
        if chars + line.len() > target_chars {
            break;
        }
        overlap.insert(0, line.clone());
        chars += line.len();
    }
    overlap
}
