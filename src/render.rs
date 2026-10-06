use crate::ModuleOutput;
use crate::config::{self, Config};
use crate::style::{Painter, Style};
use crate::sys;

use crate::json::JsonValue;

/// Builds the JSON payload, matching fastfetch's `[{name, data}]` shape.
pub fn to_json(results: &[(&'static str, Option<ModuleOutput>)]) -> String {
    let items: Vec<JsonValue> = results
        .iter()
        .filter_map(|(id, out)| {
            let out = out.as_ref()?;
            if matches!(*id, "separator" | "break" | "colors") {
                return None;
            }
            let data = json_data(out);
            let entries = vec![
                ("name".to_string(), JsonValue::String(out.name.clone())),
                ("data".to_string(), data),
            ];
            Some(JsonValue::Object(entries))
        })
        .collect();
    JsonValue::Array(items).to_string_pretty()
}

pub fn strip_ansi(s: &str) -> String {
    if !s.contains('\x1b') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c.is_ascii_alphabetic() || c == '\\' {
                in_escape = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn json_data(out: &ModuleOutput) -> JsonValue {
    let labeled: Vec<_> = out.fields.iter().filter(|f| !f.label.is_empty()).collect();

    if !labeled.is_empty() && labeled.len() == out.fields.len() {
        let mut entries = Vec::with_capacity(out.fields.len());
        for f in &out.fields {
            entries.push((f.label.clone(), JsonValue::String(strip_ansi(&f.value))));
        }
        return JsonValue::Object(entries);
    }
    if out.fields.len() == 1 {
        return JsonValue::String(strip_ansi(&out.fields[0].value));
    }
    JsonValue::Array(
        out.fields
            .iter()
            .map(|f| JsonValue::String(strip_ansi(&f.value)))
            .collect(),
    )
}

/// A single printable row: the key part and the value part are styled apart.
#[derive(Clone)]
struct Line {
    key: Option<String>,
    value: String,
    category: Option<u8>,
    is_divider: bool,
}

pub fn to_text(
    results: &[(&'static str, Option<ModuleOutput>)],
    logo_text: Option<&str>,
    image_render: Option<&crate::image::ImageRender>,
    cfg: &Config,
    style: &Style,
) -> String {
    let p = Painter::new(cfg.color);
    let mut lines: Vec<Line> = Vec::new();

    // Separator length comes from the title, so it waits for the title line.
    let mut separator: Option<String> = None;
    let auto_category_breaks = cfg.preset.as_deref() != Some("compact");
    let mut last_category: Option<u8> = None;

    for (id, out) in results {
        let Some(out) = out else {
            if *id == "separator" {
                // placeholder, filled in below
            }
            continue;
        };
        if *id == "separator" {
            continue;
        }
        if *id == "title" {
            let v = out.fields[0].value.clone();
            separator = Some("-".repeat(v.chars().count().max(3)));
            lines.push(Line {
                key: None,
                value: p.paint(&v, style.title, style.bold_title),
                category: Some(0),
                is_divider: false,
            });
            last_category = Some(0);
            continue;
        }
        if *id == "break" {
            if !lines.is_empty() && !lines.last().map(|l| l.is_divider).unwrap_or(false) {
                lines.push(Line {
                    key: None,
                    value: String::new(),
                    category: None,
                    is_divider: true,
                });
            }
            continue;
        }

        if out.fields.is_empty() {
            continue;
        }

        let cat = module_category(id);
        if auto_category_breaks {
            if let Some(prev) = last_category
                && prev != cat
                && !lines.is_empty()
                && !lines.last().map(|l| l.is_divider).unwrap_or(false)
            {
                lines.push(Line {
                    key: None,
                    value: String::new(),
                    category: None,
                    is_divider: true,
                });
            }
            last_category = Some(cat);
        }

        let key_name = cfg.key_label(&out.name);
        let icon = if cfg.nerd {
            Some(module_nerd_icon(&out.name))
        } else {
            None
        };

        for (i, f) in out.fields.iter().enumerate() {
            let key_str = match (icon, i == 0) {
                (Some(ic), true) if cfg.nerd_icons_only => format!("{ic} "),
                (Some(ic), true) => format!("{ic} {key_name}:"),
                (Some(_), false) if f.label.is_empty() => " ".repeat(if cfg.nerd_icons_only {
                    3
                } else {
                    key_name.chars().count() + 4
                }),
                (Some(ic), false) => format!("{ic} {key_name}/{}:", f.label),
                (None, true) => format!("{key_name}:"),
                (None, false) if f.label.is_empty() => " ".repeat(key_name.chars().count() + 1),
                (None, false) => format!("{key_name}/{}:", f.label),
            };

            lines.push(Line {
                key: Some(p.paint(&key_str, style.key, style.bold_key)),
                value: p.paint(&f.value, style.value, false),
                category: Some(cat),
                is_divider: false,
            });
        }
    }

    if let Some(sep) = separator {
        // insert the separator right after the title
        let pos = lines
            .iter()
            .position(|l| l.key.is_none() && !l.value.is_empty() && !l.is_divider)
            .map(|i| i + 1)
            .unwrap_or(0);
        lines.insert(
            pos,
            Line {
                key: None,
                value: p.paint(&sep, style.title, false),
                category: Some(0),
                is_divider: false,
            },
        );
    }

    // Drop leading divider if present
    if lines.first().map(|l| l.is_divider).unwrap_or(false) {
        lines.remove(0);
    }
    // Drop trailing divider if present
    if lines.last().map(|l| l.is_divider).unwrap_or(false) {
        lines.pop();
    }
    // Dedup adjacent dividers
    lines.dedup_by(|a, b| a.is_divider && b.is_divider);

    align_progress_bars(&mut lines);

    let term_cols = sys::terminal_size().map(|ts| ts.cols as usize);
    let logo_w = match logo_text {
        Some(art) if !cfg.logo_top => art
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(visible_width)
            .max()
            .unwrap_or(0),
        _ => 0,
    };
    let border_limit = term_cols.map(|c| {
        if logo_w > 0 {
            c.saturating_sub(logo_w + 4)
        } else {
            c
        }
    });

    let lines = if cfg.border {
        apply_border(&lines, &p, style.key, border_limit, cfg.border_title)
    } else {
        lines
    };

    if let Some(img) = image_render {
        return if cfg.logo_top {
            compose_image_top(img, &lines)
        } else {
            compose_image(img, &lines)
        };
    }

    match logo_text {
        Some(art) => {
            let prepared = if let Some(grad) = cfg.gradient {
                crate::style::apply_gradient_to_text(art, grad)
            } else {
                prepare_logo(art, cfg.color)
            };
            if cfg.logo_top {
                compose_top(&prepared, &lines)
            } else {
                compose(&prepared, &lines)
            }
        }
        None => lines
            .iter()
            .map(|l| match (&l.key, l.value.is_empty()) {
                (Some(k), true) => k.clone(),
                (Some(k), false) => format!("{k} {}", l.value),
                (None, _) => l.value.clone(),
            })
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn prepare_logo(art: &str, color: bool) -> String {
    if !art.contains('$') {
        return art.to_string();
    }
    if !color {
        let mut out = String::with_capacity(art.len());
        let mut chars = art.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '$'
                && let Some(&next) = chars.peek()
                && (next.is_ascii_digit() || next == '_')
            {
                chars.next();
                continue;
            }
            out.push(c);
        }
        return out;
    }

    let mut out = String::with_capacity(art.len() + 128);
    let mut chars = art.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$'
            && let Some(&next) = chars.peek()
        {
            let code = match next {
                '1' => Some("\x1b[36m"),
                '2' => Some("\x1b[37m"),
                '3' => Some("\x1b[32m"),
                '4' => Some("\x1b[33m"),
                '5' => Some("\x1b[34m"),
                '6' => Some("\x1b[35m"),
                '7' => Some("\x1b[31m"),
                '8' => Some("\x1b[90m"),
                '9' => Some("\x1b[97m"),
                '_' => Some("\x1b[0m"),
                _ => None,
            };
            if let Some(ansi) = code {
                chars.next();
                out.push_str(ansi);
                continue;
            }
        }
        out.push(c);
    }
    out.push_str("\x1b[0m");
    out
}

pub fn visible_width(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c.is_ascii_alphabetic() || c == '\\' {
                in_escape = false;
            }
        } else {
            len += 1;
        }
    }
    len
}

fn compose_image(img: &crate::image::ImageRender, lines: &[Line]) -> String {
    let offset = img.cols + 4;
    let rows = img.rows.max(lines.len());
    let mut out = String::new();

    for i in 0..rows {
        let right = lines.get(i).map(render_line).unwrap_or_default();
        if i == 0 {
            out.push_str(&img.escape);
        }
        if !right.is_empty() {
            out.push_str(&format!("\x1b[{}G{}", offset + 1, right));
        }
        out.push('\n');
    }
    out.pop();
    out
}

fn compose(art: &str, lines: &[Line]) -> String {
    let art_lines: Vec<&str> = art.lines().collect();
    let width = art_lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| visible_width(l))
        .max()
        .unwrap_or(0);

    let rows = art_lines.len().max(lines.len());
    let mut out = String::new();
    for i in 0..rows {
        let left = art_lines.get(i).copied().unwrap_or("");
        let right = lines.get(i).map(render_line).unwrap_or_default();
        out.push_str(left);
        if !right.is_empty() {
            let left_w = visible_width(left);
            for _ in left_w..width + 4 {
                out.push(' ');
            }
            out.push_str(&right);
        }
        out.push('\n');
    }
    out.pop();
    out
}

fn render_line(l: &Line) -> String {
    match &l.key {
        Some(k) if l.value.is_empty() => k.clone(),
        Some(k) => format!("{k} {}", l.value),
        None => l.value.clone(),
    }
}

fn compose_top(art: &str, lines: &[Line]) -> String {
    let mut out = String::new();
    let trimmed = art.trim_start_matches('\n').trim_end_matches('\n');
    if !trimmed.is_empty() {
        out.push_str(trimmed);
        out.push('\n');
        out.push('\n');
    }
    for l in lines {
        out.push_str(&render_line(l));
        out.push('\n');
    }
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

fn compose_image_top(img: &crate::image::ImageRender, lines: &[Line]) -> String {
    let mut out = String::new();
    out.push_str(&img.escape);
    out.push('\n');
    for _ in 0..img.rows {
        out.push('\n');
    }
    for l in lines {
        out.push_str(&render_line(l));
        out.push('\n');
    }
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

fn has_unescaped_open_bracket(s: &str) -> bool {
    let bytes = s.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'[' && (i == 0 || bytes[i - 1] != b'\x1b') {
            return true;
        }
    }
    false
}

fn find_progress_bar(s: &str) -> Option<(usize, usize)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\x1b' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            // Skip the CSI introducer, or `\x1b[` reads as a bar bracket.
            i += 2;
            while i < bytes.len() && !bytes[i].is_ascii_alphabetic() {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b'['
            && let Some(end_rel) = s[i + 1..].find(']')
        {
            let abs_end = i + 1 + end_rel + 1;
            let inner = &s[i + 1..abs_end - 1];
            if !has_unescaped_open_bracket(inner) {
                let cfg = config::get();
                let is_bar = inner.contains('\u{2588}')
                    || inner.contains('\u{2591}')
                    || inner.contains('\u{2592}')
                    || inner.contains('\u{2593}')
                    || (!cfg.bar.fill.is_empty() && inner.contains(&cfg.bar.fill))
                    || (!cfg.bar.empty.is_empty() && inner.contains(&cfg.bar.empty));

                if is_bar {
                    return Some((i, abs_end));
                }
            }
        }
        i += 1;
    }
    None
}

fn align_progress_bars(lines: &mut [Line]) {
    let mut i = 0;
    while i < lines.len() {
        if let Some((_, _)) = find_progress_bar(&lines[i].value) {
            let mut group = Vec::new();
            let mut j = i;
            while j < lines.len() {
                if let Some((b_start, _)) = find_progress_bar(&lines[j].value) {
                    let key_str = match &lines[j].key {
                        Some(k) => format!("{k} "),
                        None => String::new(),
                    };
                    let prefix_val = &lines[j].value[..b_start];
                    let prefix_w = visible_width(&key_str) + visible_width(prefix_val);
                    let has_leading_text = !prefix_val.trim().is_empty();
                    group.push((j, b_start, prefix_w, has_leading_text));
                    j += 1;
                } else {
                    break;
                }
            }

            if group.len() > 1 {
                for target_has_leading in [true, false] {
                    let sub_group: Vec<_> = group
                        .iter()
                        .filter(|(_, _, _, has_lead)| *has_lead == target_has_leading)
                        .copied()
                        .collect();
                    if sub_group.len() > 1 {
                        let max_w = sub_group.iter().map(|(_, _, w, _)| *w).max().unwrap_or(0);
                        // Align when distance is within reasonable threshold
                        let all_within_range = sub_group.iter().all(|(_, _, w, _)| max_w - w <= 25);
                        if all_within_range {
                            for (idx, b_start, w, _) in sub_group {
                                let diff = max_w.saturating_sub(w);
                                if diff > 0 {
                                    lines[idx].value.insert_str(b_start, &" ".repeat(diff));
                                }
                            }
                        }
                    }
                }
            }

            i = j;
        } else {
            i += 1;
        }
    }
}

fn apply_border(
    lines: &[Line],
    p: &Painter,
    color: crate::style::Color,
    max_limit: Option<usize>,
    title_align: crate::config::TitleAlign,
) -> Vec<Line> {
    if lines.is_empty() {
        return Vec::new();
    }

    struct Section {
        category: Option<u8>,
        lines: Vec<Line>,
    }

    let mut sections: Vec<Section> = Vec::new();
    let mut current_cat: Option<u8> = None;
    let mut current_lines: Vec<Line> = Vec::new();

    for l in lines {
        if l.is_divider {
            if !current_lines.is_empty() {
                sections.push(Section {
                    category: current_cat,
                    lines: std::mem::take(&mut current_lines),
                });
                current_cat = None;
            }
            continue;
        }

        if l.category != current_cat && !current_lines.is_empty() {
            sections.push(Section {
                category: current_cat,
                lines: std::mem::take(&mut current_lines),
            });
        }

        current_cat = l.category;
        current_lines.push(l.clone());
    }

    if !current_lines.is_empty() {
        sections.push(Section {
            category: current_cat,
            lines: current_lines,
        });
    }

    if sections.is_empty() {
        return Vec::new();
    }

    let mut natural_max_w = 0;
    for s in &sections {
        for l in &s.lines {
            let row = render_line(l);
            natural_max_w = natural_max_w.max(visible_width(&row));
        }
    }

    let max_w = if let Some(limit) = max_limit {
        let content_limit = limit.saturating_sub(4);
        if content_limit >= 30 {
            natural_max_w.min(content_limit)
        } else {
            natural_max_w
        }
    } else {
        natural_max_w
    };

    let border_w = max_w + 2;
    let mut out: Vec<Line> = Vec::new();

    for (idx, sec) in sections.iter().enumerate() {
        if idx > 0 {
            out.push(Line {
                key: None,
                value: String::new(),
                category: None,
                is_divider: false,
            });
        }

        let title_opt = sec.category.and_then(category_title);
        let top = match title_opt {
            Some(title) if border_w >= visible_width(title) + 4 => {
                let tw = visible_width(title);
                match title_align {
                    crate::config::TitleAlign::Left => {
                        let dashes = border_w.saturating_sub(tw + 3);
                        format!(
                            "{}{}{}",
                            p.paint("╭─ ", color, false),
                            p.paint(title, color, true),
                            p.paint(&format!(" {}╮", "─".repeat(dashes)), color, false),
                        )
                    }
                    crate::config::TitleAlign::Center => {
                        let remaining = border_w.saturating_sub(tw + 2);
                        let left_d = remaining / 2;
                        let right_d = remaining.saturating_sub(left_d);
                        format!(
                            "{}{}{}",
                            p.paint(&format!("╭{} ", "─".repeat(left_d)), color, false),
                            p.paint(title, color, true),
                            p.paint(&format!(" {}╮", "─".repeat(right_d)), color, false),
                        )
                    }
                    crate::config::TitleAlign::Right => {
                        let dashes = border_w.saturating_sub(tw + 3);
                        format!(
                            "{}{}{}",
                            p.paint(&format!("╭{} ", "─".repeat(dashes)), color, false),
                            p.paint(title, color, true),
                            p.paint(" ─╮", color, false),
                        )
                    }
                }
            }
            _ => p.paint(&format!("╭{}╮", "─".repeat(border_w)), color, false),
        };

        let bottom = p.paint(&format!("╰{}╯", "─".repeat(border_w)), color, false);

        out.push(Line {
            key: None,
            value: top,
            category: sec.category,
            is_divider: false,
        });

        for l in &sec.lines {
            let s = render_line(l);
            let text = if visible_width(&s) > max_w {
                truncate_ansi(&s, max_w)
            } else {
                s
            };
            let w = visible_width(&text);
            let pad = max_w.saturating_sub(w);
            let row = format!(
                "{} {text}{} {}",
                p.paint("│", color, false),
                " ".repeat(pad),
                p.paint("│", color, false)
            );
            out.push(Line {
                key: None,
                value: row,
                category: sec.category,
                is_divider: false,
            });
        }

        out.push(Line {
            key: None,
            value: bottom,
            category: sec.category,
            is_divider: false,
        });
    }

    out
}

fn truncate_ansi(s: &str, limit: usize) -> String {
    if visible_width(s) <= limit {
        return s.to_string();
    }
    if limit == 0 {
        return String::new();
    }
    let had_escape = s.contains('\x1b');
    let target = limit.saturating_sub(1);
    let mut out = String::new();
    let mut cur_w = 0;
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
            out.push(c);
        } else if in_escape {
            out.push(c);
            if c.is_ascii_alphabetic() || c == '\\' {
                in_escape = false;
            }
        } else {
            if cur_w >= target {
                break;
            }
            cur_w += 1;
            out.push(c);
        }
    }
    out.push('…');
    if had_escape {
        out.push_str("\x1b[0m");
    }
    out
}

pub fn module_nerd_icon(name: &str) -> &'static str {
    // Fast path for lower-case module names
    match name {
        "os" => return "",
        "kernel" => return "",
        "host" => return "󰌢",
        "board" => return "󱤓",
        "bios" => return "󰒋",
        "chassis" => return "󰌢",
        "uptime" => return "󱑎",
        "load" | "loadavg" => return "󰊚",
        "processes" => return "󰒋",
        "packages" => return "󰏖",
        "initsystem" | "init" => return "󰒋",
        "datetime" => return "󰅐",
        "locale" => return "󰗊",
        "users" => return "",
        "version" => return "󰞷",
        "cpu" => return "",
        "cpucache" => return "󰍛",
        "cputemp" => return "",
        "cpuusage" => return "",
        "gpu" => return "󰢮",
        "display" => return "󰍹",
        "sound" => return "󰕾",
        "memory" => return "󰘚",
        "swap" => return "󰓡",
        "disk" => return "󰋊",
        "battery" => return "󰂀",
        "vulkan" => return "󰢮",
        "opengl" => return "󰢮",
        "shell" => return "",
        "terminal" => return "",
        "terminalfont" => return "󰛖",
        "terminalsize" => return "󰞷",
        "wm" => return "󱂬",
        "wmtheme" => return "󰉼",
        "de" => return "󰧨",
        "theme" => return "󰉼",
        "icons" => return "󰀻",
        "font" => return "󰛖",
        "cursor" => return "󰆿",
        "wallpaper" => return "󰸉",
        "editor" => return "󰅩",
        "devenv" => return "",
        "git" => return "",
        "localip" => return "󰩟",
        "dns" => return "󰇖",
        "wifi" => return "󰤨",
        "netio" => return "󰛳",
        "bluetooth" => return "󰂯",
        "containers" => return "󰡨",
        "quote" => return "󰝗",
        "media" | "player" => return "󰎆",
        "brightness" => return "󰃟",
        "colors" => return "󰏘",
        "bluetoothradio" => return "󰂯",
        "physicaldisk" => return "󰋊",
        "physicalmemory" => return "󰘚",
        "diskio" => return "󰋊",
        "poweradapter" => return "󰚥",
        "lm" => return "󰧨",
        "opencl" => return "󰢮",
        "codec" => return "󰕧",
        "btrfs" => return "󰋊",
        "zpool" => return "󰋊",
        "terminaltheme" => return "󰉼",
        "top" => return "󰒋",
        "custom" => return "󰌢",
        "command" => return "",
        "monitor" => return "󰍹",
        _ => {}
    }

    match name.to_ascii_lowercase().as_str() {
        "os" => "",
        "kernel" => "",
        "host" => "󰌢",
        "board" => "󱤓",
        "bios" => "󰒋",
        "chassis" => "󰌢",
        "uptime" => "󱑎",
        "load" | "loadavg" => "󰊚",
        "processes" => "󰒋",
        "packages" => "󰏖",
        "initsystem" | "init" => "󰒋",
        "date/time" | "datetime" => "󰅐",
        "locale" => "󰗊",
        "users" => "",
        "version" => "󰞷",
        "cpu" => "",
        "cache" | "cpucache" => "󰍛",
        "cpu temp" | "cputemp" => "",
        "cpu usage" | "cpuusage" => "",
        "gpu" => "󰢮",
        "display" => "󰍹",
        "sound" => "󰕾",
        "memory" => "󰘚",
        "swap" => "󰓡",
        "disk" => "󰋊",
        "battery" => "󰂀",
        "vulkan" => "󰢮",
        "opengl" => "󰢮",
        "shell" => "",
        "terminal" => "",
        "terminal font" | "terminalfont" => "󰛖",
        "terminal size" | "terminalsize" => "󰞷",
        "wm" => "󱂬",
        "wm theme" | "wmtheme" => "󰉼",
        "de" => "󰧨",
        "theme" => "󰉼",
        "icons" => "󰀻",
        "font" => "󰛖",
        "cursor" => "󰆿",
        "wallpaper" => "󰸉",
        "editor" => "󰅩",
        "devenv" => "",
        "git" => "",
        "local ip" | "localip" => "󰩟",
        "dns" => "󰇖",
        "wi-fi" | "wifi" => "󰤨",
        "network i/o" | "netio" => "󰛳",
        "bluetooth" => "󰂯",
        "containers" => "󰡨",
        "quote" => "󰝗",
        "media" | "player" => "󰎆",
        "brightness" => "󰃟",
        "colors" => "󰏘",
        "bluetoothradio" => "󰂯",
        "physicaldisk" => "󰋊",
        "physicalmemory" => "󰘚",
        "diskio" => "󰋊",
        "poweradapter" => "󰚥",
        "lm" => "󰧨",
        "opencl" => "󰢮",
        "codec" => "󰕧",
        "btrfs" => "󰋊",
        "zpool" => "󰋊",
        "terminaltheme" => "󰉼",
        "top" => "󰒋",
        "custom" => "󰌢",
        "command" => "",
        "monitor" => "󰍹",
        _ => "󰌢",
    }
}

