use std::collections::BTreeMap;
use std::fs;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Reads a whole file into `buf`, skipping the `stat` and `String` `read_to_string` needs.
pub fn read_small<'a>(path: &[u8], buf: &'a mut [u8]) -> Option<&'a str> {
    use std::io::Read;
    use std::os::unix::ffi::OsStrExt;
    // Keep the terminator out of the path handed to the syscall.
    let raw = match path.split_last() {
        Some((b'\0', head)) => head,
        _ => path,
    };
    let p = std::ffi::OsStr::from_bytes(raw);
    let mut f = fs::File::open(p).ok()?;
    let mut n = 0;
    loop {
        if n == buf.len() {
            return None;
        }
        match f.read(&mut buf[n..]) {
            Ok(0) => break,
            Ok(k) => n += k,
            Err(_) => return None,
        }
    }
    std::str::from_utf8(&buf[..n]).ok()
}

pub fn read_trim(path: impl AsRef<Path>) -> Option<String> {
    let mut buf = [0u8; 256];
    if let Some(n) = read_bytes_into(path.as_ref(), &mut buf)
        && n < buf.len()
    {
        return std::str::from_utf8(&buf[..n])
            .ok()
            .map(|s| s.trim().to_string());
    }
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

/// Reads a numeric sysfs/procfs value, cutting it at the first non-numeric character.
pub fn read_num(path: impl AsRef<Path>) -> Option<f64> {
    let mut buf = [0u8; 64];
    if let Some(n) = read_bytes_into(path.as_ref(), &mut buf)
        && let Ok(s) = std::str::from_utf8(&buf[..n])
    {
        let end = s
            .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
            .unwrap_or(s.len());
        return s[..end].trim().parse().ok();
    }
    let s = fs::read_to_string(path).ok()?;
    let end = s
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(s.len());
    s[..end].trim().parse().ok()
}

pub fn read_num_str(path: impl AsRef<Path>) -> Option<String> {
    let mut buf = [0u8; 64];
    if let Some(n) = read_bytes_into(path.as_ref(), &mut buf)
        && let Ok(s) = std::str::from_utf8(&buf[..n])
    {
        let end = s
            .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
            .unwrap_or(s.len());
        let s = s[..end].trim();
        if !s.is_empty() {
            return Some(s.to_string());
        }
    }
    let s = fs::read_to_string(path).ok()?;
    let end = s
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(s.len());
    let s = s[..end].trim();
    (!s.is_empty()).then(|| s.to_string())
}

pub fn proc_input_devices() -> Option<&'static str> {
    static CONTENT: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    CONTENT
        .get_or_init(|| fs::read_to_string("/proc/bus/input/devices").ok())
        .as_deref()
}

// --- Distribution identity, from /etc/os-release and friends ---

#[derive(Debug, Clone, Default)]
pub struct OsRelease {
    pub data: BTreeMap<String, String>,
}

impl OsRelease {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    pub fn id(&self) -> &str {
        self.get("ID").unwrap_or("linux")
    }

    pub fn id_like(&self) -> Option<&str> {
        self.get("ID_LIKE")
    }

    pub fn name(&self) -> String {
        if let Some(n) = self.get("NAME") {
            return unquote(n);
        }
        if let Some(p) = self.get("PRETTY_NAME") {
            return unquote(p);
        }
        let id = self.id();
        let mut s = String::from(id);
        if let Some(first) = s.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        s
    }

    pub fn version(&self) -> Option<String> {
        if let Some(v) = self.get("VERSION") {
            return Some(unquote(v));
        }
        if let Some(v) = self.get("BUILD_ID") {
            return Some(unquote(v));
        }
        if let Some(v) = self.get("VARIANT_VERSION") {
            return Some(unquote(v));
        }
        None
    }

    pub fn arch(&self) -> Option<String> {
        self.get("ARCH").map(unquote).filter(|s| !s.is_empty())
    }
}

fn unquote(s: &str) -> String {
    let t = s.trim();
    if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

pub fn os_release() -> &'static OsRelease {
    static OS_RELEASE: std::sync::OnceLock<OsRelease> = std::sync::OnceLock::new();
    OS_RELEASE.get_or_init(|| {
        let mut data = BTreeMap::new();
        if let Some(c) = read_trim("/etc/os-release") {
            parse_os_release(&c, &mut data);
        }
        if data.is_empty()
            && let Some(c) = read_trim("/usr/lib/os-release")
        {
            parse_os_release(&c, &mut data);
        }
        OsRelease { data }
    })
}

