use crate::cmd;
use crate::module::{Module, ModuleOutput};

pub struct DevEnv;

impl Module for DevEnv {
    fn name(&self) -> &'static str {
        "DevEnv"
    }
    fn id(&self) -> &'static str {
        "devenv"
    }
    fn run(&self) -> Option<ModuleOutput> {
        if let Some(tools) = detect_fast_pacman()
            && !tools.is_empty()
        {
            return Some(ModuleOutput::new("DevEnv", tools.join(", ")));
        }

        let probes: [fn() -> Option<String>; 5] = [
            probe_rust,
            probe_python,
            probe_node,
            probe_go,
            probe_c_compiler,
        ];

        let tools: Vec<String> = std::thread::scope(|s| {
            let handles: Vec<_> = probes.iter().map(|probe| s.spawn(*probe)).collect();
            handles
                .into_iter()
                .filter_map(|h| h.join().ok().flatten())
                .collect()
        });

        if tools.is_empty() {
            return None;
        }

        Some(ModuleOutput::new("DevEnv", tools.join(", ")))
    }
}

#[cfg(target_os = "linux")]
fn detect_fast_pacman() -> Option<Vec<String>> {
    let c_path = *b"/var/lib/pacman/local\0";
    let fd = unsafe {
        libc::open(
            c_path.as_ptr() as *const libc::c_char,
            libc::O_RDONLY | libc::O_DIRECTORY,
        )
    };
    if fd < 0 {
        return None;
    }
    let mut rust = None;
    let mut python = None;
    let mut node = None;
    let mut go = None;
    let mut gcc = None;
    let mut clang = None;

    let mut buf = [0u8; 32768];
    loop {
        let nread = unsafe { libc::syscall(libc::SYS_getdents64, fd, buf.as_mut_ptr(), buf.len()) };
        if nread <= 0 {
            break;
        }
        let mut pos = 0usize;
        let total = nread as usize;
        while pos + 19 <= total {
            let d_reclen = (buf[pos + 16] as usize) | ((buf[pos + 17] as usize) << 8);
            if d_reclen == 0 || pos + d_reclen > total {
                break;
            }
            let d_type = buf[pos + 18];
            let name_start = pos + 19;
            let mut name_end = name_start;
            while name_end < pos + d_reclen && buf[name_end] != 0 {
                name_end += 1;
            }
            let name_bytes = &buf[name_start..name_end];
            if d_type == libc::DT_DIR
                && !name_bytes.is_empty()
                && name_bytes != b"."
                && name_bytes != b".."
                && let Ok(s) = std::str::from_utf8(name_bytes)
            {
                if is_pkg_match(s, "rust") {
                    if let Some(v) = parse_pacman_ver(s, "rust-") {
                        rust = Some(format!("Rust {v}"));
                    }
                } else if is_pkg_match(s, "python") {
                    if let Some(v) = parse_pacman_ver(s, "python-") {
                        python = Some(format!("Python {v}"));
                    }
                } else if is_pkg_match(s, "nodejs") {
                    if let Some(v) = parse_pacman_ver(s, "nodejs-") {
                        node = Some(format!("Node {v}"));
                    }
                } else if is_pkg_match(s, "go") {
                    if let Some(v) = parse_pacman_ver(s, "go-") {
                        go = Some(format!("Go {v}"));
                    }
                } else if is_pkg_match(s, "gcc") {
                    if let Some(v) = parse_pacman_ver(s, "gcc-") {
                        let short_v = v.split('.').next().unwrap_or(&v);
                        gcc = Some(format!("GCC {short_v}"));
                    }
                } else if is_pkg_match(s, "clang")
                    && let Some(v) = parse_pacman_ver(s, "clang-")
                {
                    let short_v = v.split('.').next().unwrap_or(&v);
                    clang = Some(format!("Clang {short_v}"));
                }
            }
            pos += d_reclen;
        }
    }
    unsafe { libc::close(fd) };

    let mut res = Vec::new();
    if let Some(r) = rust
        && cmd::which("rustc")
    {
        res.push(r);
    }
    if let Some(p) = python
        && (cmd::which("python3") || cmd::which("python"))
    {
        res.push(p);
    }
    if let Some(n) = node
        && cmd::which("node")
    {
        res.push(n);
    }
    if let Some(g) = go
        && cmd::which("go")
    {
        res.push(g);
    }
    if let Some(c) = gcc {
        if cmd::which("gcc") {
            res.push(c);
        }
    } else if let Some(c) = clang
        && cmd::which("clang")
    {
        res.push(c);
    }

    if res.is_empty() { None } else { Some(res) }
}

