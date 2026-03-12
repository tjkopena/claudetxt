use crate::parser::Block;
use crate::styles::get_css;
use crate::transformer::{escape_html, is_tool_call, is_update_tool_call, render_formatted_text};

/// Generates a complete standalone HTML document from parsed blocks.
/// `custom_banner` overrides the banner: Some(text) uses custom text, None uses default from input.
/// `username` is displayed as the prompt marker for user messages.
/// `title` is used as the HTML document title.
/// `nocolor` suppresses colored backgrounds in diff output.
pub fn generate_html(blocks: &[Block], custom_banner: Option<&str>, username: &str, title: &str, nocolor: bool) -> String {
    let mut body = String::new();

    // If custom banner is provided, render it first
    if let Some(banner_text) = custom_banner {
        body.push_str(&render_custom_banner(banner_text));
    }

    let mut i = 0;
    while i < blocks.len() {
        match &blocks[i] {
            Block::Banner(text) => {
                // Only render default banner if no custom banner was provided
                if custom_banner.is_none() {
                    body.push_str(&render_banner(text));
                }
            }
            Block::UserPrompt(text) => {
                body.push_str(&render_user_prompt(text, username));
            }
            Block::ClaudeMessage(text) => {
                // Check for interaction pattern: "User answered Claude's questions:"
                if text.starts_with("User answered Claude's questions:") {
                    // Look ahead for the tool output containing the Q&A pairs
                    if let Some(Block::ToolOutput(qa_text)) = blocks.get(i + 1) {
                        body.push_str(&render_interaction(qa_text, username));
                        i += 2; // Skip both the message and tool output
                        continue;
                    }
                }
                // Check for approval pattern: "User approved Claude's plan"
                if text.starts_with("User approved Claude's plan") {
                    // Look ahead for the tool output containing plan details
                    if let Some(Block::ToolOutput(details)) = blocks.get(i + 1) {
                        body.push_str(&render_approval(text, details));
                        i += 2; // Skip both the message and tool output
                        continue;
                    }
                }
                // Check for Update() tool call which produces diff output
                if is_update_tool_call(text) {
                    body.push_str(&render_claude_message(text));
                    // Look ahead for the tool output containing the diff
                    if let Some(Block::ToolOutput(diff_text)) = blocks.get(i + 1) {
                        body.push_str(&render_diff_output(diff_text, nocolor));
                        i += 2; // Skip both the message and tool output
                        continue;
                    }
                }
                body.push_str(&render_claude_message(text));
            }
            Block::ToolOutput(text) => {
                body.push_str(&render_tool_output(text, nocolor));
            }
            Block::Empty => {
                // Skip consecutive empty blocks
            }
            Block::UsageReport(text) => {
                let escaped = escape_html(text);
                body.push_str(&format!(
                    "        <div class=\"usage-report\"><span class=\"usage-bullet\">✻</span><em>{}</em></div>\n",
                    escaped
                ));
            }
            Block::Text(text) => {
                let escaped = escape_html(text);
                body.push_str(&format!(
                    "        <div class=\"text\">{}</div>\n",
                    escaped
                ));
            }
        }
        i += 1;
    }

    let escaped_title = escape_html(title);
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
{css}
    </style>
</head>
<body>
    <div class="container">
        <div class="conversation">
{body}
        </div>
    </div>
</body>
</html>"#,
        title = escaped_title,
        css = get_css(),
        body = body
    )
}

fn render_user_prompt(text: &str, username: &str) -> String {
    let escaped = escape_html(text);
    let escaped_username = escape_html(username);
    format!(
        r#"        <div class="user-prompt"><span class="prompt-marker">{}:</span><span class="prompt-text">{}</span></div>
"#,
        escaped_username, escaped
    )
}

fn render_claude_message(text: &str) -> String {
    // Check if this is a tool call
    if is_tool_call(text) {
        let escaped = escape_html(text);
        return format!(
            r#"        <div class="tool-invocation"><span class="tool-bullet">●</span><code class="tool-call">{}</code></div>
"#,
            escaped
        );
    }

    // Render formatted text with proper structure
    let html_content = render_formatted_text(text);

    format!(
        r#"        <div class="claude-message">
{}
        </div>
"#,
        html_content
    )
}

fn render_tool_output(text: &str, nocolor: bool) -> String {
    let escaped = escape_html(text);
    let is_error = text.trim().starts_with("Error");
    let class = if is_error && !nocolor {
        "tool-result tool-error"
    } else {
        "tool-result"
    };
    format!(
        r#"        <div class="tool-output"><span class="output-pipe">⎿</span><pre class="{}">{}</pre></div>
"#,
        class, escaped
    )
}

