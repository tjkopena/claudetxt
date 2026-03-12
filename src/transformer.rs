/// Escapes HTML special characters in text.
pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Transforms plain text with basic inline formatting.
/// Handles:
/// - `code` -> <code>code</code>
/// - **bold** -> <strong>bold</strong>
/// - Newlines -> appropriate breaks
pub fn transform_inline(text: &str) -> String {
    let escaped = escape_html(text);
    let mut result = String::with_capacity(escaped.len() * 2);
    let chars: Vec<char> = escaped.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Handle inline code with backticks
        if chars[i] == '`' {
            if let Some(end) = find_matching_char(&chars, i + 1, '`') {
                result.push_str("<code>");
                for c in &chars[i + 1..end] {
                    result.push(*c);
                }
                result.push_str("</code>");
                i = end + 1;
                continue;
            }
        }

        // Handle bold with **
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_double_char(&chars, i + 2, '*') {
                result.push_str("<strong>");
                for c in &chars[i + 2..end] {
                    result.push(*c);
                }
                result.push_str("</strong>");
                i = end + 2;
                continue;
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

fn find_matching_char(chars: &[char], start: usize, target: char) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] == target {
            return Some(i);
        }
        // Don't match across newlines for inline code
        if chars[i] == '\n' {
            return None;
        }
    }
    None
}

fn find_double_char(chars: &[char], start: usize, target: char) -> Option<usize> {
    let mut i = start;
    while i + 1 < chars.len() {
        if chars[i] == target && chars[i + 1] == target {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Detects if a Claude message is a tool call.
/// Matches:
/// - Lines starting with an identifier followed by parenthesized arguments: `Bash(command)`
/// - "Read N file(s) (ctrl+o to expand)" summary lines
/// - "Updated plan"
/// - "Tool loaded."
pub fn is_tool_call(text: &str) -> bool {
    let first_line = text.lines().next().unwrap_or("");

    // Check for identifier followed by opening paren: e.g. "Bash(", "Read("
    let mut chars = first_line.chars();
    if let Some(c) = chars.next() {
        if c.is_ascii_alphabetic() || c == '_' {
            for ch in chars {
                if ch == '(' {
                    return true;
                }
                if !ch.is_ascii_alphanumeric() && ch != '_' {
                    break;
                }
            }
        }
    }

    // Special non-parenthesized patterns
    first_line == "Updated plan"
        || first_line == "Tool loaded."
        || is_read_files_summary(first_line)
        || is_searched_summary(first_line)
}

/// Detects "Read N file(s) (ctrl+o to expand)" summary lines.
fn is_read_files_summary(line: &str) -> bool {
    line.starts_with("Read ")
        && line.ends_with(" (ctrl+o to expand)")
        && line.contains(" file")
}

/// Detects "Searched for N pattern(s)" summary lines.
fn is_searched_summary(line: &str) -> bool {
    line.starts_with("Searched for ")
        && line.contains(" pattern")
}

/// Detects if a tool call is an Update() which produces diff output
pub fn is_update_tool_call(text: &str) -> bool {
    let first_line = text.lines().next().unwrap_or("");
    first_line.contains("Update(")
}

/// Detects if a block of text is a directory structure.
/// Directory structures start with a line ending in '/' and contain tree characters.
fn is_directory_structure(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return false;
    }

    // First line should end with '/'
    let first_line = lines[0].trim();
    if !first_line.ends_with('/') {
        return false;
    }

    // Should contain tree-drawing characters
    text.contains('└') || text.contains('├')
}

/// Renders formatted text content into HTML, handling:
/// - Directory structures (preformatted)
/// - Paragraphs (separated by blank lines)
/// - Numbered lists (1. item)
/// - Bullet lists (- item)
/// - Inline formatting
pub fn render_formatted_text(text: &str) -> String {
    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    let mut html_parts = Vec::new();
    let mut i = 0;

    while i < paragraphs.len() {
        let trimmed = paragraphs[i].trim();
        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        // Check if this paragraph is a directory structure
        if is_directory_structure(trimmed) {
            html_parts.push(render_directory_structure(trimmed));
            i += 1;
            continue;
        }

        // Check if this paragraph is a numbered list item
        if is_numbered_item(trimmed) {
            // Parse all numbered items from this paragraph and any consecutive ones
            let mut all_text = trimmed.to_string();
            i += 1;
            while i < paragraphs.len() {
                let next_trimmed = paragraphs[i].trim();
                if is_numbered_item(next_trimmed) {
                    all_text.push_str("\n\n");
                    all_text.push_str(next_trimmed);
                    i += 1;
                } else {
                    break;
                }
            }
            html_parts.push(render_numbered_list(&all_text));
            continue;
        }
        // Check if this paragraph starts with bullet points
        else if is_bullet_list(trimmed) {
            html_parts.push(render_bullet_list(trimmed));
        }
        // Regular paragraph - may contain inline bullets
        else {
            html_parts.push(render_paragraph(trimmed));
        }
        i += 1;
    }

    html_parts.join("\n")
}

/// Renders a directory structure as preformatted text, styled like tool output but without the pipe.
fn render_directory_structure(text: &str) -> String {
    let escaped = escape_html(text);
    format!(
        "        <div class=\"tool-output\"><pre class=\"tool-result\">{}</pre></div>",
        escaped
    )
}

fn is_numbered_item(text: &str) -> bool {
    let first_line = text.lines().next().unwrap_or("");
    let trimmed = first_line.trim();

    // Check for pattern: digit(s) followed by . or )
    let chars: Vec<char> = trimmed.chars().collect();
    let mut i = 0;
    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }
    i > 0 && i < chars.len() && (chars[i] == '.' || chars[i] == ')')
}

fn is_bullet_list(text: &str) -> bool {
    let first_line = text.lines().next().unwrap_or("");
    let trimmed = first_line.trim();
    trimmed.starts_with("- ") || trimmed.starts_with("* ")
}

/// Renders numbered items as an <ol>.
/// Parses lines to find items starting with "1.", "2.", etc.
fn render_numbered_list(text: &str) -> String {
    let mut items: Vec<String> = Vec::new();
    let mut current_item: Option<String> = None;

    for line in text.lines() {
        let trimmed = line.trim();

        if line_starts_with_number(trimmed) {
            // Save previous item if any
            if let Some(item) = current_item.take() {
                items.push(item);
            }
            // Start new item (strip the number prefix)
            current_item = Some(strip_number_prefix(trimmed).to_string());
        } else if !trimmed.is_empty() && current_item.is_some() {
            // Continuation of current item
            if let Some(ref mut item) = current_item {
                item.push(' ');
                item.push_str(trimmed);
            }
        }
    }

    // Don't forget the last item
    if let Some(item) = current_item {
        items.push(item);
    }

    if items.is_empty() {
        return String::new();
    }

    let mut html = String::from("            <ol>\n");
    for item in items {
        let content = transform_inline(&item);
        html.push_str(&format!("                <li>{}</li>\n", content));
    }
    html.push_str("            </ol>");
    html
}

/// Checks if a line starts with a number followed by . or )
fn line_starts_with_number(line: &str) -> bool {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }
    i > 0 && i < chars.len() && (chars[i] == '.' || chars[i] == ')')
}

