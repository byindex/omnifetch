use crate::module::{Module, ModuleOutput};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

pub struct Containers;

impl Module for Containers {
    fn name(&self) -> &'static str {
        "Containers"
    }
    fn id(&self) -> &'static str {
        "containers"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let mut parts = Vec::new();

        if let Some(count) = count_docker_containers()
            && count > 0
        {
            parts.push(format!("{count} (docker)"));
        }

        if let Some(count) = count_podman_containers()
            && count > 0
        {
            parts.push(format!("{count} (podman)"));
        }

        if parts.is_empty() {
            // Nothing in the command line, so ask cgroups about live scopes.
            if let Some(count) = count_cgroup_containers()
                && count > 0
            {
                parts.push(format!("{count} (cgroup)"));
            }
        }

        if parts.is_empty() {
            return None;
        }

        Some(ModuleOutput::new("Containers", parts.join(", ")))
    }
}

fn query_socket_json(socket_path: &Path, request: &str) -> Option<String> {
    if !socket_path.exists() {
        return None;
    }
    let mut stream = UnixStream::connect(socket_path).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_millis(30)))
        .ok()?;
    stream
        .set_write_timeout(Some(Duration::from_millis(30)))
        .ok()?;

    stream.write_all(request.as_bytes()).ok()?;

    let mut response = Vec::with_capacity(4096);
    let mut buf = [0u8; 2048];
    while let Ok(n) = stream.read(&mut buf) {
        if n == 0 {
            break;
        }
        response.extend_from_slice(&buf[..n]);
        if response.len() > 65536 {
            break;
        }
    }

    let text = String::from_utf8_lossy(&response);
    if let Some((_, body)) = text.split_once("\r\n\r\n") {
        return Some(body.to_string());
    }
    Some(text.to_string())
}

fn count_docker_containers() -> Option<usize> {
    let uid = unsafe { libc::getuid() };
    let sockets = [
        Path::new("/var/run/docker.sock"),
        Path::new("/run/docker.sock"),
        &std::path::PathBuf::from(format!("/run/user/{uid}/docker.sock")),
    ];

    for sock in sockets {
        if let Some(body) = query_socket_json(
            sock,
            "GET /containers/json HTTP/1.0\r\nHost: localhost\r\n\r\n",
        ) && let Ok(val) = crate::json::parse(&body)
            && let Some(arr) = val.as_array()
        {
            return Some(arr.len());
        }
    }
    None
}

fn count_podman_containers() -> Option<usize> {
    let uid = unsafe { libc::getuid() };
    let user_sock = std::path::PathBuf::from(format!("/run/user/{uid}/podman/podman.sock"));
    let sockets = [user_sock.as_path(), Path::new("/run/podman/podman.sock")];

    for sock in sockets {
        if let Some(body) = query_socket_json(
            sock,
            "GET /v4.0.0/libpod/containers/json HTTP/1.0\r\nHost: localhost\r\n\r\n",
        ) && let Ok(val) = crate::json::parse(&body)
            && let Some(arr) = val.as_array()
        {
            return Some(arr.len());
        }
    }
    None
}

fn count_cgroup_containers() -> Option<usize> {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir("/sys/fs/cgroup/system.slice") {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            // Both Docker and Podman name their scopes `<engine>-<id>.scope`.
            let is_container = (name.starts_with("docker-") || name.starts_with("libpod-"))
                && name.ends_with(".scope");
            if is_container {
                count += 1;
            }
        }
    }
    if count > 0 { Some(count) } else { None }
}
