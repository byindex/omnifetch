use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::module::ModuleOutput;

pub const CACHE_VERSION: u32 = 13;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WeatherCache {
    pub host: String,
    pub resolved_addr: Option<String>,
    pub direct_failed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticCache {
    pub version: u32,
    pub boot_id: String,
    pub config_hash: u64,
    pub os_mtime: u64,
    pub packages_mtime: u64,
    pub desktop_mtime: u64,
    pub devenv_mtime: u64,
    pub audio_mtime: u64,
    pub normal_logo: Option<String>,
    pub mini_logo: Option<String>,
    pub mountpoints: Vec<String>,
    pub has_battery: Option<bool>,
    pub cputemp_path: Option<String>,
    pub weather_cache: Option<WeatherCache>,
    pub modules: HashMap<String, Option<ModuleOutput>>,
}

/// Modules that do not drift within a session.
pub fn is_static(id: &str) -> bool {
    matches!(
        id,
        "separator"
            | "break"
            | "os"
            | "kernel"
            | "host"
            | "board"
            | "bios"
            | "chassis"
            | "initsystem"
            | "cpu"
            | "cpucache"
            | "gpu"
            | "resolution"
            | "sound"
            | "font"
            | "theme"
            | "icons"
            | "cursor"
            | "wmtheme"
            | "vulkan"
            | "opengl"
            | "packages"
            | "devenv"
            | "display"
            | "bootmgr"
            | "tpm"
            | "powerprofile"
            | "security"
            | "gpudriver"
            | "displayserver"
            | "audioserver"
            | "netadapter"
            | "localip"
            | "dns"
            | "bluetoothradio"
            | "physicaldisk"
            | "physicalmemory"
            | "poweradapter"
            | "lm"
            | "opencl"
            | "codec"
            | "btrfs"
            | "zpool"
            | "terminaltheme"
            | "monitor"
            | "custom"
    )
}

fn cache_path() -> PathBuf {
    let uid = unsafe { libc::getuid() };
    let shm = Path::new("/dev/shm");
    if shm.is_dir() {
        shm.join(format!(".omnifetch_cache_{uid}.bin"))
    } else {
        std::env::temp_dir().join(format!(".omnifetch_cache_{uid}.bin"))
    }
}

pub fn get_boot_id() -> String {
    let mut buf = [0u8; 64];
    let p = std::path::Path::new("/proc/sys/kernel/random/boot_id");
    if let Some(n) = crate::sys::read_bytes_into(p, &mut buf)
        && let Ok(s) = std::str::from_utf8(&buf[..n])
    {
        return s.trim().to_string();
    }
    fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// System facts a cached value depends on. Compared against what the cache recorded.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fingerprint {
    pub boot_id: String,
    pub config_hash: u64,
    pub os_mtime: u64,
    pub packages_mtime: u64,
    pub desktop_mtime: u64,
    /// Toolchain mtime. 0 while unresolved, since finding it stats every binary on PATH.
    pub devenv_mtime: u64,
    pub audio_mtime: u64,
    pub weather_host: String,
}

impl Fingerprint {
    pub fn capture() -> Self {
        Self {
            boot_id: get_boot_id(),
            config_hash: crate::config::get().hash_value(),
            os_mtime: detect_os_mtime(),
            packages_mtime: detect_packages_mtime(),
            desktop_mtime: detect_desktop_mtime(),
            devenv_mtime: 0,
            audio_mtime: detect_audio_mtime(),
            weather_host: crate::config::get().network_cfg.weather_host.clone(),
        }
    }
}

#[inline]
fn stat_mtime(path_bytes: &[u8]) -> Option<u64> {
    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::stat(path_bytes.as_ptr() as *const libc::c_char, &mut st) } == 0 {
        Some(st.st_mtime as u64)
    } else {
        None
    }
}

