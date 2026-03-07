/// Represents different types of content blocks in a Claude Code export.
#[derive(Debug, Clone)]
pub enum Block {
    /// The welcome banner with box-drawing characters
    Banner(String),
    /// User input (lines starting with ❯)
    UserPrompt(String),
    /// Claude's message or action (lines starting with ●)
    ClaudeMessage(String),
    /// Tool output continuation (lines with ⎿)
    ToolOutput(String),
    /// Empty lines for spacing
    Empty,
}

/// Extracts the username from a banner block.
/// Looks for pattern "Welcome back {username}!" and returns the username.
/// Returns None if no username is found.
pub fn extract_username(banner: &str) -> Option<String> {
    // Look for "Welcome back " followed by username and "!"
    if let Some(start) = banner.find("Welcome back ") {
        let after_welcome = &banner[start + 13..]; // len("Welcome back ") = 13
        if let Some(end) = after_welcome.find('!') {
            let username = after_welcome[..end].trim();
            if !username.is_empty() {
                return Some(username.to_string());
            }
        }
    }
    None
}

/// Parses a Claude Code export file into structured blocks.
pub fn parse(input: &str) -> Vec<Block> {
    let lines: Vec<&str> = input.lines().collect();
    let mut blocks = Vec::new();
    let mut i = 0;
    let mut banner_checked = false;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Check for banner start (box-drawing top corner) - only at start of input
        if !banner_checked && (trimmed.starts_with('╭') || line.contains('╭')) {
            banner_checked = true;
            let mut banner_lines = vec![line.to_string()];
            i += 1;
            // Collect until we hit the bottom of the box
            while i < lines.len() {
                let l = lines[i];
                banner_lines.push(l.to_string());
                if l.contains('╰') {
                    i += 1;
                    break;
                }
                i += 1;
            }
            blocks.push(Block::Banner(banner_lines.join("\n")));
            continue;
        }

        // Mark banner as checked once we see any non-empty content
        if !trimmed.is_empty() {
            banner_checked = true;
        }

        // Check for user prompt (❯)
        if trimmed.starts_with('❯') {
            let prompt_text = trimmed.trim_start_matches('❯').trim();
            let mut full_prompt = prompt_text.to_string();
            i += 1;
            // Collect continuation lines (indented, not starting with special chars)
            while i < lines.len() {
                let l = lines[i];
                let lt = l.trim();
                if lt.is_empty()
                    || lt.starts_with('●')
                    || lt.starts_with('❯')
                    || lt.starts_with('⎿')
                    || lt.starts_with('╭')
                {
                    break;
                }
                // Check if it's a continuation (has leading whitespace in original)
                if l.starts_with(' ') || l.starts_with('\t') {
                    full_prompt.push(' ');
                    full_prompt.push_str(lt);
                    i += 1;
                } else {
                    break;
                }
            }
            blocks.push(Block::UserPrompt(full_prompt.trim().to_string()));
            continue;
        }

        // Check for Claude message (●)
        if trimmed.starts_with('●') {
            let message_text = trimmed.trim_start_matches('●').trim();
            let mut full_message = message_text.to_string();
            i += 1;
            // Collect continuation lines
            while i < lines.len() {
                let l = lines[i];
                let lt = l.trim();
                if lt.is_empty() {
                    // Empty line might be paragraph break within message
                    // Look ahead to see if content continues
                    if i + 1 < lines.len() {
                        let next = lines[i + 1].trim();
                        if !next.starts_with('●')
                            && !next.starts_with('❯')
                            && !next.starts_with('⎿')
                            && !next.starts_with('╭')
                            && !next.is_empty()
                        {
                            full_message.push_str("\n\n");
                            i += 1;
                            continue;
                        }
                    }
                    break;
                }
                if lt.starts_with('●')
                    || lt.starts_with('❯')
                    || lt.starts_with('⎿')
                    || lt.starts_with('╭')
                {
                    break;
                }
                // Continuation line
                if l.starts_with(' ') || l.starts_with('\t') {
                    full_message.push('\n');
                    full_message.push_str(lt);
                    i += 1;
                } else {
                    break;
                }
            }
            blocks.push(Block::ClaudeMessage(full_message.trim().to_string()));
            continue;
        }

        // Check for tool output (⎿)
        if trimmed.starts_with('⎿') {
            let output_text = trimmed.trim_start_matches('⎿').trim();
            let mut continuation_lines: Vec<&str> = Vec::new();
            i += 1;
            // Collect continuation lines, including across blank lines
            while i < lines.len() {
                let l = lines[i];
                let lt = l.trim();

                // Check for new message markers
                if lt.starts_with('●')
                    || lt.starts_with('❯')
                    || lt.starts_with('╭')
                    || lt.starts_with('⎿')
                {
                    break;
                }

                // Handle blank lines - look ahead to see if content continues
                if lt.is_empty() {
                    // Look ahead for more indented content
                    let mut has_more_content = false;
                    for j in (i + 1)..lines.len() {
                        let future = lines[j].trim();
                        if future.starts_with('●')
                            || future.starts_with('❯')
                            || future.starts_with('╭')
                            || future.starts_with('⎿')
                        {
                            break;
                        }
                        if !future.is_empty() {
                            // Check if it's indented (part of this tool output)
                            if lines[j].starts_with(' ') || lines[j].starts_with('\t') {
                                has_more_content = true;
                            }
                            break;
                        }
                    }
                    if has_more_content {
                        continuation_lines.push(l); // Keep the blank line
                        i += 1;
                        continue;
                    } else {
                        break;
                    }
                }

                if l.starts_with(' ') || l.starts_with('\t') {
                    continuation_lines.push(l);
                    i += 1;
                } else {
                    break;
                }
            }

            // Find minimum indentation across continuation lines
            let min_indent = continuation_lines
                .iter()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.len() - l.trim_start().len())
                .min()
                .unwrap_or(0);

            // Build output with normalized indentation
            let mut full_output = output_text.to_string();
            for cont_line in continuation_lines {
                full_output.push('\n');
                let trimmed_end = cont_line.trim_end();
                if trimmed_end.len() >= min_indent {
                    full_output.push_str(&trimmed_end[min_indent..]);
                } else {
                    full_output.push_str(trimmed_end.trim_start());
                }
            }

            blocks.push(Block::ToolOutput(full_output.trim_end().to_string()));
            continue;
        }

        // Empty lines
        if trimmed.is_empty() {
            blocks.push(Block::Empty);
            i += 1;
            continue;
        }

        // Skip other lines (likely trailing whitespace or noise)
        i += 1;
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_user_prompt() {
        let input = "❯ Hello world\n  continuation\n";
        let blocks = parse(input);
        assert!(matches!(blocks.first(), Some(Block::UserPrompt(_))));
    }

    #[test]
    fn test_parse_claude_message() {
        let input = "● This is a response\n  with more text\n";
        let blocks = parse(input);
        assert!(matches!(blocks.first(), Some(Block::ClaudeMessage(_))));
    }

    #[test]
    fn test_extract_username() {
        let banner = "Welcome back tjkopena!";
        assert_eq!(extract_username(banner), Some("tjkopena".to_string()));
    }

    #[test]
    fn test_extract_username_not_found() {
        let banner = "Some other text";
        assert_eq!(extract_username(banner), None);
    }
}
