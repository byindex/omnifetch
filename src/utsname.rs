use std::mem::MaybeUninit;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Utsname {
    pub sysname: String,
    pub nodename: String,
    pub release: String,
    pub version: String,
    pub machine: String,
}

pub fn uname_ref() -> Option<&'static Utsname> {
    static CACHED: OnceLock<Option<Utsname>> = OnceLock::new();
    CACHED
        .get_or_init(|| {
            let mut buf = MaybeUninit::<libc::utsname>::uninit();
            if unsafe { libc::uname(buf.as_mut_ptr()) } != 0 {
                return None;
            }
            let u = unsafe { buf.assume_init() };
            fn f(v: &[libc::c_char]) -> String {
                let end = v.iter().position(|&c| c == 0).unwrap_or(v.len());
                let bytes = unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, end) };
                std::str::from_utf8(bytes).unwrap_or("").to_string()
            }
            Some(Utsname {
                sysname: f(&u.sysname),
                nodename: f(&u.nodename),
                release: f(&u.release),
                version: f(&u.version),
                machine: f(&u.machine),
            })
        })
        .as_ref()
}

pub fn uname() -> Option<Utsname> {
    uname_ref().cloned()
}