fn parse_os_release(content: &str, data: &mut BTreeMap<String, String>) {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        data.insert(k.trim().to_string(), v.trim().to_string());
    }
}

// --- Device identity, from the DMI tables in sysfs ---

/// Reads /sys/class/dmi/id, skipping the fields the kernel withholds from unprivileged users.
pub fn dmi(field: &str) -> Option<String> {
    let mut path = [0u8; 64];
    let prefix = b"/sys/class/dmi/id/";
    if prefix.len() + field.len() >= path.len() {
        return None;
    }
    path[..prefix.len()].copy_from_slice(prefix);
    path[prefix.len()..prefix.len() + field.len()].copy_from_slice(field.as_bytes());
    let path_str = std::str::from_utf8(&path[..prefix.len() + field.len()]).ok()?;
    let v = read_trim(path_str)?;
    (!v.is_empty()).then_some(v)
}

// --- Wall-clock and uptime helpers ---

pub fn boot_time() -> Option<SystemTime> {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    if unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut ts) } == 0 {
        let boot_dur = Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32);
        let now = SystemTime::now();
        return now.checked_sub(boot_dur);
    }
    let up = read_num("/proc/uptime")?;
    let since = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
    Some(UNIX_EPOCH + since - Duration::from_secs_f64(up))
}

/// Renders a duration the way fastfetch does: "3d 4h 22m", "4h 22m", "12m 03s".
pub fn human_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3_600;
    let mins = (secs % 3_600) / 60;
    let s = secs % 60;
    if days > 0 {
        format!("{days}d {hours}h {mins}m")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else if mins > 0 {
        format!("{mins}m {s:02}s")
    } else {
        format!("{s}s")
    }
}

pub fn human_bytes(n: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    if n < 1024 {
        return format!("{n} B");
    }
    let mut v = n as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if v >= 100.0 {
        format!("{v:.0} {}", UNITS[i])
    } else if v >= 10.0 {
        format!("{v:.1} {}", UNITS[i])
    } else {
        format!("{v:.2} {}", UNITS[i])
    }
}

// --- Process table traversal ---

#[derive(Debug, Clone)]
pub struct ProcEntry {
    pub pid: u32,
    pub tty_nr: i32,
    pub threads: u32,
    pub comm: String,
}

impl ProcEntry {
    /// Effective uid, one `stat` per process.
    pub fn uid(&self) -> std::io::Result<u32> {
        use std::io::Write;
        use std::os::unix::fs::MetadataExt;
        let mut buf = [0u8; 32];
        let mut cur = std::io::Cursor::new(&mut buf[..]);
        let _ = write!(cur, "/proc/{}", self.pid);
        let len = cur.position() as usize;
        let path = std::str::from_utf8(&buf[..len])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(std::fs::metadata(path)?.uid())
    }
}