#[cfg(not(target_os = "linux"))]
fn detect_fast_pacman() -> Option<Vec<String>> {
    let entries = std::fs::read_dir("/var/lib/pacman/local").ok()?;
    let mut rust = None;
    let mut python = None;
    let mut node = None;
    let mut go = None;
    let mut gcc = None;
    let mut clang = None;

    for entry in entries.flatten() {
        let name = entry.file_name();
        let Ok(s) = name.into_string() else { continue };
        if is_pkg_match(&s, "rust") && cmd::which("rustc") {
            if let Some(v) = parse_pacman_ver(&s, "rust-") {
                rust = Some(format!("Rust {v}"));
            }
        } else if is_pkg_match(&s, "python") && (cmd::which("python3") || cmd::which("python")) {
            if let Some(v) = parse_pacman_ver(&s, "python-") {
                python = Some(format!("Python {v}"));
            }
        } else if is_pkg_match(&s, "nodejs") && cmd::which("node") {
            if let Some(v) = parse_pacman_ver(&s, "nodejs-") {
                node = Some(format!("Node {v}"));
            }
        } else if is_pkg_match(&s, "go") && cmd::which("go") {
            if let Some(v) = parse_pacman_ver(&s, "go-") {
                go = Some(format!("Go {v}"));
            }
        } else if is_pkg_match(&s, "gcc") && cmd::which("gcc") {
            if let Some(v) = parse_pacman_ver(&s, "gcc-") {
                let short_v = v.split('.').next().unwrap_or(&v);
                gcc = Some(format!("GCC {short_v}"));
            }
        } else if is_pkg_match(&s, "clang") && cmd::which("clang") {
            if let Some(v) = parse_pacman_ver(&s, "clang-") {
                let short_v = v.split('.').next().unwrap_or(&v);
                clang = Some(format!("Clang {short_v}"));
            }
        }
    }

    let mut res = Vec::new();
    if let Some(r) = rust {
        res.push(r);
    }
    if let Some(p) = python {
        res.push(p);
    }
    if let Some(n) = node {
        res.push(n);
    }
    if let Some(g) = go {
        res.push(g);
    }
    if let Some(c) = gcc.or(clang) {
        res.push(c);
    }

    if res.is_empty() { None } else { Some(res) }
}

fn is_pkg_match(name: &str, pkg: &str) -> bool {
    let Some(rest) = name.strip_prefix(pkg) else {
        return false;
    };
    let Some(rest) = rest.strip_prefix('-') else {
        return false;
    };
    let first = rest.chars().next().unwrap_or(' ');
    first.is_ascii_digit()
}

fn parse_pacman_ver(name: &str, prefix: &str) -> Option<String> {
    let rest = name.strip_prefix(prefix)?;
    let without_epoch = if let Some((_, r)) = rest.split_once(':') {
        r
    } else {
        rest
    };
    let ver = without_epoch
        .rsplit_once('-')
        .map(|(v, _)| v)
        .unwrap_or(without_epoch);
    Some(ver.to_string())
}

fn probe_rust() -> Option<String> {
    if !cmd::which("rustc") {
        return None;
    }
    let out = cmd::run("rustc", &["-V"])?;
    parse_rustc_version(&out)
}

fn probe_python() -> Option<String> {
    if cmd::which("python3")
        && let Some(out) = cmd::run("python3", &["-V"])
        && let Some(tool) = parse_python_version(&out)
    {
        return Some(tool);
    }
    if cmd::which("python")
        && let Some(out) = cmd::run("python", &["-V"])
        && let Some(tool) = parse_python_version(&out)
    {
        return Some(tool);
    }
    None
}

fn probe_node() -> Option<String> {
    if !cmd::which("node") {
        return None;
    }
    let out = cmd::run("node", &["-v"])?;
    parse_node_version(&out)
}

