/// Format a JSON DOM dump as an indented text tree.
///
/// Input: JSON string from the DOM walk JS (tag, id, classes, x, y, width, height, text, children).
/// Output: Indented tree string like browser-cli snapshot.
pub fn format_tree(json: &str) -> Result<String, String> {
    let node: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {e}"))?;
    let mut lines = Vec::new();
    format_node(&node, 0, &mut lines);
    Ok(lines.join("\n"))
}

fn format_node(node: &serde_json::Value, depth: usize, lines: &mut Vec<String>) {
    let indent = "  ".repeat(depth);
    let tag = node["tag"].as_str().unwrap_or("?");

    let mut line = format!("{indent}- {tag}");

    if let Some(id) = node["id"].as_str() {
        line.push_str(&format!(" #{id}"));
    }

    if let Some(classes) = node["classes"].as_array() {
        for c in classes {
            if let Some(s) = c.as_str() {
                line.push_str(&format!(" .{s}"));
            }
        }
    }

    if let Some(text) = node["text"].as_str() {
        if !text.is_empty() {
            let truncated = truncate(text, 60);
            line.push_str(&format!(" \"{truncated}\""));
        }
    }

    let x = node["x"].as_i64().unwrap_or(0);
    let y = node["y"].as_i64().unwrap_or(0);
    let w = node["width"].as_i64().unwrap_or(0);
    let h = node["height"].as_i64().unwrap_or(0);
    if w > 0 || h > 0 {
        line.push_str(&format!(" [x={x} y={y} w={w} h={h}]"));
    }

    lines.push(line);

    if let Some(children) = node["children"].as_array() {
        for child in children {
            format_node(child, depth + 1, lines);
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
