//! SVG and HTML export rendering for terminal output.

/// Escapes XML / HTML special characters.
fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StyleState {
    color: Option<String>,
    bold: bool,
}

impl StyleState {
    fn new() -> Self {
        Self {
            color: None,
            bold: false,
        }
    }

    fn reset(&mut self) {
        self.color = None;
        self.bold = false;
    }
}

/// Maps standard 8/16 ANSI codes to hex colors.
fn ansi_color_to_hex(code: u32) -> Option<&'static str> {
    match code {
        30 => Some("#45475a"),
        31 => Some("#f38ba8"),
        32 => Some("#a6e3a1"),
        33 => Some("#f9e2af"),
        34 => Some("#89b4fa"),
        35 => Some("#f5c2e7"),
        36 => Some("#94e2d5"),
        37 => Some("#cdd6f4"),
        90 => Some("#585b70"),
        91 => Some("#eba0ac"),
        92 => Some("#b4befe"),
        93 => Some("#fab387"),
        94 => Some("#74c7ec"),
        95 => Some("#cba6f7"),
        96 => Some("#89dceb"),
        97 => Some("#ffffff"),
        _ => None,
    }
}

fn parse_line_spans(line: &str, state: &mut StyleState) -> Vec<(StyleState, String)> {
    let mut spans = Vec::new();
    let mut cur_text = String::new();
    let bytes = line.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            // Flush current text under active style
            if !cur_text.is_empty() {
                spans.push((state.clone(), cur_text.clone()));
                cur_text.clear();
            }

            i += 2;
            let start = i;
            while i < bytes.len() && bytes[i] != b'm' && !bytes[i].is_ascii_alphabetic() {
                i += 1;
            }

            if i < bytes.len() && bytes[i] == b'm' {
                let seq = std::str::from_utf8(&bytes[start..i]).unwrap_or_default();
                let parts: Vec<&str> = seq.split(';').collect();
                if seq.is_empty() || seq == "0" {
                    state.reset();
                } else {
                    let mut p_idx = 0;
                    while p_idx < parts.len() {
                        let num: u32 = parts[p_idx].parse().unwrap_or(0);
                        match num {
                            0 => state.reset(),
                            1 => state.bold = true,
                            22 => state.bold = false,
                            39 => state.color = None,
                            38 => {
                                // 38;2;r;g;b (TrueColor) or 38;5;n
                                if p_idx + 4 < parts.len() && parts[p_idx + 1] == "2" {
                                    let r: u8 = parts[p_idx + 2].parse().unwrap_or(0);
                                    let g: u8 = parts[p_idx + 3].parse().unwrap_or(0);
                                    let b: u8 = parts[p_idx + 4].parse().unwrap_or(0);
                                    state.color = Some(format!("#{r:02x}{g:02x}{b:02x}"));
                                    p_idx += 4;
                                } else if p_idx + 2 < parts.len() && parts[p_idx + 1] == "5" {
                                    let n: u8 = parts[p_idx + 2].parse().unwrap_or(0);
                                    state.color = Some(format!("#{n:02x}{n:02x}{n:02x}"));
                                    p_idx += 2;
                                }
                            }
                            c => {
                                if let Some(hex) = ansi_color_to_hex(c) {
                                    state.color = Some(hex.to_string());
                                }
                            }
                        }
                        p_idx += 1;
                    }
                }
                i += 1; // skip 'm'
            }
        } else {
            cur_text.push(line[i..].chars().next().unwrap_or(' '));
            i += line[i..].chars().next().map_or(1, |c| c.len_utf8());
        }
    }

    if !cur_text.is_empty() {
        spans.push((state.clone(), cur_text));
    }

    spans
}

