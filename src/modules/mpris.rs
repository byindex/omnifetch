//! Minimal D-Bus client for MPRIS discovery; track metadata is read with `busctl`.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

const SCRATCH: usize = 64 * 1024;

pub const MPRIS_PREFIX: &str = "org.mpris.MediaPlayer2.";

/// Hello to org.freedesktop.DBus.
const HELLO_MSG: &[u8] = &[
    108, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 109, 0, 0, 0, 1, 1, 111, 0, 21, 0, 0, 0, 47, 111, 114,
    103, 47, 102, 114, 101, 101, 100, 101, 115, 107, 116, 111, 112, 47, 68, 66, 117, 115, 0, 0, 0,
    2, 1, 115, 0, 20, 0, 0, 0, 111, 114, 103, 46, 102, 114, 101, 101, 100, 101, 115, 107, 116, 111,
    112, 46, 68, 66, 117, 115, 0, 0, 0, 0, 3, 1, 115, 0, 5, 0, 0, 0, 72, 101, 108, 108, 111, 0, 0,
    0, 6, 1, 115, 0, 20, 0, 0, 0, 111, 114, 103, 46, 102, 114, 101, 101, 100, 101, 115, 107, 116,
    111, 112, 46, 68, 66, 117, 115, 0, 0, 0, 0,
];

/// ListNames on org.freedesktop.DBus.
const LIST_NAMES_MSG: &[u8] = &[
    108, 1, 0, 1, 0, 0, 0, 0, 2, 0, 0, 0, 117, 0, 0, 0, 1, 1, 111, 0, 21, 0, 0, 0, 47, 111, 114,
    103, 47, 102, 114, 101, 101, 100, 101, 115, 107, 116, 111, 112, 47, 68, 66, 117, 115, 0, 0, 0,
    2, 1, 115, 0, 20, 0, 0, 0, 111, 114, 103, 46, 102, 114, 101, 101, 100, 101, 115, 107, 116, 111,
    112, 46, 68, 66, 117, 115, 0, 0, 0, 0, 3, 1, 115, 0, 9, 0, 0, 0, 76, 105, 115, 116, 78, 97,
    109, 101, 115, 0, 0, 0, 0, 0, 0, 0, 6, 1, 115, 0, 20, 0, 0, 0, 111, 114, 103, 46, 102, 114,
    101, 101, 100, 101, 115, 107, 116, 111, 112, 46, 68, 66, 117, 115, 0, 0, 0, 0,
];

/// Which D-Bus instance to talk to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bus {
    /// The per-user session bus, where desktop apps live.
    Session,
    /// The machine-wide system bus, where daemons like BlueZ register.
    System,
}

fn connect(bus: Bus, timeout: Duration) -> Option<UnixStream> {
    let uid = unsafe { libc::getuid() };
    let path = match bus {
        Bus::Session => std::env::var("XDG_RUNTIME_DIR")
            .map(|p| format!("{p}/bus"))
            .unwrap_or_else(|_| format!("/run/user/{uid}/bus")),
        Bus::System => std::env::var("DBUS_SYSTEM_BUS_ADDRESS")
            .map(|a| {
                // The variable can hold several addresses. Take the unix one.
                a.split(';')
                    .find_map(|e| e.strip_prefix("unix:path=").map(str::to_string))
                    .map(|p| p.split(',').next().unwrap_or_default().to_string())
                    .unwrap_or_default()
            })
            .ok()
            .filter(|p| !p.is_empty())
            .or_else(|| Some("/run/dbus/system_bus_socket".to_string()))?,
    };
    let stream = UnixStream::connect(path).ok()?;
    stream.set_read_timeout(Some(timeout)).ok()?;
    stream.set_write_timeout(Some(timeout)).ok()?;
    Some(stream)
}

fn read_exact(stream: &mut UnixStream, mut buf: &mut [u8]) -> Option<()> {
    while !buf.is_empty() {
        match stream.read(buf) {
            Ok(0) | Err(_) => return None,
            Ok(n) => buf = &mut buf[n..],
        }
    }
    Some(())
}

/// Reads one complete message and returns its body.
fn read_message(stream: &mut UnixStream, scratch: &mut [u8]) -> Option<Vec<u8>> {
    const HEADER: usize = 16;
    if scratch.len() < HEADER {
        return None;
    }
    let (header, rest) = scratch.split_at_mut(HEADER);
    read_exact(stream, header)?;

    let body_len = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
    let fields_len = u32::from_le_bytes([header[12], header[13], header[14], header[15]]) as usize;
    let padded = fields_len.div_ceil(8) * 8;
    if padded + body_len > rest.len() {
        return None;
    }
    let (fields, body) = rest.split_at_mut(padded);
    read_exact(stream, fields)?;
    read_exact(stream, &mut body[..body_len])?;
    Some(body[..body_len].to_vec())
}

/// Walks the marshalled `as` body of `ListNames`: u32 length, then u32-prefixed strings padded to 4.
fn reply_strings(body: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    if body.len() < 4 {
        return out;
    }
    let array_len = u32::from_le_bytes([body[0], body[1], body[2], body[3]]) as usize;
    let end = (4 + array_len).min(body.len());
    let mut i = 4;

    while i + 4 <= end {
        // Padding aligns each element to 4 bytes from where that element starts.
        let start = i;
        let len = u32::from_le_bytes([body[i], body[i + 1], body[i + 2], body[i + 3]]) as usize;
        i += 4;
        if len > end - i {
            break;
        }
        out.push(String::from_utf8_lossy(&body[i..i + len]).into_owned());
        i += len + 1; // +1 for the NUL terminator
        let consumed = i - start;
        if consumed % 4 != 0 {
            i += 4 - consumed % 4;
        }
    }
    out
}