fn probe_go() -> Option<String> {
    if !cmd::which("go") {
        return None;
    }
    let out = cmd::run("go", &["version"])?;
    parse_go_version(&out)
}

fn probe_c_compiler() -> Option<String> {
    if cmd::which("gcc")
        && let Some(out) = cmd::run("gcc", &["-dumpversion"])
    {
        let ver = out.trim();
        if !ver.is_empty() {
            return Some(format!("GCC {ver}"));
        }
    }
    if cmd::which("clang")
        && let Some(out) = cmd::run("clang", &["--version"])
        && let Some(tool) = parse_clang_version(&out)
    {
        return Some(tool);
    }
    None
}

pub fn parse_rustc_version(s: &str) -> Option<String> {
    let ver = s.split_whitespace().nth(1)?;
    Some(format!("Rust {ver}"))
}

pub fn parse_python_version(s: &str) -> Option<String> {
    let ver = s.split_whitespace().nth(1)?;
    Some(format!("Python {ver}"))
}

pub fn parse_node_version(s: &str) -> Option<String> {
    let ver = s.trim().trim_start_matches('v');
    if ver.is_empty() {
        None
    } else {
        Some(format!("Node {ver}"))
    }
}

pub fn parse_go_version(s: &str) -> Option<String> {
    for part in s.split_whitespace() {
        if let Some(ver) = part.strip_prefix("go")
            && ver
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            return Some(format!("Go {ver}"));
        }
    }
    None
}

pub fn parse_clang_version(s: &str) -> Option<String> {
    let pos = s.find("version ")?;
    let rest = &s[pos + 8..];
    let ver = rest.split_whitespace().next()?;
    Some(format!("Clang {ver}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkg_match_needs_a_version_after_the_dash() {
        assert!(is_pkg_match("rust-1.85.0-1", "rust"));
        assert!(is_pkg_match("python-3.14.7-1", "python"));
        assert!(is_pkg_match("nodejs-26.7.0-1", "nodejs"));
        // `rustfmt` is not `rust`.
        assert!(!is_pkg_match("rustfmt-1.8.0-1", "rust"));
        assert!(!is_pkg_match("nodejs-26.7.0-1", "node"));
        assert!(!is_pkg_match("rust", "rust"));
        assert!(!is_pkg_match("rust-", "rust"));
        assert!(!is_pkg_match("brust-1.0-1", "rust"));
        // The version starts right after the dash.
        assert!(!is_pkg_match("rust-abc-1", "rust"));
    }

    #[test]
    fn pacman_ver_drops_epoch_and_release_suffix() {
        assert_eq!(
            parse_pacman_ver("rust-1.85.0-1", "rust-").as_deref(),
            Some("1.85.0")
        );
        assert_eq!(
            parse_pacman_ver("python-3.14.7-1", "python-").as_deref(),
            Some("3.14.7")
        );
        assert_eq!(
            parse_pacman_ver("curl-8.12.1-2", "curl-").as_deref(),
            Some("8.12.1")
        );
    }

    #[test]
    fn pacman_ver_handles_epoch_and_bare_names() {
        assert_eq!(
            parse_pacman_ver("ffmpeg-7:6.1.1-2", "ffmpeg-").as_deref(),
            Some("6.1.1")
        );
        assert_eq!(parse_pacman_ver("go-1.24", "go-").as_deref(), Some("1.24"));
        assert!(parse_pacman_ver("go-1.24", "rust-").is_none());
    }

    #[test]
    fn test_parsers() {
        assert_eq!(
            parse_rustc_version("rustc 1.85.0 (4d91de4e4 2025-02-17)").as_deref(),
            Some("Rust 1.85.0")
        );
        assert_eq!(
            parse_python_version("Python 3.13.2").as_deref(),
            Some("Python 3.13.2")
        );
        assert_eq!(
            parse_node_version("v22.14.0\n").as_deref(),
            Some("Node 22.14.0")
        );
        assert_eq!(
            parse_go_version("go version go1.24.0 linux/amd64").as_deref(),
            Some("Go 1.24.0")
        );
        assert_eq!(
            parse_clang_version("clang version 19.1.7 (Fedora 19.1.7-1.fc41)").as_deref(),
            Some("Clang 19.1.7")
        );
    }
}
