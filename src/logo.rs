include!(concat!(env!("OUT_DIR"), "/logos.rs"));

/// Family suffixes tried in order when the full distro key is unknown.
const SUFFIXES: [&str; 6] = ["-linux", "-os", "-bsd", "-edition", "-gnome", "-kde"];

/// Lowercases into `out`, collapsing non-alphanumeric runs to one `-`, no edge dashes.
fn normalize_into(out: &mut String, input: &str) {
    let mut pending_dash = false;
    for c in input.chars() {
        let c = c.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(c);
        } else {
            pending_dash = true;
        }
    }
}

/// Normalises a distro id or pretty name to a logo key for `set`, or `None` if unmatched.
pub fn normalize_key_in(input: &str, set: &str) -> Option<String> {
    let mut key = String::with_capacity(input.len());
    normalize_into(&mut key, input);
    if key.is_empty() {
        return None;
    }
    if has(set, &key) {
        return Some(key);
    }
    // try dropping a trailing suffix like "-linux", "-os", "-bsd"
    for suffix in SUFFIXES {
        if let Some(stripped) = key.strip_suffix(suffix)
            && has(set, stripped)
        {
            return Some(stripped.to_string());
        }
    }
    None
}

pub fn normalize_key(input: &str) -> Option<String> {
    normalize_key_in(input, "normal")
}

/// Resolves a logo by distro id, falling back to an explicit key.
pub fn lookup(id: &str, set: &str) -> Option<&'static str> {
    if let Some(v) = get(set, id) {
        return Some(v);
    }
    // Normalise against the set being searched: checking `normal` alone dropped mini-only logos.
    normalize_key_in(id, set).and_then(|k| get(set, &k))
}

pub fn exists(key: &str) -> bool {
    has("normal", key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_separator_runs_and_case() {
        let mut out = String::new();
        normalize_into(&mut out, "Arch Linux");
        assert_eq!(out, "arch-linux");
        let mut out = String::new();
        normalize_into(&mut out, "  --Arch__Linux--  ");
        assert_eq!(out, "arch-linux");
        let mut out = String::new();
        normalize_into(&mut out, "");
        assert_eq!(out, "");
        let mut out = String::new();
        normalize_into(&mut out, "!!!");
        assert_eq!(out, "");
    }

    #[test]
    fn tables_are_sorted_for_binary_search() {
        for table in [NORMAL, MINI] {
            assert!(!table.is_empty(), "logo table is empty");
            assert!(
                table.windows(2).all(|w| w[0].0 < w[1].0),
                "generated table is not strictly sorted by key"
            );
        }
    }

    #[test]
    fn get_agrees_with_has() {
        for (key, art) in NORMAL {
            assert_eq!(get("normal", key), Some(*art));
            assert!(has("normal", key), "{key} missing from normal");
        }
        for (key, art) in MINI {
            assert_eq!(get("mini", key), Some(*art));
            assert!(has("mini", key), "{key} missing from mini");
        }
    }

    #[test]
    fn normalizes_a_pretty_name() {
        assert!(has("normal", "arch"), "expected an arch logo");
        assert_eq!(normalize_key("Arch Linux").as_deref(), Some("arch"));
        assert_eq!(normalize_key("arch").as_deref(), Some("arch"));
        assert_eq!(normalize_key("totally unknown distro"), None);
    }
}