/// Bus names of currently registered MPRIS players.
pub fn players(timeout: Duration) -> Vec<String> {
    bus_names(Bus::Session, timeout)
        .into_iter()
        .filter(|n| n.starts_with(MPRIS_PREFIX))
        .collect()
}

/// Names on the session bus, quick enough to beat forking out to `busctl`.
pub fn bus_names(bus: Bus, timeout: Duration) -> Vec<String> {
    let Some(mut stream) = connect(bus, timeout) else {
        return Vec::new();
    };

    let uid = unsafe { libc::getuid() };
    let hex: String = uid
        .to_string()
        .bytes()
        .map(|b| format!("{b:02x}"))
        .collect();
    if stream
        .write_all(format!("\0AUTH EXTERNAL {hex}\r\nNEGOTIATE_UNIX_FD\r\nBEGIN\r\n").as_bytes())
        .is_err()
    {
        return Vec::new();
    }

    let mut buf = [0u8; 256];
    let n = match stream.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return Vec::new(),
    };
    if !buf[..n].starts_with(b"OK ") {
        return Vec::new();
    }

    let mut scratch = vec![0u8; SCRATCH];
    if stream.write_all(HELLO_MSG).is_err() {
        return Vec::new();
    }
    // The reply, then the unsolicited NameAcquired.
    read_message(&mut stream, &mut scratch);
    read_message(&mut stream, &mut scratch);

    if stream.write_all(LIST_NAMES_MSG).is_err() {
        return Vec::new();
    }
    let Some(body) = read_message(&mut stream, &mut scratch) else {
        return Vec::new();
    };
    reply_strings(&body)
}

/// True when a bus name with this prefix is registered.
pub fn has_service(bus: Bus, prefix: &str, timeout: Duration) -> bool {
    bus_names(bus, timeout)
        .iter()
        .any(|n| n.starts_with(prefix))
}

#[cfg(test)]
mod extra_tests {
    use super::*;

    #[test]
    fn has_service_never_panics() {
        let _ = has_service(Bus::System, "org.bluez", Duration::from_millis(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn le32(n: u32) -> [u8; 4] {
        n.to_le_bytes()
    }

    /// Builds a `ListNames` reply body: array length, then padded strings.
    fn list_names_body(names: &[&str], declared: Option<u32>) -> Vec<u8> {
        let mut content = Vec::new();
        for s in names {
            content.extend_from_slice(&le32(s.len() as u32));
            content.extend_from_slice(s.as_bytes());
            content.push(0);
            while content.len() % 4 != 0 {
                content.push(0);
            }
        }
        let mut body = le32(declared.unwrap_or(content.len() as u32)).to_vec();
        body.extend_from_slice(&content);
        body
    }

    #[test]
    fn list_names_reply_is_walked() {
        let body = list_names_body(
            &[":1.5", "org.mpris.MediaPlayer2.firefox.instance_1_9"],
            None,
        );
        let names = reply_strings(&body);
        assert_eq!(names[0], ":1.5");
        assert!(
            names
                .iter()
                .any(|n| n == "org.mpris.MediaPlayer2.firefox.instance_1_9"),
            "got {names:?}"
        );
    }

    #[test]
    fn declared_length_bounds_the_scan() {
        // A length larger than the payload must stop early, not read past it.
        let mut body = list_names_body(&["a", "b"], None);
        body[0..4].copy_from_slice(&le32(4096));
        assert_eq!(reply_strings(&body), vec!["a", "b"]);
    }

    #[test]
    fn short_length_is_respected() {
        // Only the first string is inside the declared array.
        let body = list_names_body(&[":1.0", ":1.1", ":1.2"], Some(12));
        let names = reply_strings(&body);
        assert!(names.len() <= 3);
        assert!(names.contains(&":1.0".to_string()));
    }

    #[test]
    fn truncated_replies_are_rejected_not_panicking() {
        assert!(reply_strings(&[]).is_empty());
        assert!(reply_strings(&[1, 2, 3]).is_empty());
        // Declares a string longer than what remains.
        let mut body = le32(64).to_vec();
        body.extend_from_slice(&le32(250));
        body.extend_from_slice(b"ab");
        assert!(reply_strings(&body).is_empty());
    }

    #[test]
    fn message_constants_have_plausible_headers() {
        for msg in [HELLO_MSG, LIST_NAMES_MSG] {
            assert_eq!(msg[0], b'l', "little endian");
            assert_eq!(msg[1], 1, "METHOD_CALL");
            assert_eq!(msg[3], 1, "protocol version");
            let fields = u32::from_le_bytes([msg[12], msg[13], msg[14], msg[15]]) as usize;
            // The declared length includes the preamble, so it covers header plus padded array.
            let padded = fields.div_ceil(8) * 8;
            assert_eq!(msg.len(), 16 + padded, "no body for a void call");
        }
    }

    #[test]
    fn discovery_never_panics_without_a_bus() {
        let _ = players(Duration::from_millis(50));
    }
}
