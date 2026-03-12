/// Returns the embedded CSS for the HTML output.
/// Optimized for both screen viewing and print to US Letter paper.
pub fn get_css() -> &'static str {
    r#"
:root {
    --text-color: #1a1a1a;
    --bg-color: #ffffff;
    --user-bg: #f7f7f7;
    --user-border: #e0e0e0;
    --claude-text: #1a1a1a;
    --tool-bg: #f5f5f5;
    --tool-border: #e8e8e8;
    --code-bg: #f6f8fa;
    --code-border: #e1e4e8;
    --link-color: #0066cc;
}

* {
    box-sizing: border-box;
}

html {
    font-size: 16px;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    line-height: 1.6;
    color: var(--text-color);
    background-color: var(--bg-color);
    margin: 0;
    padding: 2rem 1rem;
}

.container {
    max-width: 800px;
    margin: 0 auto;
}

.conversation {
    margin-bottom: 2rem;
}

.banner {
    font-family: "SF Mono", "Monaco", "Inconsolata", "Fira Code", monospace;
    font-size: 0.75rem;
    line-height: 1.2;
    background-color: #1a1a1a;
    color: #e0e0e0;
    border-radius: 8px;
    padding: 0.5rem 1rem;
    margin-bottom: 2rem;
    overflow-x: auto;
    white-space: pre;
}

.custom-banner {
    font-size: 400%;
    font-weight: bold;
    margin: 0 0 2rem 0;
    line-height: 1.2;
}

.custom-banner-html {
    margin-bottom: 2rem;
}

.interaction {
    margin: 1rem 0;
}

.interaction ul {
    list-style: none;
    padding-left: 0;
    margin: 0;
}

.interaction li {
    margin-bottom: 1rem;
}

.interaction .question {
    color: var(--text-color);
}

.interaction .answer {
    background-color: var(--user-bg);
    border-left: 4px solid var(--user-border);
    padding: 0.25rem 0.25rem 0.25rem 0.75rem;
    border-radius: 0 8px 8px 0;
    margin-left: 1rem;
    color: #444;
}

.user-prompt {
    background-color: var(--user-bg);
    border-left: 4px solid var(--user-border);
    padding: 1rem 1.25rem;
    margin: 1.5rem 0;
    border-radius: 0 8px 8px 0;
    color: #444;
}

.user-prompt .prompt-marker {
    font-weight: 600;
    margin-right: 0.5rem;
    user-select: none;
}

.claude-message {
    padding: 0;
    margin: 1rem 0 0 0;
}

.claude-message .message-marker {
    color: #666;
    margin-right: 0.5rem;
}

.tool-invocation {
    margin: 1rem 0 0 0;
}

/* Tight spacing when Claude message is followed by tool invocation */
.claude-message + .tool-invocation {
    margin-top: 0;
}

/* Larger spacing when tool output is followed by Claude message */
.tool-output + .claude-message {
    margin-top: 2rem;
}

.tool-bullet {
    color: #666;
    user-select: none;
    margin-right: 0.5rem;
}

.tool-call {
    font-family: "SF Mono", "Monaco", "Inconsolata", "Fira Code", monospace;
    font-size: 0.875rem;
    color: var(--text-color);
    background: none;
    padding: 0;
}

.tool-output {
    display: flex;
    align-items: flex-start;
    margin: 0 0 1rem 1.25rem;
    font-family: "SF Mono", "Monaco", "Inconsolata", "Fira Code", monospace;
    font-size: 0.875rem;
}

.output-pipe {
    color: #888;
    flex-shrink: 0;
    margin-right: 0.5rem;
    user-select: none;
}

.tool-result {
    margin: 0;
    padding: 0;
    font-family: inherit;
    font-size: inherit;
    color: #555;
    background: none;
    border: none;
    white-space: pre-wrap;
    word-wrap: break-word;
    flex: 1;
    min-width: 0;
}

.tool-error {
    color: #e06c75;
}

.diff-output {
    display: flex;
    align-items: flex-start;
    margin: 0 0 1rem 1.25rem;
    font-family: "SF Mono", "Monaco", "Inconsolata", "Fira Code", monospace;
    font-size: 0.875rem;
}

.diff-container {
    flex: 1;
    min-width: 0;
}

.diff-line {
    margin: 0;
    padding: 0 0.25rem;
    white-space: pre-wrap;
    word-wrap: break-word;
}

.diff-add {
    background-color: #d4edda;
}

.diff-del {
    background-color: #f8d7da;
}

.diff-summary {
    color: #555;
    margin-bottom: 0.25rem;
}

.usage-bullet {
    user-select: none;
    margin-right: 0.5rem;
}

pre, code {
    font-family: "SF Mono", "Monaco", "Inconsolata", "Fira Code", monospace;
    font-size: 0.875rem;
}

pre {
    background-color: var(--code-bg);
    border: 1px solid var(--code-border);
    border-radius: 6px;
    padding: 1rem;
    overflow-x: auto;
    white-space: pre-wrap;
    word-wrap: break-word;
}

code {
    background-color: var(--code-bg);
    padding: 0.2em 0.4em;
    border-radius: 3px;
}

pre code {
    background: none;
    padding: 0;
}

p {
    margin: 0.75rem 0;
}

ol, ul {
    margin: 0.75rem 0;
    padding-left: 1.5rem;
}

.claude-message ul,
.claude-message ol {
    margin-top: 0;
}

li {
    margin: 0.25rem 0;
}

h1, h2, h3, h4, h5, h6 {
    margin: 1.5rem 0 0.75rem 0;
    line-height: 1.3;
}

a {
    color: var(--link-color);
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

/* Print styles */
@media print {
    html {
        font-size: 11pt;
    }

    body {
        padding: 0;
        background: white;
    }

    .container {
        max-width: none;
    }

    .user-prompt {
        break-inside: avoid;
        page-break-inside: avoid;
        background-color: #f5f5f5 !important;
        -webkit-print-color-adjust: exact;
        print-color-adjust: exact;
    }

    .tool-output {
        break-inside: avoid;
        page-break-inside: avoid;
    }

    .tool-result {
        background: none !important;
    }

    .tool-error {
        color: #e06c75 !important;
        -webkit-print-color-adjust: exact;
        print-color-adjust: exact;
    }

    .diff-add {
        background-color: #d4edda !important;
        -webkit-print-color-adjust: exact;
        print-color-adjust: exact;
    }

    .diff-del {
        background-color: #f8d7da !important;
        -webkit-print-color-adjust: exact;
        print-color-adjust: exact;
    }

    .diff-output {
        break-inside: avoid;
        page-break-inside: avoid;
    }

    pre {
        break-inside: avoid;
        page-break-inside: avoid;
        background-color: #f5f5f5 !important;
        -webkit-print-color-adjust: exact;
        print-color-adjust: exact;
    }

    .claude-message {
        break-inside: avoid;
        page-break-inside: avoid;
    }

    /* Page setup for US Letter */
    @page {
        size: letter;
        margin: 0.75in;
    }

    .banner {
        background-color: #1a1a1a !important;
        color: #e0e0e0 !important;
        -webkit-print-color-adjust: exact;
        print-color-adjust: exact;
        page-break-inside: avoid;
    }

    .tool-invocation, .tool-output {
        page-break-inside: avoid;
    }
}
"#
}
