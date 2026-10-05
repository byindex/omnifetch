use crate::module::{Module, ModuleOutput};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

pub struct Weather;

impl Module for Weather {
    fn name(&self) -> &'static str {
        "Weather"
    }
    fn id(&self) -> &'static str {
        "weather"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let w = fetch_weather_raw_http()?;

        let trimmed = w.trim().to_string();
        if trimmed.is_empty() || trimmed.contains("Unknown") || trimmed.contains("html") {
            None
        } else {
            Some(ModuleOutput::new("Weather", trimmed))
        }
    }
}

/// Lightweight direct HTTP connection to configured weather host.
fn fetch_weather_raw_http() -> Option<String> {
    let cfg = crate::config::get();
    let host = &cfg.network_cfg.weather_host;
    let configured_ip = cfg.network_cfg.weather_ip.trim();

    let cached = if !cfg.no_cache {
        crate::cache::StaticCache::get_weather_cache().filter(|c| c.host == *host)
    } else {
        None
    };

    let mut direct_failed = cached.as_ref().is_some_and(|c| c.direct_failed);
    let mut resolved_addr_str = cached.as_ref().and_then(|c| c.resolved_addr.clone());

    let mut stream: Option<TcpStream> = None;

    // Fast-path connect using configured IP unless marked unreachable in cache.
    if !direct_failed
        && !configured_ip.is_empty()
        && let Ok(addr) = format!("{configured_ip}:80").parse::<SocketAddr>()
    {
        match TcpStream::connect_timeout(&addr, Duration::from_millis(250)) {
            Ok(s) => stream = Some(s),
            Err(_) => direct_failed = true,
        }
    }

    // Reuse previously resolved IP to avoid redundant DNS lookup latency.
    if stream.is_none()
        && let Some(ref addr_str) = resolved_addr_str
        && let Ok(addr) = addr_str.parse::<SocketAddr>()
        && let Ok(s) = TcpStream::connect_timeout(&addr, Duration::from_millis(300))
    {
        stream = Some(s);
    }

    // Resolve hostname within deadline when cache is empty or stale.
    if stream.is_none()
        && let Some(addr) = crate::sys::resolve_bounded(host, 80, Duration::from_millis(500))
    {
        resolved_addr_str = Some(addr.to_string());
        if let Ok(s) = TcpStream::connect_timeout(&addr, Duration::from_millis(300)) {
            stream = Some(s);
        }
    }

    // Persist discovered endpoints and connection status for subsequent runs.
    if !cfg.no_cache && (direct_failed || resolved_addr_str.is_some()) {
        let update = crate::cache::WeatherCache {
            host: host.clone(),
            resolved_addr: resolved_addr_str,
            direct_failed,
        };
        if cached.as_ref() != Some(&update) {
            crate::cache::StaticCache::set_weather_cache(update);
        }
    }

    let mut stream = stream?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(400)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(150)));

    let req = format!(
        "GET /?format=1 HTTP/1.1\r\nHost: {host}\r\nUser-Agent: curl/8.0\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(req.as_bytes()).ok()?;

    let mut buf = Vec::new();
    let mut chunk = [0u8; 1024];
    while let Ok(n) = stream.read(&mut chunk) {
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        if buf.len() > 4096 {
            break;
        }
    }

    let text = String::from_utf8_lossy(&buf);
    let body = text.split("\r\n\r\n").nth(1)?;
    let trimmed = body.trim().to_string();
    if trimmed.is_empty() || trimmed.contains('<') {
        None
    } else {
        Some(trimmed)
    }
}
