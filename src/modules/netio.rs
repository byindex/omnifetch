use crate::config;
use crate::module::{Module, ModuleOutput};
use crate::template;
use std::fs;

pub struct NetIo;

impl Module for NetIo {
    fn name(&self) -> &'static str {
        "Network I/O"
    }
    fn id(&self) -> &'static str {
        "netio"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let (rx, tx) = read_net_bytes()?;
        if rx == 0 && tx == 0 {
            return None;
        }

        let tmpl = config::get()
            .format("netio")
            .unwrap_or("{rx} (RX) / {tx} (TX)");

        let mut ctx = template::Context::new("netio");
        ctx.set_bytes("rx", rx);
        ctx.set_bytes("tx", tx);
        ctx.set_bytes("total", rx + tx);
        ctx.set_bytes("t", rx + tx);

        let formatted = template::render_template(tmpl, &ctx);
        Some(ModuleOutput::new("Network I/O", formatted))
    }
}

fn read_net_bytes() -> Option<(u64, u64)> {
    let content = fs::read_to_string("/proc/net/dev").ok()?;
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;

    for line in content.lines().skip(2) {
        let Some((iface, rest)) = line.split_once(':') else {
            continue;
        };
        let iface = iface.trim();
        if iface == "lo" {
            continue;
        }
        let mut it = rest
            .split_whitespace()
            .filter_map(|s| s.parse::<u64>().ok());
        if let Some(rx) = it.next() {
            total_rx += rx;
        }
        // TX bytes is field index 8 (skip 7 fields: packets, errs, drop, fifo, frame, compressed, multicast)
        if let Some(tx) = it.nth(7) {
            total_tx += tx;
        }
    }

    Some((total_rx, total_tx))
}
