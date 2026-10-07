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
            let default_val = format!(
                "{name} {ver} [{}]",
                r.get("PRODUCT_BUILD_VERSION").unwrap_or("")
            );
            if let Some(tmpl) = crate::config::get().format("os") {
                let mut ctx = crate::template::Context::new("os");
                ctx.set_str("name", name.clone());
                ctx.set_str("n", name);
                ctx.set_str("version", ver.clone());
                ctx.set_str("v", ver);
                ctx.set_str("val", default_val.clone());
                ctx.set_str("value", default_val);
                let formatted = crate::template::render_template(tmpl, &ctx);
                return Some(ModuleOutput::new("OS", formatted));
            }
            return Some(ModuleOutput::new("OS", default_val));
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
            if let Some(tmpl) = crate::config::get().format("os") {
                let mut ctx = crate::template::Context::new("os");
                ctx.set_str("name", r.name());
                ctx.set_str("n", r.name());
                let ver = r.version().unwrap_or_default();
                ctx.set_str("version", ver.clone());
                ctx.set_str("v", ver);
                ctx.set_str("arch", arch.clone());
                ctx.set_str("a", arch);
                ctx.set_str("id", r.id());
                ctx.set_str("val", v.clone());
                ctx.set_str("value", v);
                let formatted = crate::template::render_template(tmpl, &ctx);
                return Some(ModuleOutput::new("OS", formatted));
            }
            Some(ModuleOutput::new("OS", v))
        }
    }
}

fn uname_arch() -> String {
    crate::utsname::uname_ref()
        .map(|u| u.machine.clone())
        .unwrap_or_else(|| "unknown".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_module_runs() {
        let os = Os;
        let out = os.run();
        assert!(out.is_some());
        let res = out.unwrap();
        assert_eq!(res.name, "OS");
        assert!(!res.fields.is_empty());
    }
}
