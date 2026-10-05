use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct Git;

impl Module for Git {
    fn name(&self) -> &'static str {
        "Git"
    }
    fn id(&self) -> &'static str {
        "git"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let repo_root = find_git_root()?;
        let mut head_buf = [0u8; 128];
        let mut head_p = repo_root.clone();
        head_p.push(".git");
        head_p.push("HEAD");
        let head_content = sys::read_bytes_into(&head_p, &mut head_buf)
            .and_then(|n| std::str::from_utf8(&head_buf[..n]).ok())
            .map(|s| s.trim());

        let branch = detect_branch_from_head(head_content)
            .or_else(|| detect_branch(&repo_root))
            .unwrap_or_else(|| "detached".to_string());
        let commit = detect_last_commit_from_head(&repo_root, head_content)
            .or_else(|| detect_last_commit(&repo_root));
        let commit_count = detect_commit_count(&repo_root);
        let languages = detect_languages(&repo_root);

        let mut parts = Vec::new();
        parts.push(branch);

        if let Some(c) = commit {
            parts.push(format!("@{c}"));
        }

        if let Some(count) = commit_count {
            parts.push(format!("({count} commits)"));
        }

        let mut val = parts.join(" ");

        if !languages.is_empty() {
            let langs_str = languages
                .iter()
                .take(3)
                .map(|(lang, pct)| format!("{lang} {pct}%"))
                .collect::<Vec<_>>()
                .join(", ");
            val = format!("{val} [{langs_str}]");
        }

        Some(ModuleOutput::new("Git", val))
    }
}