/// Reads a file into a caller-supplied buffer without validating the whole file as UTF-8.
pub fn read_bytes_into(path: &Path, buf: &mut [u8]) -> Option<usize> {
    use std::os::unix::ffi::OsStrExt;
    let bytes = path.as_os_str().as_bytes();
    let mut c_path = [0u8; 256];
    if bytes.len() >= c_path.len() {
        let mut f = fs::File::open(path).ok()?;
        let mut total = 0;
        while total < buf.len() {
            match std::io::Read::read(&mut f, &mut buf[total..]) {
                Ok(0) | Err(_) => break,
                Ok(n) => total += n,
            }
        }
        return Some(total);
    }
    c_path[..bytes.len()].copy_from_slice(bytes);
    c_path[bytes.len()] = 0;
    let fd = unsafe {
        libc::open(
            c_path.as_ptr() as *const libc::c_char,
            libc::O_RDONLY | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return None;
    }
    let mut total = 0;
    while total < buf.len() {
        let n = unsafe {
            libc::read(
                fd,
                buf[total..].as_mut_ptr() as *mut libc::c_void,
                buf.len() - total,
            )
        };
        if n <= 0 {
            break;
        }
        total += n as usize;
    }
    unsafe {
        libc::close(fd);
    }
    Some(total)
}

/// Walks every process, taking `tty_nr` (field 7) and `num_threads` (field 20) in one read each.
pub fn iter_procs() -> impl Iterator<Item = ProcEntry> {
    let dir = fs::read_dir("/proc").ok();
    dir.into_iter().flatten().flatten().filter_map(|e| {
        let name = e.file_name();
        let s = name.to_str()?;
        if !s.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let pid: u32 = s.parse().ok()?;

        let mut path_buf = [0u8; 48];
        let prefix = b"/proc/";
        let suffix = b"/stat";
        let s_bytes = s.as_bytes();
        let total_len = prefix.len() + s_bytes.len() + suffix.len();
        if total_len > path_buf.len() {
            return None;
        }
        path_buf[..prefix.len()].copy_from_slice(prefix);
        path_buf[prefix.len()..prefix.len() + s_bytes.len()].copy_from_slice(s_bytes);
        path_buf[prefix.len() + s_bytes.len()..total_len].copy_from_slice(suffix);
        let path = std::str::from_utf8(&path_buf[..total_len]).ok()?;

        let mut buf = [0u8; 1024];
        let n = read_bytes_into(Path::new(path), &mut buf)?;
        let text = std::str::from_utf8(&buf[..n]).ok()?;
        // comm can hold spaces, so cut after the last ')'. Field 1 is the pid.
        let (comm_part, rest) = text.rsplit_once(')')?;
        let comm = comm_part
            .split_once('(')
            .map(|(_, c)| c.to_string())
            .unwrap_or_default();
        // After comm is split off, field N sits at index N-3.
        let mut it = rest.split_ascii_whitespace();
        let mut tty_nr = None;
        let mut threads = None;
        for i in 0..=17 {
            let Some(tok) = it.next() else { break };
            match i {
                4 => tty_nr = tok.parse::<i32>().ok(),
                17 => {
                    threads = tok.parse::<u32>().ok();
                    break;
                }
                _ => {}
            }
        }
        Some(ProcEntry {
            pid,
            tty_nr: tty_nr?,
            threads: threads?,
            comm,
        })
    })
}

/// Computes total process count via getdents64 and thread count from /proc/loadavg.
pub fn process_and_thread_count() -> (u32, u32) {
    let mut procs = 0u32;
    let fd = unsafe { libc::open(c"/proc".as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY) };
    if fd >= 0 {
        let mut buf = [0u8; 32768];
        loop {
            let nread =
                unsafe { libc::syscall(libc::SYS_getdents64, fd, buf.as_mut_ptr(), buf.len()) };
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
                let name_start = pos + 19;
                if name_start < pos + d_reclen && buf[name_start].is_ascii_digit() {
                    let mut name_end = name_start + 1;
                    while name_end < pos + d_reclen && buf[name_end] != 0 {
                        name_end += 1;
                    }
                    let name_bytes = &buf[name_start..name_end];
                    if name_bytes.iter().all(|b| b.is_ascii_digit()) {
                        procs += 1;
                    }
                }
                pos += d_reclen;
            }
        }
        unsafe { libc::close(fd) };
    }

    // Read total threads from /proc/loadavg (field 4: nr_running/nr_threads)
    let mut loadavg_buf = [0u8; 128];
    if let Some(n) = read_bytes_into(Path::new("/proc/loadavg"), &mut loadavg_buf)
        && let Ok(s) = std::str::from_utf8(&loadavg_buf[..n])
        && let Some(slash) = s.split_whitespace().nth(3).and_then(|f| f.split_once('/'))
        && let Ok(th) = slash.1.trim().parse::<u32>()
    {
        return (procs, th);
    }

    let mut threads = 0u32;
    for p in iter_procs() {
        threads += p.threads;
    }
    (procs, threads)
}

/// Formats the local time with strftime, avoiding a `date` subprocess.
pub fn strftime_local(fmt: &str) -> Option<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
    let secs = now.as_secs() as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe { libc::localtime_r(&secs, &mut tm) };

    let fmt_c = std::ffi::CString::new(fmt).ok()?;
    let mut out = vec![0u8; 128];
    let n = unsafe {
        libc::strftime(
            out.as_mut_ptr() as *mut libc::c_char,
            out.len(),
            fmt_c.as_ptr(),
            &tm,
        )
    };
    if n == 0 {
        return None;
    }
    out.truncate(n);
    String::from_utf8(out).ok()
}

pub fn hostname() -> String {
    let mut buf = [0u8; 64];
    if unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) } == 0 {
        let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        if end > 0
            && let Ok(s) = std::str::from_utf8(&buf[..end])
        {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    read_trim("/proc/sys/kernel/hostname")
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "unknown".into())
}

/// Resolves a uid to a login name via getpwuid.
pub fn user_name(uid: u32) -> Option<String> {
    let pw = unsafe { libc::getpwuid(uid) };
    if pw.is_null() {
        return None;
    }
    let name = unsafe { std::ffi::CStr::from_ptr((*pw).pw_name) }
        .to_str()
        .ok()?;
    Some(name.to_string())
}

