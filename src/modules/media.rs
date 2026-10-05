use std::sync::OnceLock;

use crate::config;
use crate::module::{Module, ModuleOutput};
use crate::modules::mpris;
use crate::template;

pub struct Media;

impl Module for Media {
    fn name(&self) -> &'static str {
        "Media"
    }
    fn id(&self) -> &'static str {
        "media"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let info = get_mpris_info();
        let (title, artist) = info.media.as_ref()?;

        let tmpl = config::get()
            .format("media")
            .unwrap_or(if !artist.is_empty() {
                "{title} - {artist}"
            } else {
                "{title}"
            });

        let mut ctx = template::Context::new("media");
        ctx.set_str("title", title.clone());
        ctx.set_str("artist", artist.clone());

        let formatted = template::render_template(tmpl, &ctx);
        Some(ModuleOutput::new("Media", formatted))
    }
}

#[derive(Debug, Clone, Default)]
pub struct MprisInfo {
    pub player: Option<String>,
    pub media: Option<(String, String)>,
}

static MPRIS_CACHE: OnceLock<MprisInfo> = OnceLock::new();

pub fn get_mpris_info() -> &'static MprisInfo {
    MPRIS_CACHE.get_or_init(query_session)
}

/// Finds the player on the bus directly, shelling out only for `busctl get-property`.
fn query_session() -> MprisInfo {
    let timeout = std::time::Duration::from_millis(50);
    let Some(service) = mpris::players(timeout).into_iter().next() else {
        // No bus means no players, so the process scan below would find nothing.
        if std::env::var("DBUS_SESSION_BUS_ADDRESS").is_err() {
            let _ = has_mpris_process();
        }
        return MprisInfo::default();
    };

    let player = friendly_player_name(&service);
    let media = read_metadata(&service);

    MprisInfo {
        player: Some(player),
        media,
    }
}

/// Pulls `xesam:title` and `xesam:artist` out of the player's metadata.
fn read_metadata(service: &str) -> Option<(String, String)> {
    let out = std::process::Command::new("busctl")
        .args([
            "--user",
            "get-property",
            service,
            "/org/mpris/MediaPlayer2",
            MPRIS_PLAYER_IFACE,
            "Metadata",
        ])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let title = dbus_variant_string(&text, "xesam:title").filter(|t| !t.is_empty())?;
    let artist = dbus_variant_string(&text, "xesam:artist").unwrap_or_default();
    Some((title, artist))
}

const MPRIS_PLAYER_IFACE: &str = "org.mpris.MediaPlayer2.Player";