pub fn detect_os_mtime() -> u64 {
    const PATHS: &[&[u8]] = &[
        b"/etc/os-release\0",
        b"/usr/lib/os-release\0",
        b"/etc/issue\0",
    ];
    for &p in PATHS {
        if let Some(m) = stat_mtime(p) {
            return m;
        }
    }
    0
}

pub fn detect_packages_mtime() -> u64 {
    let mut max_m = 0;

    const PRIMARY_PMS: &[&[u8]] = &[
        b"/var/lib/pacman/local\0",
        b"/var/lib/dpkg/status\0",
        b"/var/lib/rpm\0",
        b"/lib/apk/db/installed\0",
        b"/var/db/xbps\0",
        b"/var/db/pkg\0",
        b"/nix/var/nix/profiles/default\0",
    ];

    for &p in PRIMARY_PMS {
        if let Some(m) = stat_mtime(p) {
            max_m = max_m.max(m);
            break;
        }
    }

    const SECONDARY: &[&[u8]] = &[
        b"/var/lib/flatpak/app\0",
        b"/var/lib/snapd/snaps\0",
        b"/home/linuxbrew/.linuxbrew/Cellar\0",
    ];
    for &p in SECONDARY {
        if let Some(m) = stat_mtime(p) {
            max_m = max_m.max(m);
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        use std::os::unix::ffi::OsStrExt;
        let home_b = home.as_bytes();
        let mut path_buf = [0u8; 256];
        if home_b.len() < 180 {
            path_buf[..home_b.len()].copy_from_slice(home_b);
            let hlen = home_b.len();
            const SUBS: &[&[u8]] = &[
                b"/.local/share/flatpak/app\0",
                b"/.linuxbrew/Cellar\0",
                b"/.nix-profile\0",
            ];
            for &sub in SUBS {
                path_buf[hlen..hlen + sub.len()].copy_from_slice(sub);
                if let Some(m) = stat_mtime(&path_buf[..hlen + sub.len()]) {
                    max_m = max_m.max(m);
                }
            }
        }
    }

    max_m
}

pub fn detect_desktop_mtime() -> u64 {
    let Some(home) = std::env::var_os("HOME") else {
        return 0;
    };
    use std::os::unix::ffi::OsStrExt;
    let home_b = home.as_bytes();
    if home_b.is_empty() || home_b.len() > 180 {
        return 0;
    }
    let mut path_buf = [0u8; 256];
    path_buf[..home_b.len()].copy_from_slice(home_b);
    let hlen = home_b.len();

    let mut max_m = 0;
    const SUBPATHS: &[&[u8]] = &[
        b"/.config/kdeglobals\0",
        b"/.config/plasmarc\0",
        b"/.config/gtk-3.0/settings.ini\0",
        b"/.config/gtk-4.0/settings.ini\0",
        b"/.config/dconf/user\0",
        b"/.config/xfce4/xfconf/xfce-perchannel-xml/xsettings.xml\0",
        b"/.config/hypr/hyprland.conf\0",
        b"/.config/sway/config\0",
        b"/.config/fontconfig/fonts.conf\0",
        b"/.icons/default/index.theme\0",
        b"/.local/share/icons\0",
        b"/.local/share/themes\0",
        b"/.Xresources\0",
    ];

    for sub in SUBPATHS {
        path_buf[hlen..hlen + sub.len()].copy_from_slice(sub);
        if let Some(m) = stat_mtime(&path_buf[..hlen + sub.len()]) {
            max_m = max_m.max(m);
        }
    }
    max_m
}

fn find_in_path(cmd: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(cmd);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub fn detect_devenv_mtime() -> u64 {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut max_m = 0;

    for bin in ["rustc", "python3", "python", "node", "go", "gcc", "clang"] {
        if let Some(p) = find_in_path(bin)
            && let Ok(meta) = fs::metadata(&p)
            && let Ok(mod_time) = meta.modified()
            && let Ok(d) = mod_time.duration_since(std::time::UNIX_EPOCH)
        {
            max_m = max_m.max(d.as_secs());
        }
    }

    if !home.is_empty() {
        let h = Path::new(&home);
        let paths = [
            h.join(".cargo/bin/rustc"),
            h.join(".rustup/settings.toml"),
            h.join(".rustup/update-hashes"),
            h.join(".nvm"),
            h.join(".pyenv/version"),
        ];
        for p in paths {
            if let Ok(meta) = fs::metadata(&p)
                && let Ok(mod_time) = meta.modified()
                && let Ok(d) = mod_time.duration_since(std::time::UNIX_EPOCH)
            {
                max_m = max_m.max(d.as_secs());
            }
        }
    }

    max_m
}

pub fn detect_audio_mtime() -> u64 {
    let uid = unsafe { libc::getuid() };
    let mut max_m = 0;

    let mut sock_buf = [0u8; 64];
    let prefix = b"/run/user/";
    let mut itoa_buf = [0u8; 10];
    let mut n = uid;
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
    let uid_b = &itoa_buf[i..];
    sock_buf[..prefix.len()].copy_from_slice(prefix);
    sock_buf[prefix.len()..prefix.len() + uid_b.len()].copy_from_slice(uid_b);
    let base_len = prefix.len() + uid_b.len();

    const SOCKS: &[&[u8]] = &[b"/pipewire-0\0", b"/pulse/native\0"];
    for &sock in SOCKS {
        sock_buf[base_len..base_len + sock.len()].copy_from_slice(sock);
        if let Some(m) = stat_mtime(&sock_buf[..base_len + sock.len()]) {
            max_m = max_m.max(m);
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        use std::os::unix::ffi::OsStrExt;
        let home_b = home.as_bytes();
        if !home_b.is_empty() && home_b.len() < 180 {
            let mut path_buf = [0u8; 256];
            path_buf[..home_b.len()].copy_from_slice(home_b);
            let hlen = home_b.len();
            const WP_PATHS: &[&[u8]] = &[
                b"/.local/state/wireplumber/stream-properties\0",
                b"/.local/state/wireplumber/default-routes\0",
                b"/.local/state/wireplumber/default-nodes\0",
            ];
            for &wp in WP_PATHS {
                path_buf[hlen..hlen + wp.len()].copy_from_slice(wp);
                if let Some(m) = stat_mtime(&path_buf[..hlen + wp.len()]) {
                    max_m = max_m.max(m);
                }
            }
        }
    }

    max_m
}

impl Default for StaticCache {
    fn default() -> Self {
        Self::new()
    }
}

impl StaticCache {
    pub fn new() -> Self {
        let fp = Fingerprint::capture();
        Self::with_fingerprint(&fp)
    }

    /// A fresh entry. Toolchain mtime waits at 0 until a `devenv` value needs it.
    pub fn with_fingerprint(fp: &Fingerprint) -> Self {
        StaticCache {
            version: CACHE_VERSION,
            boot_id: fp.boot_id.clone(),
            config_hash: fp.config_hash,
            os_mtime: fp.os_mtime,
            packages_mtime: fp.packages_mtime,
            desktop_mtime: fp.desktop_mtime,
            devenv_mtime: 0,
            audio_mtime: fp.audio_mtime,
            normal_logo: None,
            mini_logo: None,
            mountpoints: Vec::new(),
            has_battery: None,
            cputemp_path: None,
            weather_cache: None,
            modules: HashMap::new(),
        }
    }

    pub fn get_cputemp_path() -> Option<String> {
        Self::get().and_then(|c| c.cputemp_path.clone())
    }

    pub fn load() -> Option<Self> {
        let bytes = fs::read(cache_path()).ok()?;
        let cache = Self::reconcile(&bytes, &Fingerprint::capture())?;
        if let Some(ref w) = cache.weather_cache
            && let Ok(mut lock) = WEATHER_CACHE.write()
        {
            *lock = Some(w.clone());
        }
        Some(cache)
    }

    /// Drops whatever `fp` marks stale. `None` when nothing survives.
    pub fn reconcile(bytes: &[u8], fp: &Fingerprint) -> Option<Self> {
        let mut cache = StaticCache::from_bytes(bytes)?;
        if cache.version != CACHE_VERSION {
            return None;
        }
        if !fp.boot_id.is_empty() && cache.boot_id != fp.boot_id {
            return None;
        }
        if cache.config_hash != fp.config_hash {
            cache.modules.clear();
            cache.weather_cache = None;
            cache.config_hash = fp.config_hash;
        }

        if let Some(ref w) = cache.weather_cache
            && w.host != fp.weather_host
        {
            cache.weather_cache = None;
        }

        if cache.modules.contains_key("os") && fp.os_mtime > 0 && cache.os_mtime != fp.os_mtime {
            cache.modules.remove("os");
            cache.normal_logo = None;
            cache.mini_logo = None;
            cache.os_mtime = fp.os_mtime;
        }

        let pkg_related = ["packages", "vulkan", "opengl", "devenv"];
        if pkg_related.iter().any(|id| cache.modules.contains_key(*id))
            && fp.packages_mtime > 0
            && cache.packages_mtime != fp.packages_mtime
        {
            for id in pkg_related {
                cache.modules.remove(id);
            }
            cache.packages_mtime = fp.packages_mtime;
        }

        let desktop_related = ["theme", "icons", "font", "cursor", "wmtheme"];
        if desktop_related
            .iter()
            .any(|id| cache.modules.contains_key(*id))
            && fp.desktop_mtime > 0
            && cache.desktop_mtime != fp.desktop_mtime
        {
            for id in desktop_related {
                cache.modules.remove(id);
            }
            cache.desktop_mtime = fp.desktop_mtime;
        }

        if cache.modules.contains_key("devenv") {
            // Reached only when `devenv` itself is cached.
            let current = if fp.devenv_mtime > 0 {
                fp.devenv_mtime
            } else {
                detect_devenv_mtime()
            };
            if current > 0 && cache.devenv_mtime != current {
                cache.modules.remove("devenv");
                cache.devenv_mtime = current;
            }
        }

        if cache.modules.contains_key("sound")
            && fp.audio_mtime > 0
            && cache.audio_mtime != fp.audio_mtime
        {
            cache.modules.remove("sound");
            cache.audio_mtime = fp.audio_mtime;
        }

        Some(cache)
    }

    pub fn get() -> Option<&'static Self> {
        static CACHED: std::sync::OnceLock<Option<StaticCache>> = std::sync::OnceLock::new();
        CACHED.get_or_init(Self::load).as_ref()
    }

    pub fn get_weather_cache() -> Option<WeatherCache> {
        if let Ok(lock) = WEATHER_CACHE.read()
            && lock.is_some()
        {
            return lock.clone();
        }
        Self::get().and_then(|c| c.weather_cache.clone())
    }

    pub fn set_weather_cache(wc: WeatherCache) {
        if let Ok(mut lock) = WEATHER_CACHE.write() {
            *lock = Some(wc.clone());
        }
        let mut cache = Self::load().unwrap_or_default();
        cache.weather_cache = Some(wc);
        cache.save();
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = BufferWriter::new();
        w.buf.extend_from_slice(b"OMNI");
        w.write_u32(self.version);
        w.write_str(&self.boot_id);
        w.write_u64(self.config_hash);
        w.write_u64(self.os_mtime);
        w.write_u64(self.packages_mtime);
        w.write_u64(self.desktop_mtime);
        w.write_u64(self.devenv_mtime);
        w.write_u64(self.audio_mtime);
        w.write_opt_str(self.normal_logo.as_deref());
        w.write_opt_str(self.mini_logo.as_deref());

        w.write_u32(self.mountpoints.len() as u32);
        for m in &self.mountpoints {
            w.write_str(m);
        }

        match self.has_battery {
            None => w.write_u8(0),
            Some(false) => w.write_u8(1),
            Some(true) => w.write_u8(2),
        }

        w.write_opt_str(self.cputemp_path.as_deref());

        match &self.weather_cache {
            None => w.write_u8(0),
            Some(wc) => {
                w.write_u8(1);
                w.write_str(&wc.host);
                w.write_opt_str(wc.resolved_addr.as_deref());
                w.write_u8(if wc.direct_failed { 1 } else { 0 });
            }
        }

        w.write_u32(self.modules.len() as u32);
        for (k, opt_out) in &self.modules {
            w.write_str(k);
            match opt_out {
                None => w.write_u8(0),
                Some(out) => {
                    w.write_u8(1);
                    w.write_str(&out.name);
                    w.write_u32(out.fields.len() as u32);
                    for f in &out.fields {
                        w.write_str(&f.label);
                        w.write_str(&f.value);
                    }
                }
            }
        }

        w.buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut r = BufferReader::new(bytes);
        if r.slice.get(r.pos..r.pos + 4)? != b"OMNI" {
            return None;
        }
        r.pos += 4;

        let version = r.read_u32()?;
        let boot_id = r.read_str()?;
        let config_hash = r.read_u64()?;
        let os_mtime = r.read_u64()?;
        let packages_mtime = r.read_u64()?;
        let desktop_mtime = r.read_u64()?;
        let devenv_mtime = r.read_u64()?;
        let audio_mtime = r.read_u64()?;
        let normal_logo = r.read_opt_str()?;
        let mini_logo = r.read_opt_str()?;

        let mcount = r.read_u32()? as usize;
        let mut mountpoints = Vec::with_capacity(mcount);
        for _ in 0..mcount {
            mountpoints.push(r.read_str()?);
        }

        let has_battery = match r.read_u8()? {
            0 => None,
            1 => Some(false),
            2 => Some(true),
            _ => return None,
        };

        let cputemp_path = r.read_opt_str()?;

        let weather_cache = match r.read_u8()? {
            0 => None,
            1 => {
                let host = r.read_str()?;
                let resolved_addr = r.read_opt_str()?;
                let direct_failed = r.read_u8()? != 0;
                Some(WeatherCache {
                    host,
                    resolved_addr,
                    direct_failed,
                })
            }
            _ => return None,
        };

        let mod_count = r.read_u32()? as usize;
        let mut modules = HashMap::with_capacity(mod_count);
        for _ in 0..mod_count {
            let k = r.read_str()?;
            let opt_out = match r.read_u8()? {
                0 => None,
                1 => {
                    let name = r.read_str()?;
                    let fcount = r.read_u32()? as usize;
                    let mut fields = Vec::with_capacity(fcount);
                    for _ in 0..fcount {
                        let label = r.read_str()?;
                        let value = r.read_str()?;
                        fields.push(crate::module::Field { label, value });
                    }
                    Some(ModuleOutput { name, fields })
                }
                _ => return None,
            };
            modules.insert(k, opt_out);
        }

        Some(StaticCache {
            version,
            boot_id,
            config_hash,
            os_mtime,
            packages_mtime,
            desktop_mtime,
            devenv_mtime,
            audio_mtime,
            normal_logo,
            mini_logo,
            mountpoints,
            has_battery,
            cputemp_path,
            weather_cache,
            modules,
        })
    }

    pub fn save(&self) {
        Self::save_to(self, &cache_path());
    }

    /// Temp file plus `rename`, so concurrent runs never read a half-written cache.
    fn save_to(&self, path: &Path) {
        let bytes = self.to_bytes();
        let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
        if fs::write(&tmp, &bytes).is_ok() && fs::rename(&tmp, path).is_ok() {
            return;
        }
        let _ = fs::remove_file(&tmp);
    }

    pub fn clear() {
        let _ = fs::remove_file(cache_path());
    }
}

struct BufferWriter {
    buf: Vec<u8>,
}

impl BufferWriter {
    fn new() -> Self {
        Self {
            buf: Vec::with_capacity(1024),
        }
    }

    fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    fn write_u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn write_u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn write_str(&mut self, s: &str) {
        self.write_u32(s.len() as u32);
        self.buf.extend_from_slice(s.as_bytes());
    }

    fn write_opt_str(&mut self, s: Option<&str>) {
        match s {
            Some(val) => {
                self.write_u8(1);
                self.write_str(val);
            }
            None => self.write_u8(0),
        }
    }
}

struct BufferReader<'a> {
    slice: &'a [u8],
    pos: usize,
}

impl<'a> BufferReader<'a> {
    fn new(slice: &'a [u8]) -> Self {
        Self { slice, pos: 0 }
    }

    fn read_u8(&mut self) -> Option<u8> {
        let b = *self.slice.get(self.pos)?;
        self.pos += 1;
        Some(b)
    }

    fn read_u32(&mut self) -> Option<u32> {
        let bytes = self.slice.get(self.pos..self.pos + 4)?;
        self.pos += 4;
        Some(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    fn read_u64(&mut self) -> Option<u64> {
        let bytes = self.slice.get(self.pos..self.pos + 8)?;
        self.pos += 8;
        Some(u64::from_le_bytes(bytes.try_into().unwrap()))
    }

    fn read_str(&mut self) -> Option<String> {
        let len = self.read_u32()? as usize;
        let bytes = self.slice.get(self.pos..self.pos + len)?;
        self.pos += len;
        std::str::from_utf8(bytes).ok().map(|s| s.to_string())
    }

    fn read_opt_str(&mut self) -> Option<Option<String>> {
        match self.read_u8()? {
            0 => Some(None),
            1 => Some(Some(self.read_str()?)),
            _ => None,
        }
    }
}

static WEATHER_CACHE: std::sync::RwLock<Option<WeatherCache>> = std::sync::RwLock::new(None);

#[cfg(test)]
mod tests {
    use super::*;

    /// Every mtime filled in: a mismatch then means real drift, not a missing value.
    fn fp() -> Fingerprint {
        Fingerprint {
            boot_id: "boot-1".to_string(),
            config_hash: 111,
            os_mtime: 10,
            packages_mtime: 20,
            desktop_mtime: 30,
            devenv_mtime: 40,
            audio_mtime: 50,
            weather_host: "wttr.in".to_string(),
        }
    }

    fn cached(f: &Fingerprint) -> Vec<u8> {
        let mut c = StaticCache::with_fingerprint(f);
        c.devenv_mtime = f.devenv_mtime;
        for id in [
            "os", "cpu", "gpu", "packages", "vulkan", "opengl", "devenv", "theme", "icons", "font",
            "cursor", "wmtheme", "sound",
        ] {
            c.modules
                .insert(id.to_string(), Some(ModuleOutput::new(id, "value")));
        }
        c.normal_logo = Some("arch".to_string());
        c.mini_logo = Some("arch".to_string());
        c.to_bytes()
    }

    fn reconciled(f: &Fingerprint) -> Option<StaticCache> {
        StaticCache::reconcile(&cached(f), f)
    }

    #[test]
    fn matching_fingerprint_keeps_everything() {
        let c = reconciled(&fp()).expect("cache should be usable");
        assert_eq!(c.modules.len(), 13);
        assert_eq!(c.normal_logo.as_deref(), Some("arch"));
    }

    #[test]
    fn cache_from_another_boot_is_discarded() {
        let mut written = fp();
        written.boot_id = "boot-0".to_string();
        assert!(StaticCache::reconcile(&cached(&written), &fp()).is_none());
    }

    #[test]
    fn empty_boot_id_does_not_discard() {
        let mut written = fp();
        written.boot_id = "boot-0".to_string();
        let mut current = fp();
        // Kernels without boot_id leave it empty, which is not a reason to drop everything.
        current.boot_id = String::new();
        assert!(StaticCache::reconcile(&cached(&written), &current).is_some());
    }

    #[test]
    fn schema_version_mismatch_is_discarded() {
        let mut c = StaticCache::with_fingerprint(&fp());
        c.version = CACHE_VERSION + 1;
        let bytes = c.to_bytes();
        assert!(StaticCache::reconcile(&bytes, &fp()).is_none());
    }

    #[test]
    fn config_change_clears_all_modules_but_keeps_logos() {
        let mut current = fp();
        current.config_hash = 222;
        let c = StaticCache::reconcile(&cached(&fp()), &current).unwrap();
        assert!(c.modules.is_empty(), "config change must drop modules");
        assert_eq!(c.config_hash, 222, "hash is updated to the new one");
    }

    #[test]
    fn os_mtime_change_drops_os_and_both_logos() {
        let mut current = fp();
        current.os_mtime = 99;
        let c = StaticCache::reconcile(&cached(&fp()), &current).unwrap();
        assert!(!c.modules.contains_key("os"));
        assert!(c.normal_logo.is_none(), "logo depends on the distro");
        assert!(c.mini_logo.is_none());
        assert!(c.modules.contains_key("cpu"), "unrelated modules survive");
        assert_eq!(c.os_mtime, 99);
    }

    #[test]
    fn packages_mtime_change_drops_all_four_dependents() {
        let mut current = fp();
        current.packages_mtime = 77;
        let c = StaticCache::reconcile(&cached(&fp()), &current).unwrap();
        for id in ["packages", "vulkan", "opengl", "devenv"] {
            assert!(!c.modules.contains_key(id), "{id} must be dropped");
        }
        assert!(c.modules.contains_key("sound"));
        assert_eq!(c.packages_mtime, 77);
    }

    #[test]
    fn desktop_mtime_change_drops_all_five_theming_modules() {
        let mut current = fp();
        current.desktop_mtime = 88;
        let c = StaticCache::reconcile(&cached(&fp()), &current).unwrap();
        for id in ["theme", "icons", "font", "cursor", "wmtheme"] {
            assert!(!c.modules.contains_key(id), "{id} must be dropped");
        }
        assert!(c.modules.contains_key("cpu"));
        assert_eq!(c.desktop_mtime, 88);
    }

    #[test]
    fn audio_mtime_change_drops_sound_only() {
        let mut current = fp();
        current.audio_mtime = 66;
        let c = StaticCache::reconcile(&cached(&fp()), &current).unwrap();
        assert!(!c.modules.contains_key("sound"));
        assert!(c.modules.contains_key("gpu"));
        assert_eq!(c.audio_mtime, 66);
    }

    #[test]
    fn zero_mtime_is_treated_as_unknown_and_never_invalidates() {
        // 0 means the stat failed. Treating that as drift would throw away good data.
        let mut written = fp();
        written.desktop_mtime = 0;
        let mut current = fp();
        current.desktop_mtime = 0;
        let c = StaticCache::reconcile(&cached(&written), &current).unwrap();
        assert!(c.modules.contains_key("theme"));
    }

    #[test]
    fn devenv_mtime_change_drops_devenv() {
        let written = fp();
        // Only the toolchain mtime differs, isolating the devenv rule.
        let mut current = fp();
        current.devenv_mtime = 55;
        let c = StaticCache::reconcile(&cached(&written), &current).unwrap();
        assert!(!c.modules.contains_key("devenv"));
        assert!(
            c.modules.contains_key("packages"),
            "packages mtime unchanged"
        );
    }

    /// Scratch file in temp_dir, leaving the live cache in /dev/shm alone.
    fn scratch(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("omnifetch-test-{}-{name}.bin", std::process::id()));
        let _ = fs::remove_file(&p);
        p
    }

    #[test]
    fn save_writes_readable_binary_to_the_given_path() {
        let path = scratch("save");
        let mut c = StaticCache::with_fingerprint(&fp());
        c.modules
            .insert("os".to_string(), Some(ModuleOutput::new("os", "Linux")));
        c.save_to(&path);

        let bytes = fs::read(&path).expect("cache file should exist");
        let back = StaticCache::from_bytes(&bytes).expect("valid binary cache");
        assert_eq!(back.modules["os"].as_ref().unwrap().name, "os");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn save_removes_its_temporary_file() {
        let path = scratch("tmp");
        StaticCache::with_fingerprint(&fp()).save_to(&path);

        // Matched by exact name, since scanning would catch a parallel test's file.
        let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
        assert!(path.exists(), "the cache itself should be in place");
        assert!(
            !tmp.exists(),
            "temporary file was left behind: {}",
            tmp.display()
        );
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn corrupt_binary_is_discarded_rather_than_panicking() {
        assert!(StaticCache::reconcile(b"invalid binary payload", &fp()).is_none());
        assert!(StaticCache::reconcile(&[], &fp()).is_none());
    }

    #[test]
    fn dynamic_modules_are_never_served_from_cache() {
        // These change under the user, so a cached copy would just be quietly wrong.
        for id in [
            "uptime",
            "memory",
            "swap",
            "disk",
            "cputemp",
            "cpuusage",
            "datetime",
            "loadavg",
            "processes",
            "netio",
            "wifi",
            "terminalsize",
            "keyboard",
            "mouse",
            "touchpad",
            "gamepad",
        ] {
            assert!(!is_static(id), "{id} must be re-read every run");
        }
    }

    #[test]
    fn static_modules_are_classified_static() {
        for id in [
            "os",
            "cpu",
            "gpu",
            "sound",
            "bootmgr",
            "netadapter",
            "board",
        ] {
            assert!(is_static(id), "{id} should be cacheable");
        }
        assert!(!is_static("not_a_real_module"));
    }

    #[test]
    fn weather_cache_survives_when_fingerprint_matches() {
        let f = fp();
        let mut c = StaticCache::with_fingerprint(&f);
        c.weather_cache = Some(WeatherCache {
            host: "wttr.in".to_string(),
            resolved_addr: Some("5.9.243.187:80".to_string()),
            direct_failed: true,
        });
        let bytes = c.to_bytes();
        let reconciled = StaticCache::reconcile(&bytes, &f).unwrap();
        assert_eq!(
            reconciled.weather_cache,
            Some(WeatherCache {
                host: "wttr.in".to_string(),
                resolved_addr: Some("5.9.243.187:80".to_string()),
                direct_failed: true,
            })
        );
    }

    #[test]
    fn weather_cache_dropped_on_host_change() {
        let f = fp();
        let mut c = StaticCache::with_fingerprint(&f);
        c.weather_cache = Some(WeatherCache {
            host: "wttr.in".to_string(),
            resolved_addr: Some("5.9.243.187:80".to_string()),
            direct_failed: false,
        });
        let bytes = c.to_bytes();
        let mut other_fp = f;
        other_fp.weather_host = "custom.wttr.in".to_string();
        let reconciled = StaticCache::reconcile(&bytes, &other_fp).unwrap();
        assert!(reconciled.weather_cache.is_none());
    }

    #[test]
    fn weather_cache_dropped_on_config_hash_change() {
        let f = fp();
        let mut c = StaticCache::with_fingerprint(&f);
        c.weather_cache = Some(WeatherCache {
            host: "wttr.in".to_string(),
            resolved_addr: Some("5.9.243.187:80".to_string()),
            direct_failed: false,
        });
        let bytes = c.to_bytes();
        let mut other_fp = f;
        other_fp.config_hash = 999;
        let reconciled = StaticCache::reconcile(&bytes, &other_fp).unwrap();
        assert!(reconciled.weather_cache.is_none());
    }
}