/// Renders diff output from an Update() tool call.
/// Parses lines to identify additions (+) and deletions (-) and styles them accordingly.
/// If `nocolor` is true, suppresses the colored backgrounds.
fn render_diff_output(text: &str, nocolor: bool) -> String {
    let mut html = String::from(
        "        <div class=\"diff-output\"><span class=\"output-pipe\">⎿</span><div class=\"diff-container\">\n",
    );

    let lines: Vec<&str> = text.lines().collect();
    let mut current_type: Option<DiffLineType> = None;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // First line is typically the summary
        if idx == 0 && (trimmed.starts_with("Added") || trimmed.starts_with("Removed")) {
            html.push_str(&format!(
                "            <div class=\"diff-line diff-summary\">{}</div>\n",
                render_diff_summary(trimmed, nocolor)
            ));
            continue;
        }

        // Parse the line to determine its type (use trimmed for parsing)
        let (line_type, _) = parse_diff_line(trimmed, current_type);
        current_type = Some(line_type);

        let class = if nocolor {
            "diff-line"
        } else {
            match line_type {
                DiffLineType::Addition => "diff-line diff-add",
                DiffLineType::Deletion => "diff-line diff-del",
                DiffLineType::Context => "diff-line",
            }
        };

        // Use original line to preserve indentation for continuation lines
        html.push_str(&format!(
            "            <div class=\"{}\">{}</div>\n",
            class,
            escape_html(line)
        ));
    }

    html.push_str("        </div></div>\n");
    html
}

/// Renders the diff summary line with colored parts.
/// "Added X lines" is colored green, "removed Y lines" is colored red.
/// If `nocolor` is true, returns plain escaped text without color spans.
fn render_diff_summary(text: &str, nocolor: bool) -> String {
    if nocolor {
        return escape_html(text);
    }

    let mut result = String::new();
    let lower = text.to_lowercase();

    // Find "added X line(s)" pattern
    if let Some(added_start) = lower.find("added") {
        // Find end of this phrase (look for comma or end)
        let after_added = &text[added_start..];
        let added_end = after_added.find(',').unwrap_or(after_added.len());
        let added_phrase = &text[added_start..added_start + added_end];

        // Add any text before "added"
        if added_start > 0 {
            result.push_str(&escape_html(&text[..added_start]));
        }

        // Add the "added" part with green styling
        result.push_str(&format!(
            "<span class=\"diff-add\">{}</span>",
            escape_html(added_phrase)
        ));

        // Process the rest
        let remaining_start = added_start + added_end;
        if remaining_start < text.len() {
            let remaining = &text[remaining_start..];
            let remaining_lower = remaining.to_lowercase();

            // Look for "removed X line(s)" in remaining text
            if let Some(removed_pos) = remaining_lower.find("removed") {
                // Add text between (like ", ")
                if removed_pos > 0 {
                    result.push_str(&escape_html(&remaining[..removed_pos]));
                }

                // Add the "removed" part with red styling
                let removed_phrase = &remaining[removed_pos..];
                result.push_str(&format!(
                    "<span class=\"diff-del\">{}</span>",
                    escape_html(removed_phrase)
                ));
            } else {
                result.push_str(&escape_html(remaining));
            }
        }
    } else if let Some(removed_start) = lower.find("removed") {
        // Only "removed", no "added"
        if removed_start > 0 {
            result.push_str(&escape_html(&text[..removed_start]));
        }
        let removed_phrase = &text[removed_start..];
        result.push_str(&format!(
            "<span class=\"diff-del\">{}</span>",
            escape_html(removed_phrase)
        ));
    } else {
        // No recognized pattern, just escape
        result.push_str(&escape_html(text));
    }

    result
}

#[derive(Clone, Copy)]
enum DiffLineType {
    Addition,
    Deletion,
    Context,
}

/// Parses a diff line to determine its type.
/// Lines starting with a number followed by " +" are additions.
/// Lines starting with a number followed by " -" are deletions.
/// Lines without a number inherit the type from the previous line.
fn parse_diff_line(line: &str, previous_type: Option<DiffLineType>) -> (DiffLineType, &str) {
    let chars: Vec<char> = line.chars().collect();

    // Skip leading whitespace and find where digits start
    let mut i = 0;
    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }

    // Check if line starts with digits (line number)
    let digit_start = i;
    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }

    let has_line_number = i > digit_start;

    if has_line_number {
        // Skip whitespace after line number
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        // Check for +/- marker
        if i < chars.len() {
            if chars[i] == '+' {
                return (DiffLineType::Addition, line);
            } else if chars[i] == '-' {
                return (DiffLineType::Deletion, line);
            }
        }
        // No marker means context
        (DiffLineType::Context, line)
    } else {
        // No line number - check for +/- at the position where it would be
        // Skip leading whitespace
        let mut j = 0;
        while j < chars.len() && chars[j].is_whitespace() {
            j += 1;
        }

        if j < chars.len() {
            if chars[j] == '+' {
                return (DiffLineType::Addition, line);
            } else if chars[j] == '-' {
                return (DiffLineType::Deletion, line);
            }
        }

        // Inherit from previous line, or default to context
        (previous_type.unwrap_or(DiffLineType::Context), line)
    }
}

fn render_banner(text: &str) -> String {
    let escaped = escape_html(text);
    format!(
        r#"        <pre class="banner">{}</pre>
"#,
        escaped
    )
}