pub fn module_category(id: &str) -> u8 {
    // Fast path: direct match without heap allocations
    match id {
        "title" | "separator" => return 0,
        "os" | "kernel" | "bootmgr" | "initsystem" | "security" | "host" | "board" | "bios"
        | "tpm" | "chassis" | "uptime" | "loadavg" | "processes" | "packages" | "datetime"
        | "date" | "time" | "locale" | "users" | "user" | "version" => return 1,
        "shell" | "terminal" | "terminalfont" | "terminalsize" | "terminaltheme" | "wm"
        | "wmtheme" | "de" | "theme" | "icons" | "font" | "cursor" | "wallpaper" | "editor"
        | "devenv" | "git" | "display" | "displayserver" | "resolution" => return 2,
        "cpu" | "cpucache" | "cputemp" | "cpuusage" | "powerprofile" | "gpu" | "gpudriver"
        | "sound" | "audioserver" | "memory" | "ram" | "swap" | "disk" | "battery"
        | "poweradapter" | "vulkan" | "opengl" | "physicaldisk" | "physicalmemory" | "diskio"
        | "top" | "codec" => return 3,
        "netadapter" | "localip" | "dns" | "wifi" | "netio" | "publicip" | "weather" => return 4,
        "keyboard" | "mouse" | "touchpad" | "camera" | "gamepad" | "media" | "player"
        | "bluetooth" | "bluetoothradio" | "monitor" | "brightness" | "lm" | "opencl" | "btrfs"
        | "zpool" | "containers" | "command" | "custom" => return 5,
        "colors" => return 6,
        "quote" => return 7,
        _ => {}
    }

    let lower = id.trim().to_ascii_lowercase().replace(['-', '_', ' '], "");
    match lower.as_str() {
        "title" | "separator" => 0,
        // System / OS
        "os" | "kernel" | "bootmgr" | "initsystem" | "security" | "host" | "board" | "bios"
        | "tpm" | "chassis" | "uptime" | "loadavg" | "processes" | "packages" | "datetime"
        | "date" | "time" | "locale" | "users" | "user" | "version" => 1,
        // Desktop, Shell & UI
        "shell" | "terminal" | "terminalfont" | "terminalsize" | "terminaltheme" | "wm"
        | "wmtheme" | "de" | "theme" | "icons" | "font" | "cursor" | "wallpaper" | "editor"
        | "devenv" | "git" | "display" | "displayserver" | "resolution" => 2,
        // Hardware & Performance
        "cpu" | "cpucache" | "cputemp" | "cpuusage" | "powerprofile" | "gpu" | "gpudriver"
        | "sound" | "audioserver" | "memory" | "ram" | "swap" | "disk" | "battery"
        | "poweradapter" | "vulkan" | "opengl" | "physicaldisk" | "physicalmemory" | "diskio"
        | "top" | "codec" => 3,
        // Network & Connectivity
        "netadapter" | "localip" | "dns" | "wifi" | "netio" | "publicip" | "weather" => 4,
        // Colors
        "colors" => 6,
        // Quote
        "quote" => 7,
        // Peripherals, Extra & Monitoring
        _ => 5,
    }
}

