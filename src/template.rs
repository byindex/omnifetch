use crate::ProgressBar;
use crate::config;
use crate::sys;
use std::cell::Cell;

#[derive(Debug, Clone, Default)]
pub struct TemplateValue {
    pub text: Option<String>,
    pub bytes: Option<u64>,
    pub number: Option<f64>,
}

impl TemplateValue {
    pub fn from_text(s: impl Into<String>) -> Self {
        Self {
            text: Some(s.into()),
            bytes: None,
            number: None,
        }
    }

    pub fn from_bytes(b: u64) -> Self {
        Self {
            text: None,
            bytes: Some(b),
            number: Some(b as f64),
        }
    }

    pub fn from_number(n: f64) -> Self {
        Self {
            text: None,
            bytes: None,
            number: Some(n),
        }
    }

    pub fn text(&self) -> String {
        if let Some(ref t) = self.text {
            t.clone()
        } else if let Some(b) = self.bytes {
            sys::human_bytes(b)
        } else if let Some(n) = self.number {
            format!("{n:.0}")
        } else {
            String::new()
        }
    }
}

use std::borrow::Cow;

pub struct Context {
    pub module_id: &'static str,
    pub used: Cell<Option<u64>>,
    pub total: Option<u64>,
    pub pct: Cell<Option<f64>>,
    pub vars: Vec<(Cow<'static, str>, TemplateValue)>,
}

impl Context {
    pub fn new(module_id: &'static str) -> Self {
        Self {
            module_id,
            used: Cell::new(None),
            total: None,
            pct: Cell::new(None),
            vars: Vec::with_capacity(12),
        }
    }

    pub fn with_metric(mut self, used: u64, total: u64, pct: f64) -> Self {
        self.used = Cell::new(Some(used));
        self.total = Some(total);
        self.pct = Cell::new(Some(pct));
        self
    }

    pub fn set_str(&mut self, key: impl Into<Cow<'static, str>>, val: impl Into<String>) -> &mut Self {
        self.vars.push((key.into(), TemplateValue::from_text(val)));
        self
    }

    pub fn set_bytes(&mut self, key: impl Into<Cow<'static, str>>, bytes: u64) -> &mut Self {
        self.vars.push((key.into(), TemplateValue::from_bytes(bytes)));
        self
    }

    pub fn set_num(&mut self, key: impl Into<Cow<'static, str>>, num: f64) -> &mut Self {
        self.vars.push((key.into(), TemplateValue::from_number(num)));
        self
    }

    pub fn get_val(&self, key: &str) -> Option<&TemplateValue> {
        self.vars.iter().rev().find(|(k, _)| k.as_ref() == key).map(|(_, v)| v)
    }
}

pub fn render_template(template: &str, ctx: &Context) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                out.push('{');
                i += 2;
                continue;
            }
            if let Some(close_rel) = template[i + 1..].find('}') {
                let expr = &template[i + 1..i + 1 + close_rel];
                let trimmed = expr.trim();
                if let Some(val) = eval_expression(trimmed, ctx) {
                    out.push_str(&val);
                } else if let Some(color_code) = color_escape(trimmed) {
                    if config::get().color {
                        out.push_str(color_code);
                    }
                } else {
                    out.push('{');
                    out.push_str(expr);
                    out.push('}');
                }
                i = i + 1 + close_rel + 1;
            } else {
                out.push_str(&template[i..]);
                break;
            }
        } else if bytes[i] == b'}' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'}' {
                out.push('}');
                i += 2;
                continue;
            }
            out.push('}');
            i += 1;
        } else {
            let start = i;
            while i < bytes.len() && bytes[i] != b'{' && bytes[i] != b'}' {
                i += 1;
            }
            out.push_str(&template[start..i]);
        }
    }
    out
}

/// Backwards-compatible closure-based formatter
pub fn format_template<F>(template: &str, mut lookup: F) -> String
where
    F: FnMut(&str) -> Option<String>,
{
    let mut out = String::with_capacity(template.len() + 16);
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                out.push('{');
                i += 2;
                continue;
            }
            if let Some(close_rel) = template[i + 1..].find('}') {
                let key = &template[i + 1..i + 1 + close_rel];
                if let Some(val) = lookup(key) {
                    out.push_str(&val);
                } else if let Some(color_code) = color_escape(key) {
                    if config::get().color {
                        out.push_str(color_code);
                    }
                } else {
                    out.push('{');
                    out.push_str(key);
                    out.push('}');
                }
                i = i + 1 + close_rel + 1;
            } else {
                out.push_str(&template[i..]);
                break;
            }
        } else if bytes[i] == b'}' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'}' {
                out.push('}');
                i += 2;
                continue;
            }
            out.push('}');
            i += 1;
        } else {
            let start = i;
            while i < bytes.len() && bytes[i] != b'{' && bytes[i] != b'}' {
                i += 1;
            }
            out.push_str(&template[start..i]);
        }
    }
    out
}

/// Evaluates a single template expression (e.g. `{used:gib}`, `{name | upper}`).
pub fn eval_expression(expr: &str, ctx: &Context) -> Option<String> {
    let expr = expr.trim();
    if expr.is_empty() {
        return None;
    }

    if has_pipe_outside_quotes(expr) {
        let parts = split_outside_quotes(expr, '|');
        if let Some((first, rest)) = parts.split_first() {
            let mut current = eval_simple_expression(first.trim(), ctx)?;
            for filter in rest {
                current = apply_filter(filter.trim(), &current, ctx);
            }
            return Some(current);
        }
    }

    if has_colon_outside_quotes_and_parens(expr) {
        let parts = split_colon_chain(expr);
        if let Some((first, rest)) = parts.split_first() {
            if matches!(first.trim(), "bar" | "b") {
                return Some(eval_bar_colon(rest, ctx));
            }
            let head = first.trim();
            // Converters want the number, not the formatted string.
            let source = ctx.get_val(head).is_some().then_some(head);
            let mut current = eval_simple_expression(head, ctx)?;
            for filter in rest {
                current = apply_filter_to(filter.trim(), &current, ctx, source);
            }
            return Some(current);
        }
    }

    eval_simple_expression(expr, ctx)
}

