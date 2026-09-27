use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    Fixed,
    Free,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorChar {
    Comment,        // *
    FormFeed,       // /
    Continuation,   // -
    Debug,          // D or d
    None,
}

impl IndicatorChar {
    pub fn from_char(c: char) -> Self {
        match c {
            '*' => IndicatorChar::Comment,
            '/' => IndicatorChar::FormFeed,
            '-' => IndicatorChar::Continuation,
            'D' | 'd' => IndicatorChar::Debug,
            _ => IndicatorChar::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FixedFormatLine {
    pub sequence: String,      // Columns 1-6
    pub indicator: IndicatorChar, // Column 7
    pub area_a: String,        // Columns 8-11
    pub area_b: String,        // Columns 12-72
    pub identification: String, // Columns 73-80
    pub original_line: usize,
}

impl FixedFormatLine {
    pub fn parse(line: &str, line_number: usize) -> Self {
        let chars: Vec<char> = line.chars().collect();
        
        let sequence = if chars.len() >= 6 {
            chars[0..6].iter().collect()
        } else {
            String::new()
        };

        let indicator = if chars.len() >= 7 {
            IndicatorChar::from_char(chars[6])
        } else {
            IndicatorChar::None
        };

        let area_a = if chars.len() >= 11 {
            chars[7..11].iter().collect()
        } else if chars.len() > 7 {
            chars[7..].iter().collect()
        } else {
            String::new()
        };

        let area_b = if chars.len() >= 72 {
            chars[11..72].iter().collect()
        } else if chars.len() > 11 {
            chars[11..].iter().collect()
        } else {
            String::new()
        };

        let identification = if chars.len() >= 80 {
            chars[72..80].iter().collect()
        } else if chars.len() > 72 {
            chars[72..].iter().collect()
        } else {
            String::new()
        };

        Self {
            sequence,
            indicator,
            area_a,
            area_b,
            identification,
            original_line: line_number,
        }
    }

    pub fn is_comment(&self) -> bool {
        matches!(self.indicator, IndicatorChar::Comment)
    }

    pub fn is_continuation(&self) -> bool {
        matches!(self.indicator, IndicatorChar::Continuation)
    }

    pub fn is_debug(&self) -> bool {
        matches!(self.indicator, IndicatorChar::Debug)
    }

    pub fn content(&self) -> String {
        format!("{}{}", self.area_a, self.area_b).trim_end().to_string()
    }
}

pub struct FormatNormalizer {
    format: SourceFormat,
}

impl FormatNormalizer {
    pub fn new(format: SourceFormat) -> Self {
        Self { format }
    }

    pub fn detect_format(source: &str) -> SourceFormat {
        // Check for free format directive
        if source.contains(">>SOURCE FORMAT FREE") || source.contains(">>SOURCE FREE") {
            return SourceFormat::Free;
        }

        // Heuristic: if most lines have content in columns 1-6, likely fixed format
        let lines: Vec<&str> = source.lines().collect();
        let mut fixed_indicators = 0;
        let mut total_lines = 0;

        for line in lines.iter().take(100) {
            if line.len() >= 7 {
                total_lines += 1;
                let col7 = line.chars().nth(6).unwrap_or(' ');
                if matches!(col7, '*' | '/' | '-' | 'D' | 'd' | ' ') {
                    fixed_indicators += 1;
                }
            }
        }

        if total_lines > 0 && fixed_indicators as f32 / total_lines as f32 > 0.7 {
            SourceFormat::Fixed
        } else {
            SourceFormat::Free
        }
    }

    pub fn normalize(&self, source: &str) -> Result<String> {
        match self.format {
            SourceFormat::Fixed => self.normalize_fixed(source),
            SourceFormat::Free => Ok(source.to_string()),
        }
    }

    fn normalize_fixed(&self, source: &str) -> Result<String> {
        let mut normalized = String::new();
        let mut lines: Vec<FixedFormatLine> = Vec::new();
        
        for (idx, line) in source.lines().enumerate() {
            let fixed_line = FixedFormatLine::parse(line, idx + 1);
            lines.push(fixed_line);
        }

        let mut i = 0;
        while i < lines.len() {
            let line = &lines[i];

            if line.is_comment() {
                // Preserve comments
                normalized.push_str(&format!("      * {}\n", line.content()));
                i += 1;
                continue;
            }

            if line.is_debug() {
                // Handle debug lines
                normalized.push_str(&format!("      D {}\n", line.content()));
                i += 1;
                continue;
            }

            // Handle continuation lines
            let mut content = line.content();
            i += 1;

            while i < lines.len() && lines[i].is_continuation() {
                let cont_line = &lines[i];
                let cont_content = cont_line.content();
                // Remove leading/trailing quotes if present for string continuation
                if content.ends_with('"') && cont_content.starts_with('"') {
                    content.pop();
                    content.push_str(&cont_content[1..]);
                } else if content.ends_with('\'') && cont_content.starts_with('\'') {
                    content.pop();
                    content.push_str(&cont_content[1..]);
                } else {
                    content.push_str(&cont_content);
                }
                i += 1;
            }

            if !content.trim().is_empty() {
                normalized.push_str(&content);
                normalized.push('\n');
            }
        }

        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_format_line_parse() {
        let line = "000100 IDENTIFICATION DIVISION.                                  TEST01";
        let parsed = FixedFormatLine::parse(line, 1);
        
        assert_eq!(parsed.sequence, "000100");
        assert_eq!(parsed.indicator, IndicatorChar::None);
        assert!(parsed.content().contains("IDENTIFICATION DIVISION"));
    }

    #[test]
    fn test_comment_line() {
        let line = "000100*This is a comment";
        let parsed = FixedFormatLine::parse(line, 1);
        
        assert!(parsed.is_comment());
    }

    #[test]
    fn test_continuation_line() {
        let line = "000100-    continued text";
        let parsed = FixedFormatLine::parse(line, 1);
        
        assert!(parsed.is_continuation());
    }
}

// Made with Bob
