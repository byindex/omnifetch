use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ImageRender {
    pub escape: String,
    pub cols: usize,
    pub rows: usize,
}

pub fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        out.push(CHARSET[((n >> 18) & 63) as usize] as char);
        out.push(CHARSET[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARSET[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARSET[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Generates terminal escape sequences for displaying an image alongside text.
pub fn load_image(path: &Path, cols: usize, rows: usize) -> Option<ImageRender> {
    let ext_is_jpeg = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg"))
        .unwrap_or(false);

    if ext_is_jpeg {
        eprintln!(
            "omnifetch: warning: JPEG format is not supported by terminal graphics protocols (PNG required); falling back to distro logo"
        );
        return None;
    }

    let bytes = match fs::read(path) {
        Ok(b) if !b.is_empty() => b,
        Ok(_) => return None,
        Err(e) => {
            eprintln!(
                "omnifetch: warning: failed to read image '{}': {e}; falling back to distro logo",
                path.display()
            );
            return None;
        }
    };

    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        eprintln!(
            "omnifetch: warning: JPEG format is not supported by terminal graphics protocols (PNG required); falling back to distro logo"
        );
        return None;
    }

    // Sixel format
    let is_sixel = bytes.starts_with(b"\x1bPq")
        || path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("six") || e.eq_ignore_ascii_case("sixel"))
            .unwrap_or(false);

    if is_sixel {
        let sixel_str = String::from_utf8_lossy(&bytes).to_string();
        return Some(ImageRender {
            escape: sixel_str,
            cols,
            rows,
        });
    }

    // Kitty Graphics Protocol
    let b64 = base64_encode(&bytes);
    let mut escape = String::new();
    let chunk_size = 4096;
    let chunks: Vec<&str> = b64
        .as_bytes()
        .chunks(chunk_size)
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect();

    if chunks.is_empty() {
        return None;
    }

    for (idx, chunk) in chunks.iter().enumerate() {
        let is_last = idx == chunks.len() - 1;
        let m = if is_last { 0 } else { 1 };
        if idx == 0 {
            // Header: action=T, format=auto, c=cols, r=rows, C=1
            escape.push_str(&format!(
                "\x1b_Ga=T,f=100,t=d,c={cols},r={rows},C=1,m={m};{chunk}\x1b\\"
            ));
        } else {
            escape.push_str(&format!("\x1b_Gm={m};{chunk}\x1b\\"));
        }
    }

    Some(ImageRender { escape, cols, rows })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_load_image_rejects_jpeg() {
        assert!(load_image(Path::new("test.jpg"), 34, 17).is_none());
        assert!(load_image(Path::new("test.jpeg"), 34, 17).is_none());
    }
}
