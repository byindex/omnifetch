use crate::module::{Module, ModuleOutput};

pub struct Dns;

impl Module for Dns {
    fn name(&self) -> &'static str {
        "DNS"
    }
    fn id(&self) -> &'static str {
        "dns"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut servers: Vec<String> = Vec::new();
        if let Ok(res) = std::fs::read_to_string("/etc/resolv.conf") {
            for line in res.lines() {
                let line = line.trim();
                let Some(rest) = line.strip_prefix("nameserver") else {
                    continue;
                };
                if let Some(ip) = rest.split_whitespace().next()
                    && !servers.iter().any(|s| s == ip)
                {
                    servers.push(ip.to_string());
                }
            }
        }
        if servers.is_empty() {
            return None;
        }
        Some(ModuleOutput::new("DNS", servers.join(", ")))
    }
}