/// Turns `org.mpris.MediaPlayer2.firefox.instance_1_9` into `Firefox`.
fn friendly_player_name(service: &str) -> String {
    let raw = service
        .strip_prefix(mpris::MPRIS_PREFIX)
        .unwrap_or(service)
        .split('.')
        .next()
        .unwrap_or(service);
    let mut chars = raw.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Pulls one string out of a `busctl get-property` dump, whose values follow a type signature.
fn dbus_variant_string(dump: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let after = dump.split_once(&needle)?.1;
    let rest = after.trim_start();

    // Skip the type signature and, for container types like `as 1`, its element count.
    let mut chars = rest.char_indices().peekable();
    let mut sig_end = 0;
    while let Some((i, c)) = chars.next() {
        if c.is_ascii_alphabetic() {
            sig_end = i + 1;
        } else if c.is_ascii_digit() || c == ' ' {
            if sig_end == 0 {
                return None;
            }
            // A digit directly after the signature belongs to an array length.
            if let Some(&(next_i, next_c)) = chars.peek()
                && next_c == ' '
            {
                sig_end = next_i + 1;
            }
        } else if c == '"' {
            break;
        } else {
            return None;
        }
    }

    let value_part = rest.get(sig_end..)?.trim_start();
    let quoted = value_part.strip_prefix('"')?;
    let end = quoted.find('"')?;
    Some(unescape_dbus_string(&quoted[..end]))
}

/// Decodes GVariant escapes: three-digit octal for non-ASCII bytes, plus `\n`, `\t`, `\\`, `\"`.
fn unescape_dbus_string(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'\\' || i + 1 >= bytes.len() {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        match bytes[i + 1] {
            b'n' => {
                out.push(b'\n');
                i += 2;
            }
            b'r' => {
                out.push(b'\r');
                i += 2;
            }
            b't' => {
                out.push(b'\t');
                i += 2;
            }
            // Octal escapes come first: `\0` is the NUL byte, `\012` a newline.
            b'0'..=b'7' => {
                // Three octal digits, truncated to a byte as the format allows.
                let digits = &bytes[i + 1..bytes.len().min(i + 4)];
                let octal: String = digits
                    .iter()
                    .take_while(|b| (b'0'..=b'7').contains(b))
                    .map(|b| *b as char)
                    .collect();
                match u32::from_str_radix(&octal, 8) {
                    Ok(v) => {
                        out.push(v as u8);
                        i += 1 + octal.len();
                    }
                    Err(_) => {
                        out.push(bytes[i + 1]);
                        i += 2;
                    }
                }
            }
            other => {
                out.push(other);
                i += 2;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn has_mpris_process() -> bool {
    const MPRIS_TARGETS: &[&[u8]] = &[
        b"spotify",
        b"vlc",
        b"mpv",
        b"rhythmbox",
        b"clementine",
        b"audacious",
        b"deadbeef",
        b"strawberry",
        b"celluloid",
        b"elisa",
        b"totem",
        b"pragha",
        b"quodlibet",
        b"cmus",
        b"mocp",
        b"ncmpcpp",
        b"mpd",
        b"amberol",
        b"lollypop",
        b"sayonara",
        b"gmusicbrowser",
        b"plasma-browser",
        b"tidal-hifi",
        b"apple-music",
        b"youtube-music",
    ];

    let fd = unsafe { libc::open(c"/proc".as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY) };
    if fd < 0 {
        return false;
    }

    let mut dents_buf = [0u8; 8192];
    let mut comm_buf = [0u8; 32];
    let mut path_buf = [0u8; 48];
    path_buf[..6].copy_from_slice(b"/proc/");

    let mut found = false;
    loop {
        let nread = unsafe {
            libc::syscall(
                libc::SYS_getdents64,
                fd,
                dents_buf.as_mut_ptr(),
                dents_buf.len(),
            )
        };
        if nread <= 0 {
            break;
        }
        let mut pos = 0usize;
        let total = nread as usize;
        while pos + 19 <= total {
            let d_reclen = (dents_buf[pos + 16] as usize) | ((dents_buf[pos + 17] as usize) << 8);
            if d_reclen == 0 || pos + d_reclen > total {
                break;
            }
            let name_start = pos + 19;
            let mut name_end = name_start;
            while name_end < pos + d_reclen && dents_buf[name_end] != 0 {
                name_end += 1;
            }
            let name_bytes = &dents_buf[name_start..name_end];
            if !name_bytes.is_empty()
                && name_bytes.iter().all(|b| b.is_ascii_digit())
                && let Ok(pid) = std::str::from_utf8(name_bytes).unwrap_or("").parse::<u32>()
                && pid >= 1000
            {
                let p_len = 6 + name_bytes.len();
                path_buf[6..p_len].copy_from_slice(name_bytes);
                path_buf[p_len..p_len + 6].copy_from_slice(b"/comm\0");
                let pfd =
                    unsafe { libc::open(path_buf.as_ptr() as *const libc::c_char, libc::O_RDONLY) };
                if pfd >= 0 {
                    let n = unsafe {
                        libc::read(
                            pfd,
                            comm_buf.as_mut_ptr() as *mut libc::c_void,
                            comm_buf.len(),
                        )
                    };
                    unsafe { libc::close(pfd) };
                    if n > 0 {
                        let comm = &comm_buf[..n as usize];
                        for &target in MPRIS_TARGETS {
                            if comm
                                .windows(target.len())
                                .any(|w| w.eq_ignore_ascii_case(target))
                            {
                                found = true;
                                break;
                            }
                        }
                        if found {
                            break;
                        }
                    }
                }
            }
            pos += d_reclen;
        }
        if found {
            break;
        }
    }
    unsafe { libc::close(fd) };
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trimmed version of what `busctl get-property ... Metadata` returns.
    const DUMP: &str = r#"a{sv} 3 "mpris:trackid" o "/org/mpris/MediaPlayer2/firefox" "xesam:title" s "Song Title" "xesam:artist" as 1 "The Artist""#;

    #[test]
    fn parses_plain_string_variant() {
        assert_eq!(
            dbus_variant_string(DUMP, "xesam:title").as_deref(),
            Some("Song Title")
        );
    }

    #[test]
    fn parses_string_array_variant() {
        assert_eq!(
            dbus_variant_string(DUMP, "xesam:artist").as_deref(),
            Some("The Artist")
        );
    }

    #[test]
    fn skips_unrelated_keys() {
        assert_eq!(
            dbus_variant_string(DUMP, "mpris:trackid").as_deref(),
            Some("/org/mpris/MediaPlayer2/firefox")
        );
    }

    #[test]
    fn decodes_utf8_escapes() {
        let dump = r#"a{sv} 1 "xesam:title" s "\320\255\321\202\320\260""#;
        assert_eq!(
            dbus_variant_string(dump, "xesam:title").as_deref(),
            Some("Эта")
        );
    }

    #[test]
    fn decodes_newline_escape() {
        let dump = "a{sv} 1 \"xesam:title\" s \"a\\nb\"";
        assert_eq!(
            dbus_variant_string(dump, "xesam:title").as_deref(),
            Some("a\nb")
        );
    }

    #[test]
    fn missing_key_and_empty_value_are_distinguished() {
        assert_eq!(dbus_variant_string(DUMP, "xesam:album"), None);
        let empty = r#"a{sv} 1 "xesam:album" s """#;
        assert_eq!(
            dbus_variant_string(empty, "xesam:album").as_deref(),
            Some("")
        );
    }

    #[test]
    fn last_key_on_the_line_is_found() {
        let dump = r#"a{sv} 1 "xesam:url" s "https://example.com""#;
        assert_eq!(
            dbus_variant_string(dump, "xesam:url").as_deref(),
            Some("https://example.com")
        );
    }
}
