use crate::cmd;
use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct Shell;

impl Module for Shell {
    fn name(&self) -> &'static str {
        "Shell"
    }
    fn id(&self) -> &'static str {
        "shell"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let name = detect_active_shell()?;
        let cache_key = format!("sh_{name}");
        let ver = if let Some(cached) = sys::cache_get(&cache_key) {
            let cleaned = if name == "bash" {
                clean_bash_version(&cached)
            } else {
                cached
            };
            Some(cleaned)
        } else {
            let v = detect_shell_version(&name);
            if let Some(ref ver_str) = v {
                sys::cache_set(&cache_key, ver_str);
            }
            v
        };
        Some(ModuleOutput::new(
            "Shell",
            match ver {
                Some(v) => format!("{name} {v}"),
                None => name,
            },
        ))
    }
}

pub fn detect_active_shell() -> Option<String> {
    // 1. Walk parent process tree (up to 10 hops) to find the calling shell
    let mut curr_pid = std::process::id();
    for _ in 0..10 {
        let ppid = get_ppid(curr_pid)?;
        if ppid <= 1 {
            break;
        }
        if let Some(name) = get_proc_name(ppid) {
            let clean = name.trim_start_matches('-').to_lowercase();
            if let Some(canonical) = match_known_shell(&clean) {
                return Some(canonical);
            }
        }
        curr_pid = ppid;
    }

    // 2. $SHELL from the environment
    if let Ok(path) = std::env::var("SHELL")
        && let Some(name) = path.rsplit('/').next()
    {
        let clean = name.trim_start_matches('-').to_lowercase();
        if let Some(canonical) = match_known_shell(&clean) {
            return Some(canonical);
        }
        if !clean.is_empty() {
            return Some(clean);
        }
    }

    None
}