/// Strips the number prefix (e.g., "1. " or "2) ") from a line.
fn strip_number_prefix(line: &str) -> &str {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    // Skip digits
    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }

    // Skip . or ) and whitespace
    if i < chars.len() && (chars[i] == '.' || chars[i] == ')') {
        i += 1;
    }
    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }

    // Calculate byte offset
    let byte_offset: usize = line.chars().take(i).map(|c| c.len_utf8()).sum();
    &line[byte_offset..]
}

/// Renders a paragraph that may contain inline bullet points or numbered lists.
fn render_paragraph(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();

    // Check if there are any bullet points or numbered items in the text
    let has_bullets = lines.iter().any(|l| {
        let t = l.trim();
        t.starts_with("- ") || t.starts_with("* ")
    });
    let has_numbered = lines.iter().any(|l| line_starts_with_number(l.trim()));

    if !has_bullets && !has_numbered {
        // Simple paragraph - join lines with spaces (newlines are just console wrapping)
        let joined = text.lines().map(|l| l.trim()).collect::<Vec<_>>().join(" ");
        let content = transform_inline(&joined);
        return format!("            <p>{}</p>", content);
    }

    // Split into intro text and list items
    let mut intro_lines = Vec::new();
    let mut list_lines = Vec::new();
    let mut in_list = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || line_starts_with_number(trimmed) {
            in_list = true;
            list_lines.push(trimmed);
        } else if in_list {
            // Continuation or end of list
            if trimmed.is_empty() {
                break;
            }
            list_lines.push(trimmed);
        } else {
            intro_lines.push(trimmed);
        }
    }

    let mut html = String::new();

    // Render intro if present
    if !intro_lines.is_empty() {
        let intro_text = intro_lines.join(" ");
        let content = transform_inline(&intro_text);
        html.push_str(&format!("            <p>{}</p>\n", content));
    }

    // Render list
    if !list_lines.is_empty() {
        let list_text = list_lines.join("\n");
        if has_numbered {
            html.push_str(&render_numbered_list(&list_text));
        } else {
            html.push_str(&render_bullet_list(&list_text));
        }
    }

    html
}

fn render_bullet_list(text: &str) -> String {
    let mut items = Vec::new();
    let mut current_item: Option<String> = None;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            // New bullet item
            if let Some(item) = current_item.take() {
                items.push(item);
            }
            current_item = Some(trimmed[2..].to_string());
        } else if !trimmed.is_empty() && current_item.is_some() {
            // Continuation of current item
            if let Some(ref mut item) = current_item {
                item.push(' ');
                item.push_str(trimmed);
            }
        }
    }

    if let Some(item) = current_item {
        items.push(item);
    }

    if items.is_empty() {
        return String::new();
    }

    let mut html = String::from("            <ul>\n");
    for item in items {
        let content = transform_inline(&item);
        html.push_str(&format!("                <li>{}</li>\n", content));
    }
    html.push_str("            </ul>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
    }

    #[test]
    fn test_inline_code() {
        assert_eq!(transform_inline("`code`"), "<code>code</code>");
    }

    #[test]
    fn test_bold() {
        assert_eq!(transform_inline("**bold**"), "<strong>bold</strong>");
    }

    #[test]
    fn test_tool_call_detection() {
        assert!(is_tool_call("Search(pattern: \"*.py\")"));
        assert!(is_tool_call("Bash(find /home -name \"*.rs\")"));
        assert!(!is_tool_call("This is regular text"));
    }
}
