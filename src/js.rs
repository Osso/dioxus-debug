/// Walk the DOM tree and return a JSON structure of all elements.
pub const DUMP: &str = r#"
var walk = function(el, depth) {
    var rect = el.getBoundingClientRect();
    var text = "";
    for (var t = 0; t < el.childNodes.length; t++) {
        if (el.childNodes[t].nodeType === 3) text += el.childNodes[t].textContent.trim();
    }
    var entry = {
        tag: el.tagName.toLowerCase(),
        id: el.id || null,
        classes: el.className ? String(el.className).split(/\s+/).filter(Boolean) : [],
        x: Math.round(rect.x),
        y: Math.round(rect.y),
        width: Math.round(rect.width),
        height: Math.round(rect.height),
        depth: depth,
        text: text || null,
        children: []
    };
    for (var i = 0; i < el.children.length; i++) {
        entry.children.push(walk(el.children[i], depth + 1));
    }
    return entry;
};
return walk(document.body, 0);
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