fn write_proc_pid_path<'a>(buf: &'a mut [u8; 32], pid: u32, suffix: &[u8]) -> Option<&'a str> {
    let prefix = b"/proc/";
    let mut itoa_buf = [0u8; 10];
    let mut n = pid;
    let mut i = itoa_buf.len();
    if n == 0 {
        i -= 1;
        itoa_buf[i] = b'0';
    } else {
        while n > 0 {
            i -= 1;
            itoa_buf[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
    }
    let pid_bytes = &itoa_buf[i..];
    let total_len = prefix.len() + pid_bytes.len() + suffix.len();
    if total_len > buf.len() {
        return None;
    }
    buf[..prefix.len()].copy_from_slice(prefix);
    buf[prefix.len()..prefix.len() + pid_bytes.len()].copy_from_slice(pid_bytes);
    buf[prefix.len() + pid_bytes.len()..total_len].copy_from_slice(suffix);
    std::str::from_utf8(&buf[..total_len]).ok()
}

fn get_ppid(pid: u32) -> Option<u32> {
    let mut buf = [0u8; 512];
    let mut path_buf = [0u8; 32];
    let path = write_proc_pid_path(&mut path_buf, pid, b"/stat\0")?;
    let fd = unsafe {
        libc::open(
            path.as_ptr() as *const libc::c_char,
            libc::O_RDONLY | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return None;
    }
    let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
    unsafe { libc::close(fd) };
    if n <= 0 {
        return None;
    }
    let n = n as usize;
    let rparen = buf[..n].iter().rposition(|&b| b == b')')?;
    let mut i = rparen + 1;
    while i < n && buf[i].is_ascii_whitespace() {
        i += 1;
    }
    while i < n && !buf[i].is_ascii_whitespace() {
        i += 1;
    }
    while i < n && buf[i].is_ascii_whitespace() {
        i += 1;
    }
    let mut ppid = 0u32;
    while i < n && buf[i].is_ascii_digit() {
        ppid = ppid * 10 + (buf[i] - b'0') as u32;
        i += 1;
    }
    (ppid > 0).then_some(ppid)
}

fn get_proc_name(pid: u32) -> Option<String> {
    let mut path_buf = [0u8; 32];
    if let Some(exe_str) = write_proc_pid_path(&mut path_buf, pid, b"/exe\0") {
        let mut target = [0u8; 256];
        let len = unsafe {
            libc::readlink(
                exe_str.as_ptr() as *const libc::c_char,
                target.as_mut_ptr() as *mut libc::c_char,
                target.len(),
            )
        };
        if len > 0 {
            let slice = &target[..len as usize];
            let name = slice.rsplit(|&b| b == b'/').next().unwrap_or(slice);
            if !name.is_empty()
                && let Ok(s) = std::str::from_utf8(name)
            {
                return Some(s.to_string());
            }
        }
    }
    let mut comm_buf = [0u8; 32];
    if let Some(comm_str) = write_proc_pid_path(&mut comm_buf, pid, b"/comm\0") {
        let fd = unsafe {
            libc::open(
                comm_str.as_ptr() as *const libc::c_char,
                libc::O_RDONLY | libc::O_CLOEXEC,
            )
        };
        if fd >= 0 {
            let mut buf = [0u8; 64];
            let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            unsafe { libc::close(fd) };
            if n > 0 {
                let s = std::str::from_utf8(&buf[..n as usize]).ok()?.trim();
                if !s.is_empty() {
                    return Some(s.to_string());
                }
            }
        }
    }
    None
}

pub fn match_known_shell(name: &str) -> Option<String> {
    let name = name.strip_suffix(".exe").unwrap_or(name);
    match name {
        "bash" => Some("bash".to_string()),
        "fish" => Some("fish".to_string()),
        "zsh" => Some("zsh".to_string()),
        "nu" | "nushell" => Some("nu".to_string()),
        "pwsh" | "powershell" => Some("pwsh".to_string()),
        "ksh" | "mksh" | "oksh" | "pdksh" => Some("ksh".to_string()),
        "csh" | "tcsh" => Some(name.to_string()),
        "dash" => Some("dash".to_string()),
        "ash" => Some("ash".to_string()),
        "ion" => Some("ion".to_string()),
        "xonsh" => Some("xonsh".to_string()),
        "elvish" => Some("elvish".to_string()),
        "yash" => Some("yash".to_string()),
        "posh" => Some("posh".to_string()),
        "sh" => Some("sh".to_string()),
        "rc" => Some("rc".to_string()),
        "es" => Some("es".to_string()),
        _ => {
            if name.starts_with("bash") {
                Some("bash".to_string())
            } else if name.starts_with("fish") {
                Some("fish".to_string())
            } else if name.starts_with("zsh") {
                Some("zsh".to_string())
            } else {
                None
            }
        }
    }
}

pub fn detect_shell_version(name: &str) -> Option<String> {
    match name {
        "bash" => {
            let raw = std::env::var("BASH_VERSION").ok().or_else(|| {
                cmd::run("bash", &["--version"]).and_then(|v| {
                    v.split_whitespace()
                        .find(|tok| tok.chars().next().is_some_and(|c| c.is_ascii_digit()))
                        .map(|s| s.to_string())
                })
            })?;
            Some(clean_bash_version(&raw))
        }
        "fish" => std::env::var("FISH_VERSION").ok().or_else(|| {
            cmd::run("fish", &["--version"]).and_then(|v| {
                v.split_whitespace()
                    .find(|tok| tok.chars().next().is_some_and(|c| c.is_ascii_digit()))
                    .map(|s| s.to_string())
            })
        }),
        "zsh" => std::env::var("ZSH_VERSION").ok().or_else(|| {
            cmd::run("zsh", &["--version"])
                .and_then(|v| v.split_whitespace().nth(1).map(|s| s.to_string()))
        }),
        "nu" => std::env::var("NU_VERSION")
            .ok()
            .or_else(|| cmd::run("nu", &["--version"]).map(|v| v.trim().to_string())),
        "pwsh" | "powershell" => cmd::run("pwsh", &["-Version"]).map(|v| {
            v.split_whitespace()
                .find(|tok| tok.chars().next().is_some_and(|c| c.is_ascii_digit()))
                .unwrap_or(v.trim())
                .to_string()
        }),
        "elvish" => cmd::run("elvish", &["-version"]).map(|v| v.trim().to_string()),
        "xonsh" => cmd::run("xonsh", &["--version"]).map(|v| v.trim().to_string()),
        _ => None,
    }
}

pub fn clean_bash_version(raw: &str) -> String {
    let s = raw.trim();
    let without_paren = s.split('(').next().unwrap_or(s);
    let without_release = without_paren
        .split("-release")
        .next()
        .unwrap_or(without_paren);
    without_release.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_bash_version() {
        assert_eq!(clean_bash_version("5.3.15(1)-release"), "5.3.15");
        assert_eq!(clean_bash_version("5.2.26(1)-release"), "5.2.26");
        assert_eq!(clean_bash_version("5.1.16(1)"), "5.1.16");
        assert_eq!(clean_bash_version("5.3.15-release"), "5.3.15");
        assert_eq!(clean_bash_version("5.3.15"), "5.3.15");
    }

    #[test]
    fn test_match_known_shell() {
        assert_eq!(match_known_shell("fish").as_deref(), Some("fish"));
        assert_eq!(match_known_shell("bash").as_deref(), Some("bash"));
        assert_eq!(match_known_shell("zsh").as_deref(), Some("zsh"));
        assert_eq!(match_known_shell("nushell").as_deref(), Some("nu"));
        assert_eq!(match_known_shell("powershell").as_deref(), Some("pwsh"));
        assert_eq!(match_known_shell("bash5").as_deref(), Some("bash"));
        assert_eq!(match_known_shell("agy"), None);
    }
}