/// Converts ANSI terminal output to an SVG image.
pub fn to_svg(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let line_height = 20.0;
    let char_width = 8.5;
    let padding_x = 24.0;
    let padding_top = 48.0;
    let padding_bottom = 24.0;

    let max_cols = lines
        .iter()
        .map(|l| crate::render::visible_width(l))
        .max()
        .unwrap_or(60);

    let width = ((max_cols as f64 * char_width) + (padding_x * 2.0))
        .ceil()
        .max(280.0) as usize;
    let title_x = width as f64 / 2.0;
    let height =
        ((lines.len() as f64 * line_height) + padding_top + padding_bottom).ceil() as usize;

    let mut svg = String::with_capacity(text.len() * 3 + 1024);
    svg.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">
  <defs>
    <style>
      .terminal {{
        font-family: 'JetBrains Mono', 'Fira Code', 'DejaVu Sans Mono', 'Courier New', monospace;
        font-size: 13.5px;
        white-space: pre;
      }}
    </style>
  </defs>
  <!-- Window Background -->
  <rect width="{width}" height="{height}" rx="10" ry="10" fill="#181825"/>
  <rect width="{width}" height="32" rx="10" ry="10" fill="#1e1e2e"/>
  <rect y="22" width="{width}" height="10" fill="#1e1e2e"/>
  <!-- Window Controls -->
  <circle cx="20" cy="16" r="6" fill="#f38ba8"/>
  <circle cx="38" cy="16" r="6" fill="#f9e2af"/>
  <circle cx="56" cy="16" r="6" fill="#a6e3a1"/>
  <text x="{title_x:.1}" y="20" text-anchor="middle" fill="#6c7086" font-family="'JetBrains Mono', monospace" font-size="12">omnifetch</text>
  <!-- Content -->
  <g class="terminal">
"##
    ));

    let mut state = StyleState::new();
    for (idx, line) in lines.iter().enumerate() {
        let y = padding_top + (idx as f64 * line_height);
        let spans = parse_line_spans(line, &mut state);

        svg.push_str(&format!(
            r#"    <text x="{padding_x}" y="{y:.1}" xml:space="preserve">"#
        ));
        for (st, s) in spans {
            let escaped = escape_xml(&s);
            let fill = st.color.as_deref().unwrap_or("#cdd6f4");
            if st.bold {
                svg.push_str(&format!(
                    r#"<tspan fill="{fill}" font-weight="bold">{escaped}</tspan>"#
                ));
            } else {
                svg.push_str(&format!(r#"<tspan fill="{fill}">{escaped}</tspan>"#));
            }
        }
        svg.push_str("</text>\n");
    }

    svg.push_str("  </g>\n</svg>\n");
    svg
}

/// Converts ANSI terminal output to a standalone HTML page.
pub fn to_html(text: &str) -> String {
    let mut html = String::with_capacity(text.len() * 3 + 1024);
    html.push_str(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>omnifetch</title>
  <style>
    body {
      background-color: #181825;
      color: #cdd6f4;
      font-family: 'JetBrains Mono', 'Fira Code', 'DejaVu Sans Mono', 'Courier New', monospace;
      padding: 24px;
      margin: 0;
    }
    .window {
      background-color: #1e1e2e;
      border-radius: 10px;
      padding: 20px 24px;
      display: inline-block;
      box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    }
    pre {
      margin: 0;
      line-height: 1.35;
      font-size: 14px;
      white-space: pre;
    }
  </style>
</head>
<body>
  <div class="window">
    <pre><code>"#,
    );

    let mut state = StyleState::new();
    for line in text.lines() {
        let spans = parse_line_spans(line, &mut state);
        for (st, s) in spans {
            let escaped = escape_xml(&s);
            let color_attr = st
                .color
                .as_deref()
                .map(|c| format!("color: {c};"))
                .unwrap_or_default();
            let weight_attr = if st.bold { "font-weight: bold;" } else { "" };
            if !color_attr.is_empty() || !weight_attr.is_empty() {
                html.push_str(&format!(
                    r#"<span style="{color_attr}{weight_attr}">{escaped}</span>"#
                ));
            } else {
                html.push_str(&escaped);
            }
        }
        html.push('\n');
    }

    html.push_str(
        r#"</code></pre>
  </div>
</body>
</html>
"#,
    );
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_svg_contains_content() {
        let text = "\x1b[31mHello\x1b[0m \x1b[32mWorld\x1b[0m";
        let svg = to_svg(text);
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("Hello"));
        assert!(svg.contains("World"));
        assert!(svg.contains("#f38ba8")); // red
        assert!(svg.contains("#a6e3a1")); // green
        assert!(svg.ends_with("</svg>\n"));
    }

    #[test]
    fn test_to_html_contains_content() {
        let text = "\x1b[1;34mBold Blue\x1b[0m";
        let html = to_html(text);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Bold Blue"));
        assert!(html.contains("#89b4fa")); // blue
        assert!(html.contains("font-weight: bold;"));
    }
}