pub fn username() -> String {
    for key in ["USER", "LOGNAME"] {
        if let Ok(v) = std::env::var(key)
            && !v.is_empty()
        {
            return v;
        }
    }
    user_name(unsafe { libc::getuid() }).unwrap_or_else(|| "user".into())
}

// --- Memory totals from /proc/meminfo ---

#[derive(Debug, Default, Clone, Copy)]
pub struct MemInfo {
    pub total_kb: u64,
    pub free_kb: u64,
    pub available_kb: u64,
    pub buffers_kb: u64,
    pub cached_kb: u64,
    pub shared_kb: u64,
    pub swap_total_kb: u64,
    pub swap_free_kb: u64,
}

pub fn meminfo() -> MemInfo {
    static CACHED: std::sync::OnceLock<MemInfo> = std::sync::OnceLock::new();
    *CACHED.get_or_init(|| {
        let mut m = MemInfo::default();
        let mut buf = [0u8; 4096];
        let Some(n) = read_bytes_into(Path::new("/proc/meminfo"), &mut buf) else {
            return m;
        };
        let Ok(content) = std::str::from_utf8(&buf[..n]) else {
            return m;
        };
        let mut mask = 0u8;
        for line in content.lines() {
            let Some((k, v)) = line.split_once(':') else {
                continue;
            };
            #[inline(always)]
            fn parse_first_num(s: &str) -> u64 {
                let bytes = s.trim_start().as_bytes();
                let mut n = 0u64;
                for &b in bytes {
                    if b.is_ascii_digit() {
                        n = n * 10 + (b - b'0') as u64;
                    } else {
                        break;
                    }
                }
                n
            }
            match k {
                "MemTotal" => {
                    m.total_kb = parse_first_num(v);
                    mask |= 1 << 0;
                }
                "MemFree" => {
                    m.free_kb = parse_first_num(v);
                    mask |= 1 << 1;
                }
                "MemAvailable" => {
                    m.available_kb = parse_first_num(v);
                    mask |= 1 << 2;
                }
                "Buffers" => {
                    m.buffers_kb = parse_first_num(v);
                    mask |= 1 << 3;
                }
                "Cached" => {
                    m.cached_kb = parse_first_num(v);
                    mask |= 1 << 4;
                }
                "Shmem" => {
                    m.shared_kb = parse_first_num(v);
                    mask |= 1 << 5;
                }
                "SwapTotal" => {
                    m.swap_total_kb = parse_first_num(v);
                    mask |= 1 << 6;
                }
                "SwapFree" => {
                    m.swap_free_kb = parse_first_num(v);
                    mask |= 1 << 7;
                }
                _ => {}
            }
            if mask == 0xFF {
                break;
            }
        }
        m
    })
}

// --- Per-boot memo, rebuilt after a reboot ---

pub fn cache_get(key: &str) -> Option<String> {
    let mut path = [0u8; 64];
    let prefix = b"/dev/shm/.omnifetch_";
    if prefix.len() + key.len() >= path.len() {
        return None;
    }
    path[..prefix.len()].copy_from_slice(prefix);
    path[prefix.len()..prefix.len() + key.len()].copy_from_slice(key.as_bytes());
    let s = std::str::from_utf8(&path[..prefix.len() + key.len()]).ok()?;
    let val = fs::read_to_string(s).ok()?;
    let trimmed = val.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn cache_set(key: &str, val: &str) {
    let mut path = [0u8; 64];
    let prefix = b"/dev/shm/.omnifetch_";
    if prefix.len() + key.len() >= path.len() {
        return;
    }
    path[..prefix.len()].copy_from_slice(prefix);
    path[prefix.len()..prefix.len() + key.len()].copy_from_slice(key.as_bytes());
    if let Ok(s) = std::str::from_utf8(&path[..prefix.len() + key.len()]) {
        let _ = fs::write(s, val);
    }
}

// --- Terminal identity, size and capabilities ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
    pub xpixel: u16,
    pub ypixel: u16,
}

pub fn terminal_size() -> Option<TerminalSize> {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    let r = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) };
    if r == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
        Some(TerminalSize {
            cols: ws.ws_col,
            rows: ws.ws_row,
            xpixel: ws.ws_xpixel,
            ypixel: ws.ws_ypixel,
        })
    } else {
        None
    }
}

