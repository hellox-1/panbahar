fn escape_jsx(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('{', "&#123;")
        .replace('}', "&#125;")
}

fn inline_jsx(text: &str) -> String {
    let mut result = String::new();
    let mut rest = text;

    while let Some(start) = rest.find("**") {
        result.push_str(&escape_jsx(&rest[..start]));
        rest = &rest[start + 2..];
        if let Some(end) = rest.find("**") {
            result.push_str("<strong>");
            result.push_str(&escape_jsx(&rest[..end]));
            result.push_str("</strong>");
            rest = &rest[end + 2..];
        } else {
            result.push_str("**");
        }
    }
    result.push_str(&escape_jsx(rest));
    result
}

pub fn render_blog(markdown: &str) -> String {
    let mut output = String::from("export default function Blog() {\n");
    output.push_str("  return (\n");
    output.push_str("    <article className=\"panbahar-blog\">\n");

    let mut in_list = false;
    let mut in_code = false;
    let mut code = String::new();

    for line in markdown.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            if in_code {
                output.push_str("      <pre className=\"panbahar-code\"><code>");
                output.push_str(&escape_jsx(&code));
                output.push_str("</code></pre>\n");
                code.clear();
                in_code = false;
            } else {
                if in_list {
                    output.push_str("      </ul>\n");
                    in_list = false;
                }
                in_code = true;
            }
            continue;
        }

        if in_code {
            if !code.is_empty() {
                code.push('\n');
            }
            code.push_str(line);
            continue;
        }

        if trimmed.is_empty() {
            if in_list {
                output.push_str("      </ul>\n");
                in_list = false;
            }
            continue;
        }

        if trimmed
            .strip_prefix("# ")
            .or_else(|| trimmed.strip_prefix("## "))
            .or_else(|| trimmed.strip_prefix("### "))
            .is_some()
        {
            let (tag, text) = if let Some(text) = trimmed.strip_prefix("# ") {
                ("h1", text)
            } else if let Some(text) = trimmed.strip_prefix("## ") {
                ("h2", text)
            } else {
                ("h3", trimmed.strip_prefix("### ").unwrap())
            };
            if in_list {
                output.push_str("      </ul>\n");
                in_list = false;
            }
            output.push_str(&format!("      <{tag}>{}</{tag}>\n", inline_jsx(text)));
        } else if let Some(text) = trimmed.strip_prefix("- ") {
            if !in_list {
                output.push_str("      <ul>\n");
                in_list = true;
            }
            output.push_str("        <li>");
            output.push_str(&inline_jsx(text));
            output.push_str("</li>\n");
        } else {
            if in_list {
                output.push_str("      </ul>\n");
                in_list = false;
            }
            output.push_str("      <p>");
            output.push_str(&inline_jsx(trimmed));
            output.push_str("</p>\n");
        }
    }

    if in_list {
        output.push_str("      </ul>\n");
    }
    if in_code {
        output.push_str("      <pre className=\"panbahar-code\"><code>");
        output.push_str(&escape_jsx(&code));
        output.push_str("</code></pre>\n");
    }

    output.push_str("    </article>\n");
    output.push_str("  );\n");
    output.push_str("}\n");
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_markdown() {
        let output = render_blog("# Hello\n\nA **bold** idea.\n\n- One");
        assert!(output.contains("<h1>Hello</h1>"));
        assert!(output.contains("<p>A <strong>bold</strong> idea.</p>"));
        assert!(output.contains("<li>One</li>"));
    }
}