pub fn category_title(cat: u8) -> Option<&'static str> {
    match cat {
        0 => None,
        1 => Some("System"),
        2 => Some("Visual"),
        3 => Some("Hardware"),
        4 => Some("Network"),
        5 => Some("Devices"),
        6 => Some("Colors"),
        7 => Some("Quote"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_border() {
        let p = Painter::new(false);
        let lines = vec![
            Line {
                key: Some("OS:".into()),
                value: "Arch".into(),
                category: None,
                is_divider: false,
            },
            Line {
                key: None,
                value: "hello".into(),
                category: None,
                is_divider: false,
            },
        ];
        let boxed = apply_border(
            &lines,
            &p,
            crate::style::Color::Default,
            None,
            crate::config::TitleAlign::Left,
        );
        assert_eq!(boxed.len(), 4);
        assert!(boxed[0].value.starts_with('╭'));
        assert!(boxed[1].value.starts_with('│'));
        assert!(boxed[3].value.starts_with('╰'));
        assert!(boxed[3].value.ends_with('╯'));
    }

    #[test]
    fn test_apply_border_with_divider() {
        let p = Painter::new(false);
        let lines = vec![
            Line {
                key: Some("OS:".into()),
                value: "Arch".into(),
                category: None,
                is_divider: false,
            },
            Line {
                key: None,
                value: String::new(),
                category: None,
                is_divider: true,
            },
            Line {
                key: Some("CPU:".into()),
                value: "Intel".into(),
                category: None,
                is_divider: false,
            },
        ];
        let boxed = apply_border(
            &lines,
            &p,
            crate::style::Color::Default,
            None,
            crate::config::TitleAlign::Left,
        );
        assert_eq!(boxed.len(), 7);
        assert!(boxed[0].value.starts_with('╭'));
        assert!(boxed[1].value.starts_with('│'));
        assert!(boxed[2].value.starts_with('╰'));
        assert!(boxed[3].value.is_empty());
        assert!(boxed[4].value.starts_with('╭'));
        assert!(boxed[5].value.starts_with('│'));
        assert!(boxed[6].value.starts_with('╰'));
    }

    #[test]
    fn test_apply_border_with_category_titles() {
        let p = Painter::new(false);
        let lines = vec![
            Line {
                key: Some("OS:".into()),
                value: "Arch".into(),
                category: Some(1),
                is_divider: false,
            },
            Line {
                key: Some("Shell:".into()),
                value: "fish".into(),
                category: Some(2),
                is_divider: false,
            },
        ];
        let boxed = apply_border(
            &lines,
            &p,
            crate::style::Color::Default,
            None,
            crate::config::TitleAlign::Left,
        );
        assert_eq!(boxed.len(), 7);
        assert!(boxed[0].value.contains("System"));
        assert!(boxed[4].value.contains("Visual"));

        // Center align
        let center_boxed = apply_border(
            &lines,
            &p,
            crate::style::Color::Default,
            None,
            crate::config::TitleAlign::Center,
        );
        assert!(center_boxed[0].value.contains(" System "));

        // Right align
        let right_boxed = apply_border(
            &lines,
            &p,
            crate::style::Color::Default,
            None,
            crate::config::TitleAlign::Right,
        );
        assert!(right_boxed[0].value.contains(" System ─╮"));
    }

    #[test]
    fn test_apply_border_with_truncation() {
        let p = Painter::new(false);
        let lines = vec![Line {
            key: Some("VeryLong:".into()),
            value: "1234567890abcdefghijklmnopqrstuvwxyz".into(),
            category: None,
            is_divider: false,
        }];
        // Limit total box to 35 columns
        let boxed = apply_border(
            &lines,
            &p,
            crate::style::Color::Default,
            Some(35),
            crate::config::TitleAlign::Left,
        );
        assert_eq!(boxed.len(), 3);
        assert!(boxed[1].value.contains('…'));
        assert_eq!(
            visible_width(&boxed[0].value),
            visible_width(&boxed[1].value)
        );
        assert_eq!(
            visible_width(&boxed[1].value),
            visible_width(&boxed[2].value)
        );
        assert!(boxed[0].value.starts_with('╭'));
        assert!(boxed[2].value.starts_with('╰'));
        assert!(!boxed[1].value.contains('\x1b'));
    }

    #[test]
    fn test_truncate_ansi_preserves_color_flag() {
        let plain = "Hello, world! This is a long string";
        let truncated = truncate_ansi(plain, 10);
        assert!(truncated.contains('…'));
        assert!(!truncated.contains('\x1b'));

        let colored = "\x1b[31mHello, world! This is a long string\x1b[0m";
        let truncated_colored = truncate_ansi(colored, 10);
        assert!(truncated_colored.contains('…'));
        assert!(truncated_colored.ends_with("\x1b[0m"));
    }

    #[test]
    fn test_compose_top() {
        let lines = vec![Line {
            key: Some("OS:".into()),
            value: "Arch".into(),
            category: None,
            is_divider: false,
        }];
        let out = compose_top("LOGO", &lines);
        assert_eq!(out, "LOGO\n\nOS: Arch");
    }

    #[test]
    fn test_align_progress_bars() {
        let mut lines = vec![
            Line {
                key: Some("Memory:".into()),
                value: "3.96 GiB / 7.62 GiB [██████████░░░░░░░░░░] 52%".into(),
                category: None,
                is_divider: false,
            },
            Line {
                key: Some("Swap:".into()),
                value: "1.17 GiB / 12.0 GiB [██░░░░░░░░░░░░░░░░░░] 10%".into(),
                category: None,
                is_divider: false,
            },
            Line {
                key: Some("Disk:".into()),
                value: "/     29.0 GiB / 118 GiB [█████░░░░░░░░░░░░░░░] (25%)".into(),
                category: None,
                is_divider: false,
            },
            Line {
                key: Some("      ".into()),
                value: "/boot 83.9 MiB / 1022 MiB [██░░░░░░░░░░░░░░░░░░] (8%)".into(),
                category: None,
                is_divider: false,
            },
        ];
        align_progress_bars(&mut lines);
        let pos0 = lines[0].value.find('[').unwrap();
        let pos1 = lines[1].value.find('[').unwrap();
        let pos2 = lines[2].value.find('[').unwrap();
        let pos3 = lines[3].value.find('[').unwrap();

        let w0 = visible_width(&format!("{} ", lines[0].key.as_ref().unwrap())) + pos0;
        let w1 = visible_width(&format!("{} ", lines[1].key.as_ref().unwrap())) + pos1;
        let w2 = visible_width(&format!("{} ", lines[2].key.as_ref().unwrap())) + pos2;
        let w3 = visible_width(&format!("{} ", lines[3].key.as_ref().unwrap())) + pos3;

        assert_eq!(w0, w1);
        assert_eq!(w1, w2);
        assert_eq!(w2, w3);
    }

    #[test]
    fn test_to_json_strips_ansi_and_skips_colors() {
        let results = vec![
            (
                "memory",
                Some(ModuleOutput::new(
                    "Memory",
                    "3.82 GiB [\x1b[32m████\x1b[90m░░░░\x1b[0m] 50%",
                )),
            ),
            (
                "colors",
                Some(ModuleOutput::new("Colors", "\x1b[31m███\x1b[0m")),
            ),
        ];
        let json = to_json(&results);
        assert!(!json.contains("\x1b"));
        assert!(json.contains("3.82 GiB [████░░░░] 50%"));
        assert!(!json.contains("Colors"));
    }
}