pub fn supports_color() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if let Ok(force) = std::env::var("OMNIFETCH_FORCE_COLOR") {
        return force != "0";
    }
    if std::env::var("TERM").map(|t| t == "dumb").unwrap_or(false) {
        return false;
    }
    if std::env::var_os("CI").is_some() {
        return false;
    }
    unsafe { libc::isatty(libc::STDOUT_FILENO) == 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter_procs_finds_this_process() {
        let me = std::process::id();
        let all: Vec<ProcEntry> = iter_procs().collect();
        let found = all.iter().find(|p| p.pid == me).expect("self not listed");
        assert!(found.threads >= 1, "thread count was {}", found.threads);
    }

    #[test]
    fn proc_count_is_plausible() {
        let all: Vec<ProcEntry> = iter_procs().collect();
        assert!(all.len() > 1, "only saw {} processes", all.len());
        assert!(
            all.iter().all(|p| p.threads >= 1),
            "a process reported zero threads"
        );
    }

    #[test]
    fn parses_tty_and_threads_from_stat_layout() {
        // First token is field 3, hence index N-3 for field N.
        let rest = "S 1 2 3 34816 4 5 6 7 8 9 10 11 12 13 14 15 999";
        let toks: Vec<&str> = rest.split_ascii_whitespace().collect();
        assert_eq!(toks[4], "34816", "tty_nr is field 7, rest index 4");
        assert_eq!(toks[17], "999", "num_threads is field 20, rest index 17");
    }

    #[test]
    fn human_duration_picks_the_largest_useful_unit() {
        let d = |s: u64| human_duration(Duration::from_secs(s));
        assert_eq!(d(0), "0s");
        assert_eq!(d(45), "45s");
        assert_eq!(d(90), "1m 30s");
        assert_eq!(d(3_661), "1h 1m");
        assert_eq!(d(86_400 + 3_600 + 60), "1d 1h 1m");
        // Seconds are only shown alongside minutes, never next to hours.
        assert!(!d(7_200).contains('s'));
    }

    #[test]
    fn human_bytes_scales_and_keeps_precision_useful() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(1023), "1023 B");
        assert_eq!(human_bytes(1024), "1.00 KiB");
        assert_eq!(human_bytes(1024 * 1024), "1.00 MiB");
        // Precision steps down as the number grows.
        assert_eq!(human_bytes(10 * 1024), "10.0 KiB");
        assert_eq!(human_bytes(500 * 1024), "500 KiB");
        // Never overflows the unit table.
        let huge = u64::MAX;
        assert!(human_bytes(huge).ends_with("PiB"));
    }
}

#[cfg(test)]
mod read_small_tests {
    use super::*;

    #[test]
    fn reads_a_proc_file() {
        let mut buf = [0u8; 128];
        let s = read_small(b"/proc/self/statm\0", &mut buf).expect("statm should read");
        assert!(s.split_ascii_whitespace().count() >= 2, "got {s:?}");
    }

    #[test]
    fn missing_path_and_overlong_file_return_none() {
        let mut buf = [0u8; 128];
        assert!(read_small(b"/nope/nope\0", &mut buf).is_none());
        let mut tiny = [0u8; 4];
        assert!(read_small(b"/proc/self/statm\0", &mut tiny).is_none());
    }
}

/// Resolves `host:port` under a deadline: `to_socket_addrs` blocks for the whole retry budget.
pub fn resolve_bounded(host: &str, port: u16, timeout: Duration) -> Option<SocketAddr> {
    let (tx, rx) = mpsc::channel();
    let host = host.to_owned();
    // Nothing can interrupt an in-flight lookup, hence a detached thread.
    std::thread::spawn(move || {
        let first = (host.as_str(), port)
            .to_socket_addrs()
            .ok()
            .and_then(|mut a| a.next());
        let _ = tx.send(first);
    });
    rx.recv_timeout(timeout).ok().flatten()
}

#[cfg(test)]
mod resolve_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn resolve_returns_a_usable_address() {
        let addr = resolve_bounded("localhost", 80, Duration::from_secs(2))
            .expect("localhost should resolve");
        assert_eq!(addr.port(), 80);
        assert!(addr.ip().is_loopback());
    }

    #[test]
    fn resolve_deadline_is_actually_enforced() {
        // No real lookup finishes inside 1 ms, so only the deadline can return here.
        let start = Instant::now();
        let out = resolve_bounded("localhost", 80, Duration::from_millis(1));
        let elapsed = start.elapsed();
        // Either answer is fine, but it has to come back on time.
        assert!(
            elapsed < Duration::from_secs(1),
            "resolve_bounded blocked for {elapsed:?}"
        );
        let _ = out;
    }

    #[test]
    fn resolve_returns_none_for_an_invalid_name() {
        let start = Instant::now();
        let out = resolve_bounded("invalid.invalid", 80, Duration::from_millis(300));
        assert!(out.is_none());
        assert!(start.elapsed() < Duration::from_secs(2));
    }
}
