extern crate proc_macro;

use proc_macro::TokenStream;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn resolve_includes(current_file: &Path, files_visited: &mut Vec<PathBuf>) -> String {
    let content =
        fs::read_to_string(current_file).unwrap_or_else(|_| panic!("Failed to read {:?}", current_file));
    files_visited.push(current_file.to_path_buf());

    let mut resolved_content = String::new();
    let parent_dir = current_file.parent().unwrap();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//!include ") {
            let include_path_str = trimmed.trim_start_matches("//!include ").trim();
            let include_path = parent_dir.join(include_path_str);
            let included_content = resolve_includes(&include_path, files_visited);
            resolved_content.push_str(&included_content);
            resolved_content.push('\n');
        } else {
            resolved_content.push_str(line);
            resolved_content.push('\n');
        }
    }
    resolved_content
}

#[proc_macro]
pub fn include_wgsl(input: TokenStream) -> TokenStream {
    // The input is a string literal. Extract its value.
    let input_str = input.into_iter().next().expect("Expected string literal").to_string();
    let relative_path = input_str.trim_matches('"');

    // Resolve relative to the calling crate's manifest dir
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let base_path = Path::new(&manifest_dir).join(relative_path);

    let mut files_visited = Vec::new();
    let resolved_content = resolve_includes(&base_path, &mut files_visited);

    let mut output = String::new();
    output.push_str("{\n");
    for file in files_visited {
        let file_str = file.to_str().unwrap().replace("\\", "/");
        output.push_str(&format!("    const _: &[u8] = include_bytes!(r#\"{}\"#);\n", file_str));
    }
    // Output the fully resolved shader
    output.push_str(&format!("    r#\"{}\"#\n", resolved_content));
    output.push_str("}\n");

    output.parse().expect("Failed to parse generated TokenStream")
}
