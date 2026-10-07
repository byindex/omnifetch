use crate::cache::StaticCache;
use crate::config;
use crate::module::{Module, ModuleOutput};
use crate::template;

pub struct Disk;

impl Module for Disk {
    fn name(&self) -> &'static str {
        "Disk"
    }
    fn id(&self) -> &'static str {
        "disk"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut mounts = get_mounts();
        mounts.sort_by_key(|m| std::cmp::Reverse(m.total));
        mounts.truncate(4);
        if mounts.is_empty() {
            return None;
        }
        let max_mp = mounts.iter().map(|m| m.mountpoint.len()).max().unwrap_or(0);
        let tmpl = config::get()
            .format("disk")
            .unwrap_or("{mount} {used} / {total} {bar} ({pct}%)");

        if !tmpl.contains('{') {
            return Some(ModuleOutput::new("Disk", tmpl.to_string()));
        }

        let lines: Vec<String> = mounts
            .iter()
            .map(|m| {
                let free = m.total.saturating_sub(m.used);
                let padded_mp = format!("{:<max_mp$}", m.mountpoint);
                let mut ctx = template::Context::new("disk").with_metric(m.used, m.total, m.pct);
                ctx.set_str("mount", padded_mp.clone());
                ctx.set_str("m", padded_mp);
                ctx.set_bytes("used", m.used);
                ctx.set_bytes("u", m.used);
                ctx.set_bytes("total", m.total);
                ctx.set_bytes("t", m.total);
                ctx.set_bytes("free", free);
                ctx.set_bytes("f", free);
                ctx.set_num("pct", m.pct);
                ctx.set_num("p", m.pct);

                template::render_template(tmpl, &ctx)
            })
            .collect();
        Some(ModuleOutput::multi("Disk", lines))
    }
}

pub struct Mount {
    pub mountpoint: String,
    pub used: u64,
    pub total: u64,
    pub pct: f64,
}

const SKIP_FS: &[&str] = &[
    "tmpfs",
    "devtmpfs",
    "proc",
    "sysfs",
    "cgroup",
    "cgroup2",
    "overlay",
    "squashfs",
    "ramfs",
    "devpts",
    "securityfs",
    "debugfs",
    "tracefs",
    "configfs",
    "fusectl",
    "bpf",
    "autofs",
    "mqueue",
    "hugetlbfs",
    "pstore",
    "binfmt_misc",
    "rpc_pipefs",
    "nsfs",
    "efivarfs",
    "iso9660",
    "fuse.sshfs",
];

fn stat_mountpoint(mountpoint: &str) -> Option<Mount> {
    let mut buf = [0u8; 512];
    if mountpoint.len() >= buf.len() {
        return None;
    }
    buf[..mountpoint.len()].copy_from_slice(mountpoint.as_bytes());
    buf[mountpoint.len()] = 0;
    let mut st: libc::statfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statfs(buf.as_ptr() as *const libc::c_char, &mut st) } != 0 {
        return None;
    }
    let bsize = if st.f_frsize > 0 {
        st.f_frsize as u64
    } else {
        st.f_bsize as u64
    };
    let total = st.f_blocks as u64 * bsize;
    let free = st.f_bfree as u64 * bsize;
    if total < 256 * 1024 * 1024 {
        return None;
    }
    let used = total.saturating_sub(free);
    Some(Mount {
        mountpoint: mountpoint.to_string(),
        used,
        total,
        pct: used as f64 / total as f64 * 100.0,
    })
}

pub fn detect_mountpoints() -> Vec<String> {
    let Ok(content) = std::fs::read_to_string("/proc/self/mounts") else {
        return vec!["/".into()];
    };
    let mut mps = Vec::new();
    let mut seen_devs = std::collections::HashSet::new();

    for line in content.lines() {
        let mut it = line.split_whitespace();
        let (Some(device), Some(mountpoint), Some(fstype)) = (it.next(), it.next(), it.next())
        else {
            continue;
        };
        if !device.starts_with("/dev/") || SKIP_FS.contains(&fstype) {
            continue;
        }
        if !seen_devs.insert(device) {
            continue;
        }
        mps.push(mountpoint.to_string());
    }
    if mps.is_empty() {
        mps.push("/".into());
    }
    mps
}

fn get_mounts() -> Vec<Mount> {
    if let Some(cache) = StaticCache::get()
        && !cache.mountpoints.is_empty()
    {
        let mut v = Vec::new();
        for mp in &cache.mountpoints {
            if let Some(m) = stat_mountpoint(mp) {
                v.push(m);
            }
        }
        if !v.is_empty() {
            return v;
        }
    }

    let mps = detect_mountpoints();
    mps.into_iter()
        .filter_map(|mp| stat_mountpoint(&mp))
        .collect()
}
