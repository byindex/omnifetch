use std::io::Read;
use std::process::{Command, Stdio};

/// Runs a command and returns trimmed stdout, or None if it failed.
pub fn run(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let mut s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        return None;
    }
    if s.len() > 16 * 1024 {
        s.truncate(16 * 1024);
    }
    Some(s)
}

/// Same as `run`, but returns stdout even on a non-zero exit status.
pub fn run_lossy(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!s.is_empty()).then_some(s)
}

pub fn which(program: &str) -> bool {
    if program.contains('/') {
        return std::path::Path::new(program).exists();
    }
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(program).exists())
}

pub fn read_all(mut r: impl Read) -> String {
    let mut s = String::new();
    let _ = r.read_to_string(&mut s);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_which_existing_and_nonexistent() {
        assert!(which("sh") || which("bash"));
        assert!(which("/bin/sh") || which("/usr/bin/sh"));
        assert!(!which("definitely_not_a_valid_command_12345xyz"));
    }

    #[test]
    fn test_run_success_and_failure() {
        let out = run("sh", &["-c", "echo omnifetch_test"]);
        assert_eq!(out.as_deref(), Some("omnifetch_test"));

        let fail = run("sh", &["-c", "exit 1"]);
        assert_eq!(fail, None);

        let missing = run("nonexistent_command_xyz", &[]);
        assert_eq!(missing, None);
    }

    #[test]
    fn test_run_lossy() {
        let out = run_lossy("sh", &["-c", "echo lossy_test; exit 1"]);
        assert_eq!(out.as_deref(), Some("lossy_test"));

        let empty = run_lossy("sh", &["-c", "exit 0"]);
        assert_eq!(empty, None);
    }

    #[test]
    fn test_read_all() {
        let data = b"hello from memory buffer";
        let res = read_all(&data[..]);
        assert_eq!(res, "hello from memory buffer");
    }
}
