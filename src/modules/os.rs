use crate::module::{Module, ModuleOutput};
use crate::sys;

pub struct Os;

impl Module for Os {
    fn name(&self) -> &'static str {
        "OS"
    }
    fn id(&self) -> &'static str {
        "os"
    }
    fn run(&self) -> Option<ModuleOutput> {
        #[cfg(target_os = "macos")]
        {
            let r = sys::os_release();
            let ver = crate::cmd::run("sw_vers", &["-productVersion"]).unwrap_or_default();
            let name = r.get("PRODUCT_NAME").unwrap_or("macOS").to_string();
            return Some(ModuleOutput::new(
                "OS",
                format!(
                    "{name} {ver} [{}]",
                    r.get("PRODUCT_BUILD_VERSION").unwrap_or("")
                ),
            ));
        }
        #[cfg(not(target_os = "macos"))]
        {
            let r = sys::os_release();
            let mut v = r.name();
            if let Some(ver) = r.version() {
                v.push(' ');
                v.push_str(&ver);
            }
            let arch = r.arch().unwrap_or_else(uname_arch);
            v.reserve(arch.len() + 3);
            v.push_str(" [");
            v.push_str(&arch);
            v.push(']');
            Some(ModuleOutput::new("OS", v))
        }
    }
}

fn uname_arch() -> String {
    crate::utsname::uname_ref()
        .map(|u| u.machine.clone())
        .unwrap_or_else(|| "unknown".into())
}
