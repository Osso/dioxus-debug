/// Walk the DOM tree and return a JSON structure of all elements.
pub const DUMP: &str = r#"
function walk(node, depth) {
    if (node.nodeType === Node.TEXT_NODE) {
        var text = node.textContent.trim();
        if (text) return { type: "text", text: text, depth: depth };
        return null;
    }
    if (node.nodeType !== Node.ELEMENT_NODE) return null;
    var el = node;
    var rect = el.getBoundingClientRect();
    var entry = {
        type: "element",
        tag: el.tagName.toLowerCase(),
        id: el.id || null,
        classes: el.className ? el.className.split(/\s+/).filter(Boolean) : [],
        x: Math.round(rect.x),
        y: Math.round(rect.y),
        width: Math.round(rect.width),
        height: Math.round(rect.height),
        depth: depth,
        children: []
    };
    for (var i = 0; i < el.childNodes.length; i++) {
        var child = walk(el.childNodes[i], depth + 1);
        if (child) entry.children.push(child);
    }
    return entry;
}
return JSON.stringify(walk(document.body, 0));
"#;

/// Click an element matching a CSS selector, falling back to text match.
/// Placeholder `{SELECTOR}` is replaced at runtime.
pub const CLICK: &str = r#"
var el = document.querySelector("{SELECTOR}");
if (!el) {
    var all = document.querySelectorAll("*");
    for (var i = 0; i < all.length; i++) {
        if (all[i].textContent.trim() === "{SELECTOR}") {
            el = all[i]; break;
        }
    }
}
if (!el) return "error:Element not found: {SELECTOR}";
el.click();
return "ok";
"#;

/// Click an element whose text content matches exactly.
/// Placeholder `{TEXT}` is replaced at runtime.
pub const CLICK_TEXT: &str = r#"
var all = document.querySelectorAll("*");
for (var i = all.length - 1; i >= 0; i--) {
    var el = all[i];
    if (el.childNodes.length === 1 && el.childNodes[0].nodeType === Node.TEXT_NODE) {
        if (el.textContent.trim() === "{TEXT}") {
            el.click();
            return "ok";
        }
    }
}
return "error:Text not found: {TEXT}";
"#;

/// Set input value on an element matching a CSS selector.
/// Placeholders: `{SELECTOR}`, `{VALUE}`
pub const INPUT: &str = r#"
var el = document.querySelector("{SELECTOR}");
if (!el) return "error:Element not found: {SELECTOR}";
var nativeSetter = Object.getOwnPropertyDescriptor(
    window.HTMLInputElement.prototype, "value"
).set;
nativeSetter.call(el, "{VALUE}");
el.dispatchEvent(new Event("input", { bubbles: true }));
el.dispatchEvent(new Event("change", { bubbles: true }));
return "ok";
"#;

/// Return the full page HTML as a screenshot proxy.
pub const SCREENSHOT: &str = r#"
return document.documentElement.outerHTML;
"#;

/// Build the JS string for a click command.
pub fn click_js(selector: &str) -> String {
    if let Some(text) = selector.strip_prefix("text:") {
        CLICK_TEXT.replace("{TEXT}", &escape_js(text))
    } else {
        CLICK.replace("{SELECTOR}", &escape_js(selector))
    }
}

/// Build the JS string for an input command.
pub fn input_js(selector: &str, value: &str) -> String {
    INPUT
        .replace("{SELECTOR}", &escape_js(selector))
        .replace("{VALUE}", &escape_js(value))
}

/// Escape a string for safe embedding in JS string literals.
fn escape_js(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
