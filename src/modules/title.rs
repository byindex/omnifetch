use crate::config;
use crate::module::{Module, ModuleOutput};
use crate::sys;
use crate::template;

pub struct Title;

impl Module for Title {
    fn name(&self) -> &'static str {
        "Title"
    }
    fn id(&self) -> &'static str {
        "title"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let user = sys::username();
        let host = sys::hostname();
        let tmpl = config::get().format("title").unwrap_or("{user}@{host}");

        let mut ctx = template::Context::new("title");
        ctx.set_str("user", &user);
        ctx.set_str("u", &user);
        ctx.set_str("host", &host);
        ctx.set_str("h", &host);

        let formatted = template::render_template(tmpl, &ctx);
        Some(ModuleOutput::new("Title", formatted))
    }
}
