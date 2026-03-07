mod html;
mod parser;
mod styles;
mod transformer;

use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Convert Claude Code /export files to standalone HTML
#[derive(Parser)]
#[command(name = "claudetxt")]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file (Claude Code /export markdown file)
    input: PathBuf,

    /// Output file (defaults to stdout if not specified)
    output: Option<PathBuf>,

    /// Override the banner with custom text or HTML
    #[arg(long)]
    banner: Option<String>,

    /// Override the username displayed in user prompts
    #[arg(long)]
    username: Option<String>,

    /// Set the HTML document title
    #[arg(long)]
    title: Option<String>,

    /// Suppress colored backgrounds in diff output
    #[arg(long)]
    nocolor: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Read input file
    let input_content = fs::read_to_string(&args.input).map_err(|e| {
        format!("Failed to read input file '{}': {}", args.input.display(), e)
    })?;

    // Parse the export format
    let blocks = parser::parse(&input_content);

    // Determine username: CLI arg > banner extraction > default "User"
    let username = args.username.unwrap_or_else(|| {
        // Try to extract from banner
        for block in &blocks {
            if let parser::Block::Banner(banner_text) = block {
                if let Some(name) = parser::extract_username(banner_text) {
                    return name;
                }
            }
        }
        "User".to_string()
    });

    // Determine title: CLI arg > default pattern
    let title = args.title.unwrap_or_else(|| {
        let filename = args.input.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("conversation");
        format!("Claude Code: {} ({})", filename, username)
    });

    // Generate HTML
    let html_output = html::generate_html(&blocks, args.banner.as_deref(), &username, &title, args.nocolor);

    // Write output
    match args.output {
        Some(output_path) => {
            fs::write(&output_path, &html_output).map_err(|e| {
                format!("Failed to write output file '{}': {}", output_path.display(), e)
            })?;
            eprintln!("Wrote HTML to {}", output_path.display());
        }
        None => {
            io::stdout().write_all(html_output.as_bytes())?;
        }
    }

    Ok(())
}
