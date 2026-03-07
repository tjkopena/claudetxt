# claudetxt

Convert Claude Code TUI `/export` files to standalone HTML documents.

## Overview

The Claude Code terminal interface's `/export` files have multiple
uses. The motivating one for `claudetxt` is archiving the actual
textual session for future (human) reference.  This might be done
before ending the session or prior to a context compaction. Having the
session captured in this way might be useful to:

- Review design knowledge and intent implicit in the prompts.
- Review and analyze decisions and the Claude inputs behind them.
- Present to others for design/code review or pedagogical reasons.

`claudetxt` parses the syntactically-structured console `/export` text
and generates a self-contained HTML file presenting the captured
conversation in a lightly styled way, including:

- Welcome banners with box-drawn characters
- User prompts and Claude responses
- Tool invocations and their output (with error highlighting)
- Diff output with color-coded additions and deletions
- Interactive Q&A blocks and plan approvals
- Numbered and bulleted lists
- Directory tree structures

In addition to direct viewing the output is intended to be suitable
for conversion to PDF via a tool such as weasyprint or simply printing
to file from a browser.

The output HTML includes the full style definition, it has no external
dependencies so it can be easily shared.

There is probably a more complete semantic session capture or plugin
or something to do this better, but `claudetxt` is simple & hopefully
useful.  If you encounter some `/export` it does not parse or style
well please post a ticket.


## Installation

```sh
cargo build --release
```

The binary will be at `target/release/claudetxt`.

## Usage

```sh
# Output to stdout
claudetxt conversation.txt

# Output to a file
claudetxt conversation.txt -o conversation.html
```

### Options

| Flag | Description |
|---|---|
| `--banner <TEXT>` | Override the banner with custom text or HTML |
| `--username <NAME>` | Override the username displayed in user prompts |
| `--title <TITLE>` | Set the HTML document title |
| `--nocolor` | Suppress colored backgrounds in diff output |

### Examples

```sh
# Custom title and username
claudetxt export.txt -o output.html --title "Debug Session" --username "Alice"

# Replace the banner with plain text
claudetxt export.txt -o output.html --banner "Project Review"

# Replace the banner with custom HTML
claudetxt export.txt -o output.html --banner '<h1 style="color: blue;">My Project</h1>'

# Disable diff colors (useful for printing)
claudetxt export.txt -o output.html --nocolor
```

## How It Works

1. **Parsing** (`parser.rs`) — Splits the export file into typed
   blocks (`Banner`, `UserPrompt`, `ClaudeMessage`, `ToolOutput`) by
   recognizing Unicode markers (`╭`, `❯`, `●`, `⎿`).
2. **Transformation** (`transformer.rs`) — Handles inline formatting,
   tool call detection, and structured content rendering (lists,
   directory trees).
3. **HTML generation** (`html.rs`) — Assembles the blocks into a
   complete HTML document, with special handling for diffs, Q&A
   interactions, and plan approvals.
4. **Styling** (`styles.rs`) — Provides embedded CSS for both screen
   and print output.

## License

This project is dedicated to the public domain under the [CC0 1.0
Universal](https://creativecommons.org/publicdomain/zero/1.0/)
license. To the extent possible under law, the author has waived all
copyright and related or neighboring rights to this work.