/// Renders an interaction block (User answered Claude's questions).
/// Parses Q&A pairs marked with · and separated by →.
fn render_interaction(text: &str, username: &str) -> String {
    let mut html = String::from("        <div class=\"interaction\">\n            <ul>\n");

    // Split by · to get individual Q&A pairs
    // The text may have the · at the start of lines
    let mut current_qa = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('·') {
            // Process previous Q&A if any
            if !current_qa.is_empty() {
                if let Some((question, answer)) = parse_qa_pair(&current_qa) {
                    html.push_str(&format_qa_item(&question, &answer, username));
                }
            }
            // Start new Q&A (remove the · prefix)
            current_qa = trimmed.trim_start_matches('·').trim().to_string();
        } else if !trimmed.is_empty() {
            // Continuation of current Q&A
            if !current_qa.is_empty() {
                current_qa.push(' ');
            }
            current_qa.push_str(trimmed);
        }
    }

    // Process last Q&A
    if !current_qa.is_empty() {
        if let Some((question, answer)) = parse_qa_pair(&current_qa) {
            html.push_str(&format_qa_item(&question, &answer, username));
        }
    }

    html.push_str("            </ul>\n        </div>\n");
    html
}

/// Parses a Q&A pair separated by →.
fn parse_qa_pair(text: &str) -> Option<(String, String)> {
    if let Some(arrow_pos) = text.find('→') {
        let question = text[..arrow_pos].trim().to_string();
        let answer = text[arrow_pos + '→'.len_utf8()..].trim().to_string();
        Some((question, answer))
    } else {
        None
    }
}

/// Formats a Q&A pair as an HTML list item.
fn format_qa_item(question: &str, answer: &str, username: &str) -> String {
    let escaped_q = escape_html(question);
    let escaped_a = escape_html(answer);
    let escaped_u = escape_html(username);
    format!(
        "                <li><div class=\"question\">{}</div><div class=\"answer\"><strong>{} ➡</strong> {}</div></li>\n",
        escaped_q, escaped_u, escaped_a
    )
}

/// Renders an approval block (User approved Claude's plan).
/// The message is rendered as a tool invocation and the full details as its tool output.
fn render_approval(message: &str, details: &str) -> String {
    let escaped_message = escape_html(message);
    let escaped_details = escape_html(details);

    format!(
        r#"        <div class="tool-invocation"><span class="tool-bullet">●</span><code class="tool-call">{}</code></div>
        <div class="tool-output"><span class="output-pipe">⎿</span><pre class="tool-result">{}</pre></div>
"#,
        escaped_message, escaped_details
    )
}

/// Checks if text appears to be simple (no HTML tags).
fn is_simple_text(text: &str) -> bool {
    !text.contains('<') || !text.contains('>')
}

/// Renders a custom banner. If simple text, wraps in styled h1; otherwise uses as-is.
fn render_custom_banner(text: &str) -> String {
    if is_simple_text(text) {
        let escaped = escape_html(text);
        format!(
            r#"        <h1 class="custom-banner">{}</h1>
"#,
            escaped
        )
    } else {
        // Contains HTML, use as-is
        format!(
            r#"        <div class="custom-banner-html">{}</div>
"#,
            text
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_html_structure() {
        let blocks = vec![Block::UserPrompt("Hello".to_string())];
        let html = generate_html(&blocks, None, "User", "Test Title", false);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<div class=\"user-prompt\">"));
    }

    #[test]
    fn test_default_banner() {
        let blocks = vec![
            Block::Banner("╭───╮".to_string()),
            Block::UserPrompt("Hello".to_string()),
        ];
        let html = generate_html(&blocks, None, "User", "Test Title", false);
        assert!(html.contains("<pre class=\"banner\">"));
        assert!(html.contains("╭───╮"));
    }

    #[test]
    fn test_custom_banner_simple() {
        let blocks = vec![
            Block::Banner("╭───╮".to_string()),
            Block::UserPrompt("Hello".to_string()),
        ];
        let html = generate_html(&blocks, Some("My Project"), "User", "Test Title", false);
        assert!(html.contains("<h1 class=\"custom-banner\">My Project</h1>"));
        assert!(!html.contains("╭───╮"));
    }

    #[test]
    fn test_custom_banner_html() {
        let blocks = vec![Block::UserPrompt("Hello".to_string())];
        let html = generate_html(&blocks, Some("<strong>Bold</strong>"), "User", "Test Title", false);
        assert!(html.contains("<div class=\"custom-banner-html\"><strong>Bold</strong></div>"));
    }

    #[test]
    fn test_username_in_prompt() {
        let blocks = vec![Block::UserPrompt("Hello".to_string())];
        let html = generate_html(&blocks, None, "tjkopena", "Test Title", false);
        assert!(html.contains("tjkopena:"));
    }

    #[test]
    fn test_custom_title() {
        let blocks = vec![Block::UserPrompt("Hello".to_string())];
        let html = generate_html(&blocks, None, "User", "My Custom Title", false);
        assert!(html.contains("<title>My Custom Title</title>"));
    }
}
