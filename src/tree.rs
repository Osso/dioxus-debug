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

#[cfg(test)]
mod tests {
    use super::format_tree;

    #[test]
    fn formats_nested_dom_with_metadata() {
        let json = r##"{
            "tag": "body",
            "id": null,
            "classes": [],
            "x": 0,
            "y": 0,
            "width": 320,
            "height": 200,
            "text": null,
            "children": [
                {
                    "tag": "button",
                    "id": "save",
                    "classes": ["primary", "wide"],
                    "x": 10,
                    "y": 20,
                    "width": 90,
                    "height": 30,
                    "text": "Save",
                    "children": []
                }
            ]
        }"##;

        let tree = format_tree(json).unwrap();

        assert_eq!(
            tree,
            "- body [x=0 y=0 w=320 h=200]\n  - button #save .primary .wide \"Save\" [x=10 y=20 w=90 h=30]"
        );
    }

    #[test]
    fn omits_empty_text_and_zero_size() {
        let json = r#"{"tag":"span","text":"","children":[]}"#;

        let tree = format_tree(json).unwrap();

        assert_eq!(tree, "- span");
    }

    #[test]
    fn truncates_long_text() {
        let json = format!(r#"{{"tag":"p","text":"{}","children":[]}}"#, "a".repeat(61));

        let tree = format_tree(&json).unwrap();

        assert_eq!(tree, format!("- p \"{}...\"", "a".repeat(60)));
    }

    #[test]
    fn reports_invalid_json() {
        let err = format_tree("{not valid").unwrap_err();

        assert!(err.starts_with("Invalid JSON:"));
    }
}
