use std::fs;
use std::path::Path;

use crate::module::{Module, ModuleOutput};

pub struct Packages;

impl Module for Packages {
    fn name(&self) -> &'static str {
        "Packages"
    }
    fn id(&self) -> &'static str {
        "packages"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut counts = Vec::new();

        if let Some(count) = count_pacman()
            && count > 0
        {
            counts.push(format!("{count} (pacman)"));
        }

        if let Some(count) = count_dpkg()
            && count > 0
        {
            counts.push(format!("{count} (dpkg)"));
        }

        if let Some(count) = count_rpm()
            && count > 0
        {
            counts.push(format!("{count} (rpm)"));
        }

        if let Some(count) = count_flatpak()
            && count > 0
        {
            counts.push(format!("{count} (flatpak)"));
        }

        if let Some(count) = count_snap()
            && count > 0
        {
            counts.push(format!("{count} (snap)"));
        }

        if let Some(count) = count_brew()
            && count > 0
        {
            counts.push(format!("{count} (brew)"));
        }

        if counts.is_empty() {
            return None;
        }

        Some(ModuleOutput::new("Packages", counts.join(", ")))
    }
}

#[cfg(target_os = "linux")]
fn count_dirs_fast(path_str: &str) -> Option<usize> {
    let mut c_path = [0u8; 128];
    let bytes = path_str.as_bytes();
    if bytes.len() >= c_path.len() {
        return None;
    }
    c_path[..bytes.len()].copy_from_slice(bytes);
    c_path[bytes.len()] = 0;

    let fd = unsafe {
        libc::open(
            c_path.as_ptr() as *const libc::c_char,
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return None;
    }
    let mut count = 0usize;
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
            if name_bytes != b"."
                && name_bytes != b".."
                && (d_type == libc::DT_DIR || d_type == libc::DT_UNKNOWN)
            {
                count += 1;
            }
            pos += d_reclen;
        }
    }
    unsafe { libc::close(fd) };
    Some(count)
}

fn count_dirs(path: impl AsRef<Path>) -> Option<usize> {
    let entries = fs::read_dir(path).ok()?;
    let mut count = 0;
    for e in entries.flatten() {
        if let Ok(ft) = e.file_type()
            && ft.is_dir()
        {
            count += 1;
        }
    }
    Some(count)
}

fn count_pacman() -> Option<usize> {
    #[cfg(target_os = "linux")]
    if let Some(count) = count_dirs_fast("/var/lib/pacman/local") {
        return (count > 0).then_some(count);
    }
    let p = Path::new("/var/lib/pacman/local");
    if !p.is_dir() {
        return None;
    }
    count_dirs(p)
}

fn count_dpkg() -> Option<usize> {
    use std::io::Read;
    let mut file = fs::File::open("/var/lib/dpkg/status").ok()?;
    let mut buf = [0u8; 32768];
    let mut count = 0;
    let mut at_line_start = true;
    let target = b"Package: ";
    let mut leftover = 0;
    loop {
        let n = file.read(&mut buf[leftover..]).ok()?;
        if n == 0 && leftover == 0 {
            break;
        }
        let total = leftover + n;
        let chunk = &buf[..total];
        let mut i = 0;
        while i < chunk.len() {
            if at_line_start {
                if chunk.len() - i < target.len() {
                    break;
                }
                if chunk[i..].starts_with(target) {
                    count += 1;
                    i += target.len();
                }
            }
            if let Some(pos) = chunk[i..].iter().position(|&b| b == b'\n') {
                i += pos + 1;
                at_line_start = true;
            } else {
                i = chunk.len();
                at_line_start = false;
            }
        }
        leftover = total - i;
        if leftover > 0 {
            buf.copy_within(i..total, 0);
        }
        if n == 0 {
            break;
        }
    }
    (count > 0).then_some(count)
}

fn count_rpm() -> Option<usize> {
    None
}

fn count_flatpak() -> Option<usize> {
    let mut total = 0;
    for dir in ["/var/lib/flatpak/app", "/usr/share/flatpak/app"] {
        if let Some(c) = count_dirs(dir) {
            total += c;
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let user_flatpak = format!("{home}/.local/share/flatpak/app");
        if let Some(c) = count_dirs(user_flatpak) {
            total += c;
        }
    }
    (total > 0).then_some(total)
}

fn count_snap() -> Option<usize> {
    let p = Path::new("/var/lib/snapd/snaps");
    let entries = fs::read_dir(p).ok()?;
    let mut count = 0;
    for e in entries.flatten() {
        if let Some(name) = e.file_name().to_str()
            && name.ends_with(".snap")
        {
            count += 1;
        }
    }
    (count > 0).then_some(count)
}

fn count_brew() -> Option<usize> {
    for base in [
        "/home/linuxbrew/.linuxbrew/Cellar",
        "/opt/homebrew/Cellar",
        "/usr/local/Cellar",
    ] {
        if let Some(c) = count_dirs(base)
            && c > 0
        {
            return Some(c);
        }
    }
    None
}