fn eval_simple_expression(expr: &str, ctx: &Context) -> Option<String> {
    let expr = expr.trim();

    // Arithmetic and ranges (`used + 2 GiB`) are tried before filters and bare values.
    if let Some(val) = eval_math_expression(expr, ctx) {
        return Some(val);
    }

    if let Some((fn_name, args_str)) = parse_function_call(expr) {
        let args = parse_args(args_str);
        return eval_function(fn_name, &args, ctx);
    }

    // Dynamic pct / p value (if updated by math on used or pct)
    if (expr == "pct" || expr == "p")
        && let Some(p) = ctx.pct.get()
    {
        return Some(format!("{p:.0}"));
    }

    if let Some(val) = ctx.get_val(expr) {
        return Some(val.text());
    }

    if expr == "bar" || expr == "b" {
        return Some(render_bar(ctx, None, None, None));
    }

    None
}

fn eval_function(name: &str, args: &[String], ctx: &Context) -> Option<String> {
    match name.to_ascii_lowercase().as_str() {
        "bar" | "b" => {
            let width = args.first().and_then(|w| w.parse::<usize>().ok());
            let fill = args.get(1).map(String::as_str);
            let empty = args.get(2).map(String::as_str);
            Some(render_bar(ctx, width, fill, empty))
        }
        "bar_color" => {
            let width = args.first().and_then(|w| w.parse::<usize>().ok());
            Some(render_color_bar(ctx, width))
        }

        "gib" | "gb" => {
            let val = resolve_first_arg_as_bytes(args, ctx)?;
            Some(format!("{:.2} GiB", val as f64 / 1_073_741_824.0))
        }
        "mib" | "mb" => {
            let val = resolve_first_arg_as_bytes(args, ctx)?;
            Some(format!("{:.1} MiB", val as f64 / 1_048_576.0))
        }
        "kib" | "kb" => {
            let val = resolve_first_arg_as_bytes(args, ctx)?;
            Some(format!("{:.0} KiB", val as f64 / 1024.0))
        }
        "raw" | "bytes" => {
            let val = resolve_first_arg_as_bytes(args, ctx)?;
            Some(val.to_string())
        }
        "human" => {
            let val = resolve_first_arg_as_bytes(args, ctx)?;
            Some(sys::human_bytes(val))
        }

        "upper" | "uppercase" => {
            let s = resolve_first_arg_as_str(args, ctx);
            Some(s.to_uppercase())
        }
        "lower" | "lowercase" => {
            let s = resolve_first_arg_as_str(args, ctx);
            Some(s.to_lowercase())
        }
        "title" | "capitalize" => {
            let s = resolve_first_arg_as_str(args, ctx);
            Some(
                s.split_whitespace()
                    .map(|word| {
                        let mut c = word.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        }
        "trunc" | "truncate" => {
            let s = resolve_first_arg_as_str(args, ctx);
            let limit = args
                .get(1)
                .and_then(|l| l.parse::<usize>().ok())
                .unwrap_or(30);
            if s.chars().count() > limit {
                Some(s.chars().take(limit.saturating_sub(1)).collect::<String>() + "…")
            } else {
                Some(s)
            }
        }
        "replace" => {
            let s = resolve_first_arg_as_str(args, ctx);
            let from = args.get(1).map(String::as_str).unwrap_or("");
            let to = args.get(2).map(String::as_str).unwrap_or("");
            Some(s.replace(from, to))
        }
        "remove" => {
            let s = resolve_first_arg_as_str(args, ctx);
            let pat = args.get(1).map(String::as_str).unwrap_or("");
            Some(s.replace(pat, ""))
        }
        "trim" => {
            let s = resolve_first_arg_as_str(args, ctx);
            Some(s.trim().to_string())
        }
        "pad_left" => {
            let s = resolve_first_arg_as_str(args, ctx);
            let w = args
                .get(1)
                .and_then(|w| w.parse::<usize>().ok())
                .unwrap_or(0);
            // Padding left means the text itself is right-aligned.
            Some(format!("{s:>w$}"))
        }
        "pad_right" => {
            let s = resolve_first_arg_as_str(args, ctx);
            let w = args
                .get(1)
                .and_then(|w| w.parse::<usize>().ok())
                .unwrap_or(0);
            // Padding right means the text itself is left-aligned.
            Some(format!("{s:<w$}"))
        }

        "round" => {
            let s = resolve_first_arg_as_str(args, ctx);
            let prec = args
                .get(1)
                .and_then(|p| p.parse::<usize>().ok())
                .unwrap_or(0);
            if let Ok(num) = s.parse::<f64>() {
                Some(format!("{num:.prec$}"))
            } else {
                Some(s)
            }
        }

        "default" => {
            let s = resolve_first_arg_as_str(args, ctx);
            if s.trim().is_empty() {
                Some(args.get(1).cloned().unwrap_or_default())
            } else {
                Some(s)
            }
        }

        "env" => {
            let var_name = args.first().map(String::as_str)?;
            Some(std::env::var(var_name).unwrap_or_default())
        }
        "date" | "time" => {
            let fmt = args
                .first()
                .map(String::as_str)
                .unwrap_or("%Y-%m-%d %H:%M:%S");
            Some(format_current_time(fmt))
        }

        _ => None,
    }
}

fn apply_filter(filter_expr: &str, input: &str, ctx: &Context) -> String {
    apply_filter_to(filter_expr, input, ctx, None)
}

/// One filter application. `source` lets byte converters recover the raw value.
fn apply_filter_to(filter_expr: &str, input: &str, ctx: &Context, source: Option<&str>) -> String {
    let filter_expr = filter_expr.trim();

    if source.is_some()
        && needs_raw_value(filter_expr)
        && let Some(s) = source
        && let Some(raw) = raw_value_text(s, ctx)
        && let Some(res) = eval_function(filter_expr, &[raw], ctx)
    {
        return res;
    }

    if let Some((fn_name, args_str)) = parse_function_call(filter_expr) {
        let mut args = vec![input.to_string()];
        args.extend(parse_args(args_str));
        if let Some(res) = eval_function(fn_name, &args, ctx) {
            return res;
        }
    }

    let mut args = vec![input.to_string()];
    match filter_expr.to_ascii_lowercase().as_str() {
        "gib" | "gb" | "mib" | "mb" | "kib" | "kb" | "raw" | "human" | "upper" | "lower"
        | "title" | "trim" => {
            if let Some(res) = eval_function(filter_expr, &args, ctx) {
                return res;
            }
        }
        other if other.parse::<usize>().is_ok() => {
            args.push(other.to_string());
            if let Some(res) = eval_function("round", &args, ctx) {
                return res;
            }
        }
        _ => {}
    }

    input.to_string()
}

/// Filters that must see the raw number rather than the formatted string.
fn needs_raw_value(filter_expr: &str) -> bool {
    matches!(
        filter_expr.trim().to_ascii_lowercase().as_str(),
        "gib" | "gb" | "mib" | "mb" | "kib" | "kb" | "raw" | "bytes" | "human"
    )
}

/// The unformatted value behind a variable, as text.
fn raw_value_text(name: &str, ctx: &Context) -> Option<String> {
    let v = ctx.get_val(name)?;
    match &v.bytes {
        Some(b) => Some(b.to_string()),
        None => v.text().parse::<f64>().ok().map(|n| n.to_string()),
    }
}

fn eval_bar_colon(rest: &[String], ctx: &Context) -> String {
    let width = rest.first().and_then(|w| w.parse::<usize>().ok());
    let fill = rest.get(1).map(String::as_str);
    let empty = rest.get(2).map(String::as_str);
    render_bar(ctx, width, fill, empty)
}

fn render_bar(
    ctx: &Context,
    width: Option<usize>,
    fill: Option<&str>,
    empty: Option<&str>,
) -> String {
    let cfg = config::get();
    let w = width.unwrap_or(cfg.bar.width);
    let f = fill.unwrap_or(&cfg.bar.fill);
    let e = empty.unwrap_or(&cfg.bar.empty);
    let use_color = cfg.color && (fill.is_none() || fill == Some(&cfg.bar.fill));

    match (ctx.used.get(), ctx.total) {
        (Some(u), Some(t)) => ProgressBar::render_colored(u, t, w, f, e, use_color),
        _ => String::new(),
    }
}

fn render_color_bar(ctx: &Context, width: Option<usize>) -> String {
    let cfg = config::get();
    let w = width.unwrap_or(cfg.bar.width);
    let pct = ctx.pct.get().unwrap_or(0.0);

    let (color_start, color_end) = if cfg.color {
        if pct < 60.0 {
            ("\x1b[32m", "\x1b[0m")
        } else if pct < 85.0 {
            ("\x1b[33m", "\x1b[0m")
        } else {
            ("\x1b[31m", "\x1b[0m")
        }
    } else {
        ("", "")
    };

    let bar = render_bar(ctx, Some(w), None, None);
    format!("{color_start}{bar}{color_end}")
}

fn resolve_first_arg_as_str(args: &[String], ctx: &Context) -> String {
    let Some(first) = args.first() else {
        return String::new();
    };
    if let Some(val) = ctx.get_val(first) {
        return val.text();
    }
    first.clone()
}

fn resolve_first_arg_as_bytes(args: &[String], ctx: &Context) -> Option<u64> {
    let first = args.first()?;
    if let Some(val) = ctx.get_val(first) {
        if let Some(b) = val.bytes {
            return Some(b);
        }
        if let Ok(b) = val.text().parse::<u64>() {
            return Some(b);
        }
    }
    first.parse::<u64>().ok()
}

fn parse_function_call(expr: &str) -> Option<(&str, &str)> {
    let open = expr.find('(')?;
    if !expr.ends_with(')') {
        return None;
    }
    let fn_name = expr[..open].trim();
    let args_str = &expr[open + 1..expr.len() - 1];
    Some((fn_name, args_str))
}

pub fn parse_args(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut cur = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut was_quoted = false;

    for ch in s.chars() {
        match ch {
            '\'' if !in_double => {
                in_single = !in_single;
                was_quoted = true;
            }
            '"' if !in_single => {
                in_double = !in_double;
                was_quoted = true;
            }
            ',' if !in_single && !in_double => {
                if !was_quoted {
                    cur = cur.trim().to_string();
                }
                args.push(cur);
                cur = String::new();
                was_quoted = false;
            }
            _ => {
                // Quotes make everything literal. Outside them only buffered runs matter.
                if in_single || in_double || !cur.is_empty() || !ch.is_whitespace() {
                    cur.push(ch);
                }
            }
        }
    }
    if !was_quoted {
        cur = cur.trim().to_string();
    }
    if !cur.is_empty() || !args.is_empty() || was_quoted {
        args.push(cur);
    }
    args
}

fn has_pipe_outside_quotes(s: &str) -> bool {
    let mut in_s = false;
    let mut in_d = false;
    for ch in s.chars() {
        match ch {
            '\'' if !in_d => in_s = !in_s,
            '"' if !in_s => in_d = !in_d,
            '|' if !in_s && !in_d => return true,
            _ => {}
        }
    }
    false
}

fn has_colon_outside_quotes_and_parens(s: &str) -> bool {
    let mut in_s = false;
    let mut in_d = false;
    let mut paren_depth: usize = 0;
    for ch in s.chars() {
        match ch {
            '\'' if !in_d => in_s = !in_s,
            '"' if !in_s => in_d = !in_d,
            '(' if !in_s && !in_d => paren_depth += 1,
            ')' if !in_s && !in_d => paren_depth = paren_depth.saturating_sub(1),
            ':' if !in_s && !in_d && paren_depth == 0 => return true,
            _ => {}
        }
    }
    false
}

fn split_outside_quotes(s: &str, delimiter: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut in_s = false;
    let mut in_d = false;

    for ch in s.chars() {
        match ch {
            '\'' if !in_d => {
                in_s = !in_s;
                cur.push(ch);
            }
            '"' if !in_s => {
                in_d = !in_d;
                cur.push(ch);
            }
            c if c == delimiter && !in_s && !in_d => {
                parts.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur.trim().to_string());
    }
    parts
}

fn split_colon_chain(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut in_s = false;
    let mut in_d = false;
    let mut paren_depth: usize = 0;

    for ch in s.chars() {
        match ch {
            '\'' if !in_d => {
                in_s = !in_s;
                cur.push(ch);
            }
            '"' if !in_s => {
                in_d = !in_d;
                cur.push(ch);
            }
            '(' if !in_s && !in_d => {
                paren_depth += 1;
                cur.push(ch);
            }
            ')' if !in_s && !in_d => {
                paren_depth = paren_depth.saturating_sub(1);
                cur.push(ch);
            }
            ':' if !in_s && !in_d && paren_depth == 0 => {
                parts.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur.trim().to_string());
    }
    parts
}

fn format_current_time(fmt: &str) -> String {
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        let mut buf = [0u8; 128];
        let c_fmt = std::ffi::CString::new(fmt).unwrap_or_default();
        let len = libc::strftime(
            buf.as_mut_ptr() as *mut libc::c_char,
            buf.len(),
            c_fmt.as_ptr(),
            &tm,
        );
        if len > 0 {
            String::from_utf8_lossy(&buf[..len]).to_string()
        } else {
            String::new()
        }
    }
}

fn color_escape(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "black" => Some("\x1b[30m"),
        "red" => Some("\x1b[31m"),
        "green" => Some("\x1b[32m"),
        "yellow" => Some("\x1b[33m"),
        "blue" => Some("\x1b[34m"),
        "magenta" => Some("\x1b[35m"),
        "cyan" => Some("\x1b[36m"),
        "white" => Some("\x1b[37m"),
        "bright_black" | "gray" | "grey" => Some("\x1b[90m"),
        "bright_red" => Some("\x1b[91m"),
        "bright_green" => Some("\x1b[92m"),
        "bright_yellow" => Some("\x1b[93m"),
        "bright_blue" => Some("\x1b[94m"),
        "bright_magenta" => Some("\x1b[95m"),
        "bright_cyan" => Some("\x1b[96m"),
        "bright_white" => Some("\x1b[97m"),
        "bold" => Some("\x1b[1m"),
        "reset" => Some("\x1b[0m"),
        _ => None,
    }
}

// Helpers for backward compatibility with modules
pub fn parse_key_modifier(key: &str) -> (&str, Option<&str>) {
    let key = key.trim();
    if let Some((base, rest)) = key.split_once(':') {
        (base.trim(), Some(rest.trim()))
    } else if let Some((base, rest)) = key.split_once('(') {
        let rest = rest.strip_suffix(')').unwrap_or(rest).trim();
        (base.trim(), Some(rest))
    } else {
        (key, None)
    }
}

pub fn format_bytes(bytes: u64, modifier: Option<&str>) -> String {
    match modifier.map(str::to_ascii_lowercase).as_deref() {
        Some("gib") | Some("gb") => format!("{:.2} GiB", bytes as f64 / 1_073_741_824.0),
        Some("mib") | Some("mb") => format!("{:.1} MiB", bytes as f64 / 1_048_576.0),
        Some("kib") | Some("kb") => format!("{:.0} KiB", bytes as f64 / 1024.0),
        Some("raw") | Some("b") | Some("bytes") => bytes.to_string(),
        _ => sys::human_bytes(bytes),
    }
}

fn random_f64(min: f64, max: f64) -> f64 {
    if min >= max {
        return min;
    }
    thread_local! {
        static RNG: Cell<u64> = Cell::new({
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(123456789);
            let pid = std::process::id() as u64;
            nanos ^ (pid << 32) ^ 0x9e3779b97f4a7c15
        });
    }
    let r = RNG.with(|rng| {
        let mut x = rng.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        rng.set(x);
        x
    });
    let norm = (r as f64) / (u64::MAX as f64);
    min + norm * (max - min)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Unit {
    Bytes(f64),
    Percent,
    Number,
}

#[derive(Debug, Clone, Copy)]
enum Operand {
    Bytes(f64),
    Percent(f64),
    Number(f64),
}

fn parse_unit(s: &str) -> Option<Unit> {
    let lower = s.trim().to_ascii_lowercase();
    match lower.as_str() {
        "b" | "byte" | "bytes" => Some(Unit::Bytes(1.0)),
        "k" | "kb" | "kib" => Some(Unit::Bytes(1024.0)),
        "m" | "mb" | "mib" => Some(Unit::Bytes(1024.0 * 1024.0)),
        "g" | "gb" | "gib" => Some(Unit::Bytes(1024.0 * 1024.0 * 1024.0)),
        "t" | "tb" | "tib" => Some(Unit::Bytes(1024.0 * 1024.0 * 1024.0 * 1024.0)),
        "%" => Some(Unit::Percent),
        "" => Some(Unit::Number),
        _ => None,
    }
}

fn parse_scalar_number_and_unit(s: &str) -> Option<(f64, Unit)> {
    let s = s.trim();
    let mut num_end = 0;
    let mut has_digit = false;
    for (i, c) in s.char_indices() {
        if c.is_ascii_digit() || (c == '.' && has_digit) || (i == 0 && (c == '-' || c == '+')) {
            if c.is_ascii_digit() {
                has_digit = true;
            }
            num_end = i + c.len_utf8();
        } else {
            break;
        }
    }
    if !has_digit {
        return None;
    }
    let num_str = &s[..num_end];
    let unit_str = s[num_end..].trim();
    let num: f64 = num_str.parse().ok()?;
    let unit = parse_unit(unit_str)?;
    Some((num, unit))
}

fn parse_number_and_unit(s: &str) -> Option<(f64, Unit)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    if let Some((min_str, max_str)) = s.split_once("..") {
        let min_val: f64 = min_str.trim().parse().ok()?;
        let (max_val, unit) = parse_scalar_number_and_unit(max_str.trim())?;
        let chosen = random_f64(min_val, max_val);
        return Some((chosen, unit));
    }

    parse_scalar_number_and_unit(s)
}

fn find_binary_op(expr: &str) -> Option<(usize, char)> {
    let bytes = expr.as_bytes();
    let len = bytes.len();
    if len < 3 {
        return None;
    }

    // Pass 1: '+' or '-' (not part of range "..")
    for i in 1..len - 1 {
        let b = bytes[i];
        if b == b'+' {
            return Some((i, '+'));
        }
        if b == b'-' && bytes[i - 1] != b'.' && bytes[i + 1] != b'.' {
            return Some((i, '-'));
        }
    }

    // Pass 2: '*' or '/'
    for (i, &b) in bytes.iter().enumerate().take(len - 1).skip(1) {
        if b == b'*' || b == b'/' {
            return Some((i, b as char));
        }
    }

    None
}

fn resolve_operand(token: &str, ctx: &Context) -> Option<Operand> {
    let token = token.trim();
    if let Some(v) = ctx.get_val(token) {
        if let Some(b) = v.bytes {
            return Some(Operand::Bytes(b as f64));
        }
        if let Some(n) = v.number {
            if token == "pct" || token == "p" {
                return Some(Operand::Percent(n));
            }
            return Some(Operand::Number(n));
        }
        let text = v.text();
        if let Ok(n) = text.trim_end_matches('%').trim().parse::<f64>() {
            if text.ends_with('%') {
                return Some(Operand::Percent(n));
            }
            return Some(Operand::Number(n));
        }
    }

    let (num, unit) = parse_number_and_unit(token)?;
    match unit {
        Unit::Bytes(mult) => Some(Operand::Bytes(num * mult)),
        Unit::Percent => Some(Operand::Percent(num)),
        Unit::Number => Some(Operand::Number(num)),
    }
}

fn format_pct_value(p: f64) -> String {
    if p.fract() == 0.0 {
        format!("{p:.0}%")
    } else {
        format!("{p:.1}%")
    }
}

fn format_num_value(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{n:.0}")
    } else {
        let s = format!("{n:.2}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn format_operand_value(val: f64, unit: Unit) -> String {
    match unit {
        Unit::Bytes(mult) => sys::human_bytes((val * mult) as u64),
        Unit::Percent => format_pct_value(val),
        Unit::Number => format_num_value(val),
    }
}

fn update_ctx_bytes(ctx: &Context, var_name: &str, new_bytes: u64) {
    if var_name == "used" || var_name == "u" {
        let new_kb = new_bytes / 1024;
        ctx.used.set(Some(new_kb));
        if let Some(total_kb) = ctx.total
            && total_kb > 0
        {
            let new_pct = (new_kb as f64 / total_kb as f64) * 100.0;
            ctx.pct.set(Some(new_pct));
        }
    }
}

fn update_ctx_pct(ctx: &Context, var_name: &str, new_pct: f64) {
    if var_name == "pct" || var_name == "p" {
        ctx.pct.set(Some(new_pct));
        if let Some(total_kb) = ctx.total {
            let new_kb = ((new_pct / 100.0) * total_kb as f64) as u64;
            ctx.used.set(Some(new_kb));
        }
    }
}

fn evaluate_binary_op(
    left_name: &str,
    left: Operand,
    op: char,
    right: Operand,
    ctx: &Context,
) -> Option<String> {
    match (left, op, right) {
        // --- BYTES OPERATIONS ---
        (Operand::Bytes(l), '+', Operand::Bytes(r)) => {
            let res = l + r;
            update_ctx_bytes(ctx, left_name, res as u64);
            Some(sys::human_bytes(res.max(0.0) as u64))
        }
        (Operand::Bytes(l), '-', Operand::Bytes(r)) => {
            let res = (l - r).max(0.0);
            update_ctx_bytes(ctx, left_name, res as u64);
            Some(sys::human_bytes(res as u64))
        }
        (Operand::Bytes(l), '+', Operand::Percent(pct)) => {
            let total = ctx.total.map(|t| (t * 1024) as f64).unwrap_or(l);
            let delta = total * (pct / 100.0);
            let res = l + delta;
            update_ctx_bytes(ctx, left_name, res as u64);
            Some(sys::human_bytes(res.max(0.0) as u64))
        }
        (Operand::Bytes(l), '-', Operand::Percent(pct)) => {
            let total = ctx.total.map(|t| (t * 1024) as f64).unwrap_or(l);
            let delta = total * (pct / 100.0);
            let res = (l - delta).max(0.0);
            update_ctx_bytes(ctx, left_name, res as u64);
            Some(sys::human_bytes(res as u64))
        }
        (Operand::Bytes(l), '*', Operand::Number(n)) => {
            let res = (l * n).max(0.0);
            update_ctx_bytes(ctx, left_name, res as u64);
            Some(sys::human_bytes(res as u64))
        }
        (Operand::Bytes(l), '/', Operand::Number(n)) => {
            let res = if n != 0.0 { (l / n).max(0.0) } else { l };
            update_ctx_bytes(ctx, left_name, res as u64);
            Some(sys::human_bytes(res as u64))
        }

        // --- PERCENT OPERATIONS ---
        (Operand::Percent(l), '+', Operand::Percent(r))
        | (Operand::Percent(l), '+', Operand::Number(r)) => {
            let res = (l + r).min(100.0);
            update_ctx_pct(ctx, left_name, res);
            Some(format_pct_value(res))
        }
        (Operand::Percent(l), '-', Operand::Percent(r))
        | (Operand::Percent(l), '-', Operand::Number(r)) => {
            let res = (l - r).max(0.0);
            update_ctx_pct(ctx, left_name, res);
            Some(format_pct_value(res))
        }
        (Operand::Percent(l), '*', Operand::Number(n)) => {
            let res = (l * n).clamp(0.0, 100.0);
            update_ctx_pct(ctx, left_name, res);
            Some(format_pct_value(res))
        }
        (Operand::Percent(l), '/', Operand::Number(n)) => {
            let res = if n != 0.0 {
                (l / n).clamp(0.0, 100.0)
            } else {
                l
            };
            update_ctx_pct(ctx, left_name, res);
            Some(format_pct_value(res))
        }

        // --- NUMBER OPERATIONS ---
        (Operand::Number(l), '+', Operand::Percent(r))
            if left_name == "pct" || left_name == "p" =>
        {
            let res = (l + r).min(100.0);
            update_ctx_pct(ctx, left_name, res);
            Some(format_pct_value(res))
        }
        (Operand::Number(l), '-', Operand::Percent(r))
            if left_name == "pct" || left_name == "p" =>
        {
            let res = (l - r).max(0.0);
            update_ctx_pct(ctx, left_name, res);
            Some(format_pct_value(res))
        }
        (Operand::Number(l), '+', Operand::Number(r)) => {
            let res = l + r;
            Some(format_num_value(res))
        }
        (Operand::Number(l), '-', Operand::Number(r)) => {
            let res = l - r;
            Some(format_num_value(res))
        }
        (Operand::Number(l), '*', Operand::Number(r)) => {
            let res = l * r;
            Some(format_num_value(res))
        }
        (Operand::Number(l), '/', Operand::Number(r)) => {
            let res = if r != 0.0 { l / r } else { l };
            Some(format_num_value(res))
        }
        _ => None,
    }
}

pub fn eval_math_expression(expr: &str, ctx: &Context) -> Option<String> {
    let expr = expr.trim();
    if expr.is_empty() {
        return None;
    }

    if let Some((op_idx, op)) = find_binary_op(expr) {
        let left_str = expr[..op_idx].trim();
        let right_str = expr[op_idx + 1..].trim();
        if left_str.is_empty() || right_str.is_empty() {
            return None;
        }

        let left_op = resolve_operand(left_str, ctx)?;
        let right_op = resolve_operand(right_str, ctx)?;

        return evaluate_binary_op(left_str, left_op, op, right_op, ctx);
    }

    // Check if it's a standalone random range, e.g. "2..4 GiB" or "2..4%" or "2..4"
    if expr.contains("..")
        && let Some((val, unit)) = parse_number_and_unit(expr)
    {
        return Some(format_operand_value(val, unit));
    }

    None
}

pub fn format_pct(pct: f64, modifier: Option<&str>) -> String {
    match modifier {
        Some("1") => format!("{pct:.1}"),
        Some("2") => format!("{pct:.2}"),
        _ => format!("{pct:.0}"),
    }
}

pub fn format_bar(used: u64, total: u64, modifier: Option<&str>) -> String {
    let cfg = config::get();
    let width = modifier
        .and_then(|m| m.parse::<usize>().ok())
        .unwrap_or(cfg.bar.width);
    ProgressBar::render_colored(used, total, width, &cfg.bar.fill, &cfg.bar.empty, cfg.color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_requested_literal_mix() {
        let mut ctx = Context::new("memory");
        ctx.set_bytes("free", 4_554_805_248);
        ctx.set_bytes("f", 4_554_805_248);

        let tmpl = "{f} free of 1337GB";
        let res = render_template(tmpl, &ctx);
        assert_eq!(res, "4.24 GiB free of 1337GB");
    }

    #[test]
    fn test_function_calls() {
        let mut ctx = Context::new("memory");
        ctx.set_bytes("f", 4_554_805_248);
        ctx.set_str("name", "Intel(R) Core(TM) i5-3360M CPU @ 2.80GHz");

        assert_eq!(render_template("{gib(f)}", &ctx), "4.24 GiB");
        assert_eq!(render_template("{upper(f)}", &ctx), "4.24 GIB");
        assert_eq!(render_template("{trunc(name, 10)}", &ctx), "Intel(R) …");
    }

    #[test]
    fn test_pipes() {
        let mut ctx = Context::new("cpu");
        ctx.set_str("name", "Intel(R) Core(TM) i5-3360M");

        let tmpl = "{name | remove('Intel(R) ') | upper | trunc(12)}";
        let res = render_template(tmpl, &ctx);
        assert_eq!(res, "CORE(TM) I5…");
    }

    #[test]
    fn test_bar_custom_chars() {
        let mut ctx = Context::new("memory");
        ctx.used.set(Some(50));
        ctx.total = Some(100);

        let tmpl = "{bar(10, '=', '-')}";
        let res = render_template(tmpl, &ctx);
        assert_eq!(res, "[=====-----]");
    }

    #[test]
    fn test_math_expressions() {
        let mut ctx = Context::new("memory");
        // 8 GiB total, 3 GiB used.
        let total_bytes = 8 * 1024 * 1024 * 1024;
        let used_bytes = 3 * 1024 * 1024 * 1024;
        ctx = ctx.with_metric(3 * 1024 * 1024, 8 * 1024 * 1024, 37.5);
        ctx.set_bytes("used", used_bytes);
        ctx.set_bytes("total", total_bytes);
        ctx.set_num("pct", 37.5);

        let res = render_template("{used + 2 GiB}", &ctx);
        assert_eq!(res, "5.00 GiB");

        let res = render_template("{used - 1 GiB}", &ctx);
        assert_eq!(res, "2.00 GiB");

        let res = render_template("{used * 2}", &ctx);
        assert_eq!(res, "6.00 GiB");

        // Percent offsets apply to the metric total, not to `used`.
        let res = render_template("{used + 2%}", &ctx);
        assert_eq!(res, "3.16 GiB");

        let res = render_template("{pct + 5%}", &ctx);
        assert_eq!(res, "42.5%");

        // The value inside a `..` range is random, hence bounds only.
        let res = render_template("{used + 2..4 GiB}", &ctx);
        assert!(res.ends_with(" GiB"));
        let val: f64 = res.trim_end_matches(" GiB").parse().unwrap();
        assert!((5.0..=7.0).contains(&val));
    }

    /// `with_metric` counts KiB (what the modules feed) while `set_bytes` keeps raw bytes.
    const GIB: u64 = 1024 * 1024 * 1024;
    const KIB: u64 = 1024;

    fn mem_ctx() -> Context {
        let mut ctx = Context::new("memory").with_metric(4 * GIB / KIB, 8 * GIB / KIB, 50.0);
        ctx.set_bytes("used", 4 * GIB);
        ctx.set_bytes("total", 8 * GIB);
        ctx.set_bytes("free", 4 * GIB);
        ctx.set_num("pct", 50.0);
        ctx
    }

    #[test]
    fn literal_text_is_passed_through_untouched() {
        let ctx = mem_ctx();
        assert_eq!(render_template("", &ctx), "");
        assert_eq!(render_template("no tags here", &ctx), "no tags here");
        assert_eq!(render_template("100% done", &ctx), "100% done");
    }

    #[test]
    fn unclosed_and_unknown_tags_are_preserved_verbatim() {
        let ctx = mem_ctx();
        // Truncated tag: the rest of the line has to survive.
        assert_eq!(render_template("{used", &ctx), "{used");
        assert_eq!(render_template("a {b c", &ctx), "a {b c");
        // Unknown names stay visible instead of vanishing.
        assert_eq!(render_template("{nope}", &ctx), "{nope}");
    }

    #[test]
    fn doubled_braces_are_escaped() {
        let ctx = mem_ctx();
        assert_eq!(render_template("{{literal}}", &ctx), "{literal}");
    }

    #[test]
    fn unit_converters_cover_the_full_scale() {
        let ctx = mem_ctx();
        assert_eq!(render_template("{raw(used)}", &ctx), "4294967296");
        assert_eq!(render_template("{kib(used)}", &ctx), "4194304 KiB");
        assert_eq!(render_template("{mib(used)}", &ctx), "4096.0 MiB");
        assert_eq!(render_template("{gib(used)}", &ctx), "4.00 GiB");
        assert_eq!(render_template("{human(used)}", &ctx), "4.00 GiB");
    }

    #[test]
    fn colon_and_function_syntax_agree() {
        let ctx = mem_ctx();
        // Both call styles, same raw value into the converter.
        for f in ["gib", "mib", "kib", "raw", "human"] {
            assert_eq!(
                render_template(&format!("{{used:{f}}}"), &ctx),
                render_template(&format!("{{{f}(used)}}"), &ctx),
                "mismatch for {f}"
            );
        }
        assert_eq!(render_template("{used:gib}", &ctx), "4.00 GiB");
        assert_eq!(render_template("{used:raw}", &ctx), "4294967296");
    }

    #[test]
    fn colon_syntax_supports_arbitrary_filters_and_chains() {
        let mut ctx = Context::new("cpu");
        ctx.set_str("name", "Intel(R) Core(TM) i5-3360M");
        assert_eq!(
            render_template("{name:remove('Intel(R) ')}", &ctx),
            "Core(TM) i5-3360M"
        );
        // trunc(n) yields at most n characters, counting the ellipsis itself.
        assert_eq!(
            render_template("{name:remove('Intel(R) '):upper:trunc(8)}", &ctx),
            "CORE(TM…"
        );
    }

    #[test]
    fn bar_honours_explicit_width_and_characters() {
        let ctx = mem_ctx();
        assert_eq!(render_template("{bar(4, '#', '.')}", &ctx), "[##..]");
        assert_eq!(render_template("{bar(8, '#', '.')}", &ctx), "[####....]");
    }

    #[test]
    fn bar_tracks_the_metric_ratio() {
        let full = Context::new("memory").with_metric(4, 4, 100.0);
        assert_eq!(render_template("{bar(4, '#', '.')}", &full), "[####]");
        let empty = Context::new("memory").with_metric(0, 4, 0.0);
        assert_eq!(render_template("{bar(4, '#', '.')}", &empty), "[....]");
        let half = Context::new("memory").with_metric(2, 4, 50.0);
        assert_eq!(render_template("{bar(4, '#', '.')}", &half), "[##..]");
    }

    #[test]
    fn string_functions_cover_the_documented_set() {
        let mut ctx = Context::new("cpu");
        ctx.set_str("name", "intel core tm i5");
        assert_eq!(render_template("{upper(name)}", &ctx), "INTEL CORE TM I5");
        assert_eq!(render_template("{lower(name)}", &ctx), "intel core tm i5");
        assert_eq!(render_template("{title(name)}", &ctx), "Intel Core Tm I5");
        assert_eq!(render_template("{trunc(name, 6)}", &ctx), "intel…");
        assert_eq!(render_template("{trim('  x  ')}", &ctx), "x");
        assert_eq!(render_template("{pad_left('7', 3)}", &ctx), "  7");
        assert_eq!(render_template("{pad_right('7', 3)}", &ctx), "7  ");
    }

    #[test]
    fn replace_and_remove_operate_on_substrings() {
        let mut ctx = Context::new("cpu");
        ctx.set_str("name", "Intel(R) Core(TM) i5");
        assert_eq!(
            render_template("{replace(name, 'Intel', 'AMD')}", &ctx),
            "AMD(R) Core(TM) i5"
        );
        assert_eq!(
            render_template("{remove(name, '(TM)')}", &ctx),
            "Intel(R) Core i5"
        );
        assert_eq!(
            render_template("{remove(name, '(ZZ)')}", &ctx),
            "Intel(R) Core(TM) i5"
        );
    }

    #[test]
    fn trunc_never_splits_a_multibyte_char() {
        let mut ctx = Context::new("quote");
        ctx.set_str("q", "привет мир");
        let out = render_template("{trunc(q, 6)}", &ctx);
        assert!(out.ends_with('…'));
        assert!(out.chars().count() <= 7, "got {out:?}");
    }

    #[test]
    fn percentage_math_rounds_as_documented() {
        let ctx = mem_ctx();
        assert_eq!(render_template("{pct:round(1)}", &ctx), "50.0");
        assert_eq!(render_template("{pct:round(0)}", &ctx), "50");
    }

    #[test]
    fn arithmetic_on_used_updates_bar_and_pct() {
        let out = render_template("{used + 2 GiB} | {pct} | {bar(8, '#', '.')}", &mem_ctx());
        let parts: Vec<&str> = out.split(" | ").collect();
        assert_eq!(parts[0], "6.00 GiB");
        assert_eq!(parts[1], "75");
        assert_eq!(parts[2], "[######..]");
    }

    #[test]
    fn arithmetic_on_pct_moves_the_bar_but_not_the_used_variable() {
        // Percent math updates the metric behind the bar, leaving `used` as recorded.
        let out = render_template("{pct + 10%} | {bar(8, '#', '.')}", &mem_ctx());
        let parts: Vec<&str> = out.split(" | ").collect();
        assert_eq!(parts[0], "60%");
        assert_eq!(parts[1], "[#####...]");
    }

    #[test]
    fn arithmetic_state_does_not_leak_between_renders() {
        // Arithmetic mutates in place. Each module builds its own Context.
        let ctx = mem_ctx();
        assert_eq!(render_template("{used + 2 GiB}", &ctx), "6.00 GiB");
        assert_eq!(render_template("{used + 2 GiB}", &ctx), "6.00 GiB");
        assert_eq!(render_template("{used + 2 GiB}", &mem_ctx()), "6.00 GiB");
    }

    #[test]
    fn random_ranges_stay_inside_their_bounds() {
        for _ in 0..20 {
            let out = render_template("{used + 2..4 GiB}", &mem_ctx());
            let v: f64 = out.trim_end_matches(" GiB").parse().unwrap();
            assert!((6.0..=8.0).contains(&v), "out of range: {v}");
        }
    }

    #[test]
    fn default_substitutes_for_empty_values_only() {
        let mut ctx = Context::new("cpu");
        ctx.set_str("freq", "");
        assert_eq!(render_template("{freq:default('N/A')}", &ctx), "N/A");
        ctx.set_str("freq", "2592 MHz");
        assert_eq!(render_template("{freq:default('N/A')}", &ctx), "2592 MHz");
    }

    #[test]
    fn date_formatting_uses_strftime_syntax() {
        let ctx = mem_ctx();
        let year = render_template("{date('%Y')}", &ctx);
        assert_eq!(year.len(), 4, "expected a 4-digit year, got {year:?}");
        assert!(year.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn ansi_color_tags_only_render_when_color_is_enabled() {
        let ctx = mem_ctx();
        // Colors default on, so the escape has to be there.
        assert!(render_template("{red}warn{reset}", &ctx).contains('\x1b'));
        // An unknown word is not a color and must survive as text.
        assert_eq!(
            render_template("{definitely_not_a_color}", &ctx),
            "{definitely_not_a_color}"
        );
    }
}
