use std::fs;
use std::path::{Path, PathBuf};

pub fn find_react_files(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    visit(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|name| name.to_str());
            if matches!(name, Some("node_modules" | ".git" | "target")) {
                continue;
            }
            visit(&path, files)?;
        } else if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("tsx" | "jsx")
        ) {
            files.push(path);
        }
    }
    Ok(())
}

pub fn validate_target(path: &Path) -> Result<(), String> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("tsx" | "jsx") => Ok(()),
        _ => Err("target file must be .tsx or .jsx".to_string()),
    }
}

pub fn read_target(path: &Path) -> Result<String, String> {
    validate_target(path)?;
    if !path.exists() {
        return Ok(String::new());
    }
    let target = fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    Ok(read_source(&target).unwrap_or(target))
}

pub fn write_blog(path: &Path, markdown: &str) -> Result<(), String> {
    validate_target(path)?;
    let component = crate::renderer::render_blog(markdown);
    let source = encode_source(markdown);
    let output =
        format!("{component}\n\n// panbahar-source:start\n{source}// panbahar-source:end\n");
    fs::write(path, output).map_err(|error| format!("could not write {}: {error}", path.display()))
}

const SOURCE_START: &str = "// panbahar-source:start";
const SOURCE_LINE: &str = "// panbahar-source:";
const SOURCE_END: &str = "// panbahar-source:end";

fn encode_source(source: &str) -> String {
    let mut encoded = String::new();
    for byte in source.bytes() {
        encoded.push_str(&format!("{byte:02x}"));
    }
    format!("{SOURCE_LINE}{encoded}\n")
}

fn read_source(target: &str) -> Option<String> {
    let start = target.find(SOURCE_START)?;
    let lines = target[start + SOURCE_START.len()..].lines();
    let mut hex = String::new();
    for line in lines {
        if line == SOURCE_END {
            break;
        }
        if let Some(value) = line.strip_prefix(SOURCE_LINE) {
            hex.push_str(value);
        }
    }
    if hex.is_empty() {
        return Some(String::new());
    }

    let bytes: Option<Vec<u8>> = (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(hex.get(index..index + 2)?, 16).ok())
        .collect();
    String::from_utf8(bytes?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn reopen_shows_written_source_instead_of_generated_jsx() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("panbahar-{unique}.tsx"));

        write_blog(&path, "# Hello\n\nA **bold** paragraph.").unwrap();
        assert_eq!(
            read_target(&path).unwrap(),
            "# Hello\n\nA **bold** paragraph."
        );

        let target = fs::read_to_string(&path).unwrap();
        assert!(target.contains("export default function Blog()"));
        assert!(target.contains("<h1>Hello</h1>"));
        assert_eq!(
            read_source(&target).unwrap(),
            "# Hello\n\nA **bold** paragraph."
        );

        let _ = fs::remove_file(path);
    }
}