pub fn find_git_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    for _ in 0..10 {
        let git_dir = current.join(".git");
        if git_dir.exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

fn detect_branch_from_head(head_content: Option<&str>) -> Option<String> {
    let trimmed = head_content?;
    if let Some(branch) = trimmed.strip_prefix("ref: refs/heads/") {
        Some(branch.to_string())
    } else if trimmed.len() >= 7 {
        Some(trimmed[..7].to_string())
    } else {
        Some(trimmed.to_string())
    }
}

pub fn detect_branch(repo_root: &Path) -> Option<String> {
    let mut head_buf = [0u8; 128];
    let head_p = repo_root.join(".git/HEAD");
    let content = if let Some(n) = sys::read_bytes_into(&head_p, &mut head_buf) {
        std::str::from_utf8(&head_buf[..n]).ok().map(|s| s.trim())
    } else {
        None
    };
    if let Some(b) = detect_branch_from_head(content) {
        return Some(b);
    }
    let head_content = fs::read_to_string(head_p).ok()?;
    let trimmed = head_content.trim();
    if let Some(branch) = trimmed.strip_prefix("ref: refs/heads/") {
        Some(branch.to_string())
    } else {
        Some(trimmed.chars().take(7).collect())
    }
}

fn detect_last_commit_from_head(repo_root: &Path, head_content: Option<&str>) -> Option<String> {
    let trimmed = head_content?;
    if let Some(ref_path) = trimmed.strip_prefix("ref: ") {
        let ref_file = repo_root.join(".git").join(ref_path);
        let mut hash_buf = [0u8; 64];
        if let Some(n) = sys::read_bytes_into(&ref_file, &mut hash_buf)
            && let Ok(s) = std::str::from_utf8(&hash_buf[..n])
        {
            let t = s.trim();
            if t.len() >= 7 {
                return Some(t[..7].to_string());
            }
        }
        if let Ok(packed) = fs::read_to_string(repo_root.join(".git/packed-refs")) {
            for line in packed.lines() {
                if let Some((hash, name)) = line.split_once(' ')
                    && name == ref_path
                    && hash.len() >= 7
                {
                    return Some(hash[..7].to_string());
                }
            }
        }
        None
    } else if trimmed.len() >= 7 {
        Some(trimmed[..7].to_string())
    } else {
        None
    }
}

pub fn detect_last_commit(repo_root: &Path) -> Option<String> {
    let mut head_buf = [0u8; 128];
    let head_p = repo_root.join(".git/HEAD");
    let content = if let Some(n) = sys::read_bytes_into(&head_p, &mut head_buf) {
        std::str::from_utf8(&head_buf[..n]).ok().map(|s| s.trim())
    } else {
        None
    };
    if let Some(c) = detect_last_commit_from_head(repo_root, content) {
        return Some(c);
    }
    let head_content = fs::read_to_string(head_p).ok()?;
    let trimmed = head_content.trim();
    if let Some(ref_path) = trimmed.strip_prefix("ref: ") {
        if let Ok(hash) = fs::read_to_string(repo_root.join(".git").join(ref_path)) {
            return Some(hash.trim().chars().take(7).collect());
        }
        if let Ok(packed) = fs::read_to_string(repo_root.join(".git/packed-refs")) {
            for line in packed.lines() {
                if line.ends_with(ref_path)
                    && let Some((hash, _)) = line.split_once(' ')
                {
                    return Some(hash.chars().take(7).collect());
                }
            }
        }
        None
    } else if trimmed.len() >= 7 {
        Some(trimmed.chars().take(7).collect())
    } else {
        None
    }
}

pub fn detect_commit_count(repo_root: &Path) -> Option<usize> {
    use std::io::Read;
    let logs = repo_root.join(".git/logs/HEAD");
    let mut f = fs::File::open(logs).ok()?;
    let mut buf = [0u8; 8192];
    let mut count = 0;
    loop {
        let n = f.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        count += buf[..n].iter().filter(|&&b| b == b'\n').count();
    }
    (count > 0).then_some(count)
}

pub fn detect_languages(repo_root: &Path) -> Vec<(String, usize)> {
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    let mut total_files = 0;

    let mut stack = vec![repo_root.to_path_buf()];
    let mut checked_files = 0;

    'outer: while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let bytes = name.as_encoded_bytes();

            if bytes.starts_with(b".")
                || bytes == b"target"
                || bytes == b"node_modules"
                || bytes == b"build"
                || bytes == b"dist"
                || bytes == b".git"
                || bytes == b"assets"
                || bytes == b"docs"
                || bytes == b"doc"
                || bytes == b"man"
                || bytes == b"packaging"
                || bytes == b"images"
                || bytes == b"img"
            {
                continue;
            }

            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_dir() {
                if stack.len() < 30 {
                    stack.push(entry.path());
                }
            } else if ft.is_file()
                && let Some(dot_idx) = bytes.iter().rposition(|&b| b == b'.')
            {
                let ext = &bytes[dot_idx + 1..];
                let lang = match ext {
                    b"rs" | b"RS" => Some("Rust"),
                    b"c" | b"C" | b"h" | b"H" => Some("C"),
                    b"cpp" | b"CPP" | b"hpp" | b"HPP" | b"cc" | b"cxx" => Some("C++"),
                    b"py" | b"PY" => Some("Python"),
                    b"js" | b"mjs" => Some("JavaScript"),
                    b"ts" | b"tsx" => Some("TypeScript"),
                    b"go" => Some("Go"),
                    b"sh" | b"bash" | b"zsh" => Some("Shell"),
                    b"html" | b"htm" => Some("HTML"),
                    b"css" | b"scss" | b"sass" => Some("CSS"),
                    b"lua" => Some("Lua"),
                    b"zig" => Some("Zig"),
                    b"nim" => Some("Nim"),
                    b"java" => Some("Java"),
                    b"kt" | b"kts" => Some("Kotlin"),
                    b"swift" => Some("Swift"),
                    b"rb" => Some("Ruby"),
                    b"php" => Some("PHP"),
                    _ => None,
                };
                if let Some(l) = lang {
                    *counts.entry(l).or_insert(0) += 1;
                    total_files += 1;
                    checked_files += 1;
                    if checked_files >= 150 {
                        break 'outer;
                    }
                }
            }
        }
    }

    if total_files == 0 {
        return Vec::new();
    }

    let mut vec: Vec<(String, usize)> = counts
        .into_iter()
        .map(|(lang, cnt)| {
            let pct = ((cnt as f64 / total_files as f64) * 100.0).round() as usize;
            (lang.to_string(), pct)
        })
        .collect();

    vec.sort_by_key(|(_, pct)| std::cmp::Reverse(*pct));
    vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_root_finds_self() {
        assert!(find_git_root().is_some());
    }

    #[test]
    fn test_git_branch_not_empty() {
        if let Some(root) = find_git_root() {
            let branch = detect_branch(&root);
            assert!(branch.is_some());
        }
    }
}
