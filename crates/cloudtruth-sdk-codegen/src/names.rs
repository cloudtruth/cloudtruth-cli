// Converts a URL with snake_case path segments to PascalCase.
// Forward slashes and underscores are converted to capitalized names
// braces from path variables are ignored
pub fn convert_url_to_type_name(url: &str) -> String {
    let mut pascal = String::new();
    let mut capitalize = true;
    for ch in url.chars() {
        if ch == '{' || ch == '}' {
            continue;
        } else if ch == '_' || ch == '/' {
            capitalize = true;
        } else if capitalize {
            pascal.push(ch.to_ascii_uppercase());
            capitalize = false;
        } else {
            pascal.push(ch);
        }
    }
    pascal
}

// Remove curly brackets from a path variable of them form "{name}"
// If no brackets, string is returned as-is
pub fn trim_path_var_brackets(s: &str) -> &str {
    s.strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or(s)
}
