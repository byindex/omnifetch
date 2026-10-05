use crate::config;
use crate::module::{Module, ModuleOutput};
use crate::template::{self, Context};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Quote;

pub const BUILTIN_QUOTES_JSON: &str = include_str!("../../assets/quotes.json");

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteEntry {
    pub quote: String,
    pub author: Option<String>,
    pub chance: Option<ChanceValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChanceValue {
    Num(f64),
    Str(String),
}

impl ChanceValue {
    pub fn to_percentage(&self) -> Option<f64> {
        match self {
            ChanceValue::Num(n) => {
                if *n > 0.0 {
                    Some(*n)
                } else {
                    None
                }
            }
            ChanceValue::Str(s) => {
                let clean = s.trim().trim_end_matches('%').trim();
                clean.parse::<f64>().ok().filter(|&v| v > 0.0)
            }
        }
    }
}

impl Module for Quote {
    fn name(&self) -> &'static str {
        "Quote"
    }
    fn id(&self) -> &'static str {
        "quote"
    }
    fn run(&self) -> Option<ModuleOutput> {
        let (quote, author) = select_quote();

        let cfg = config::get();
        let tmpl = cfg
            .format
            .get("quote")
            .map(String::as_str)
            .unwrap_or("\"{quote}\" — {author}");

        let mut ctx = Context::new("quote");
        ctx.set_str("quote", &quote);
        ctx.set_str("q", &quote);
        ctx.set_str("author", &author);
        ctx.set_str("a", &author);

        let formatted = template::render_template(tmpl, &ctx);
        Some(ModuleOutput::new("Quote", formatted))
    }
}

pub fn select_quote() -> (String, String) {
    let quotes = load_quotes();
    if let Some((q, a)) = select_from_quotes(&quotes) {
        (q, a)
    } else {
        (
            "Talk is cheap. Show me the code.".to_string(),
            "Linus Torvalds".to_string(),
        )
    }
}

fn load_quotes() -> Vec<QuoteEntry> {
    let cfg = config::get();

    // 1. Explicit path from CLI flag `--quotes-file` or config `quotes_file`
    if let Some(ref p) = cfg.quotes_file {
        let resolved = resolve_path(p);
        if let Ok(quotes) = load_quotes_from_file(&resolved)
            && !quotes.is_empty()
        {
            return quotes;
        }
    }

    // 2. Standard config directory paths
    if let Some(quotes) = load_user_quotes()
        && !quotes.is_empty()
    {
        return quotes;
    }

    // 3. Built-in JSON, compiled into the binary
    get_builtin_quotes()
}

static BUILTIN_CACHE: OnceLock<Vec<QuoteEntry>> = OnceLock::new();

pub fn get_builtin_quotes() -> Vec<QuoteEntry> {
    BUILTIN_CACHE
        .get_or_init(|| parse_quotes_json(BUILTIN_QUOTES_JSON).unwrap_or_default())
        .clone()
}

fn load_user_quotes() -> Option<Vec<QuoteEntry>> {
    let mut candidates = Vec::new();

    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME")
        && !xdg.is_empty()
    {
        let xdg_path = PathBuf::from(xdg).join("omnifetch");
        candidates.push(xdg_path.join("quotes.json"));
        candidates.push(xdg_path.join("quotes.txt"));
        candidates.push(xdg_path.join("quotes"));
    }

    if let Ok(home) = std::env::var("HOME")
        && !home.is_empty()
    {
        let config_path = PathBuf::from(home).join(".config/omnifetch");
        candidates.push(config_path.join("quotes.json"));
        candidates.push(config_path.join("quotes.txt"));
        candidates.push(config_path.join("quotes"));
    }

    for path in candidates {
        if path.is_file()
            && let Ok(quotes) = load_quotes_from_file(&path)
            && !quotes.is_empty()
        {
            return Some(quotes);
        }
    }
    None
}

pub fn load_quotes_from_file(path: &Path) -> Result<Vec<QuoteEntry>, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let is_json = path.extension().and_then(|s| s.to_str()) == Some("json")
        || content.trim_start().starts_with('[')
        || content.trim_start().starts_with('{');

    if is_json && let Some(list) = parse_quotes_json(&content) {
        return Ok(list);
    }

    let list = parse_quotes_text(&content);
    if !list.is_empty() {
        return Ok(list);
    }

    Err("no quotes found in file".to_string())
}

pub fn parse_quotes_json(content: &str) -> Option<Vec<QuoteEntry>> {
    let val = crate::json::parse(content).ok()?;
    if let Some(arr) = val.as_array() {
        let entries: Vec<QuoteEntry> = arr.iter().filter_map(parse_single_quote).collect();
        if !entries.is_empty() {
            return Some(entries);
        }
    }
    if let Some(quotes_arr) = val.get("quotes").and_then(crate::json::JsonValue::as_array) {
        let entries: Vec<QuoteEntry> = quotes_arr.iter().filter_map(parse_single_quote).collect();
        if !entries.is_empty() {
            return Some(entries);
        }
    }
    None
}

fn parse_single_quote(item: &crate::json::JsonValue) -> Option<QuoteEntry> {
    if let Some(s) = item.as_str() {
        return Some(QuoteEntry {
            quote: s.to_string(),
            author: None,
            chance: None,
        });
    }

    let quote = item
        .get("quote")
        .or_else(|| item.get("q"))
        .or_else(|| item.get("text"))
        .or_else(|| item.get("content"))
        .or_else(|| item.get("цитата"))
        .and_then(crate::json::JsonValue::as_str)?;

    let author = item
        .get("author")
        .or_else(|| item.get("a"))
        .or_else(|| item.get("by"))
        .or_else(|| item.get("author_name"))
        .or_else(|| item.get("автор"))
        .and_then(crate::json::JsonValue::as_str)
        .map(ToString::to_string);

    let chance = item
        .get("chance")
        .or_else(|| item.get("weight"))
        .or_else(|| item.get("prob"))
        .or_else(|| item.get("probability"))
        .or_else(|| item.get("шанс"))
        .and_then(|v| match v {
            crate::json::JsonValue::Number(n) => Some(ChanceValue::Num(*n)),
            crate::json::JsonValue::String(s) => Some(ChanceValue::Str(s.clone())),
            _ => None,
        });

    Some(QuoteEntry {
        quote: quote.to_string(),
        author,
        chance,
    })
}

fn parse_quotes_text(content: &str) -> Vec<QuoteEntry> {
    let mut list = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((q, a)) = trimmed.split_once("|||") {
            list.push(QuoteEntry {
                quote: q.trim().to_string(),
                author: Some(a.trim().to_string()),
                chance: None,
            });
        } else if let Some((q, a)) = trimmed.split_once(" — ") {
            list.push(QuoteEntry {
                quote: q.trim().trim_matches('"').to_string(),
                author: Some(a.trim().to_string()),
                chance: None,
            });
        } else if let Some((q, a)) = trimmed.split_once(" - ") {
            list.push(QuoteEntry {
                quote: q.trim().trim_matches('"').to_string(),
                author: Some(a.trim().to_string()),
                chance: None,
            });
        } else {
            list.push(QuoteEntry {
                quote: trimmed.to_string(),
                author: Some("Anonymous".to_string()),
                chance: None,
            });
        }
    }
    list
}

pub fn select_from_quotes(quotes: &[QuoteEntry]) -> Option<(String, String)> {
    if quotes.is_empty() {
        return None;
    }
    if quotes.len() == 1 {
        let q = &quotes[0];
        return Some((
            q.quote.clone(),
            q.author.clone().unwrap_or_else(|| "Anonymous".to_string()),
        ));
    }

    // 1. Collect explicit chances and count default quotes
    let mut explicit_sum = 0.0;
    let mut default_count = 0usize;
    let mut chances: Vec<Option<f64>> = Vec::with_capacity(quotes.len());

    for entry in quotes {
        if let Some(ch) = entry.chance.as_ref().and_then(|c| c.to_percentage()) {
            explicit_sum += ch;
            chances.push(Some(ch));
        } else {
            default_count += 1;
            chances.push(None);
        }
    }

    // 2. Assign remaining percentage proportionally to default quotes
    let default_weight = if default_count > 0 {
        if explicit_sum < 100.0 {
            (100.0 - explicit_sum) / (default_count as f64)
        } else {
            0.0
        }
    } else {
        0.0
    };

    let mut weights: Vec<f64> = Vec::with_capacity(quotes.len());
    let mut total_weight = 0.0;

    for ch in chances {
        let w = match ch {
            Some(explicit) => explicit,
            None => default_weight,
        };
        weights.push(w);
        total_weight += w;
    }

    // Weights that sum to zero would divide by zero, so pick uniformly instead
    if total_weight <= 0.0 {
        let idx = (random_f64() * quotes.len() as f64) as usize % quotes.len();
        let q = &quotes[idx];
        return Some((
            q.quote.clone(),
            q.author.clone().unwrap_or_else(|| "Anonymous".to_string()),
        ));
    }

    // 3. Weighted random sampling
    let r = random_f64() * total_weight;
    let mut cumulative = 0.0;
    let mut selected_idx = quotes.len() - 1;

    for (i, &w) in weights.iter().enumerate() {
        cumulative += w;
        if r < cumulative {
            selected_idx = i;
            break;
        }
    }

    let q = &quotes[selected_idx];
    Some((
        q.quote.clone(),
        q.author.clone().unwrap_or_else(|| "Anonymous".to_string()),
    ))
}

fn resolve_path(path: &Path) -> PathBuf {
    let p_str = path.to_string_lossy();
    if p_str.starts_with("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home).join(&p_str[2..]);
    }
    path.to_path_buf()
}

fn random_f64() -> f64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = unsafe { libc::getpid() } as u128;
    let mut state = (nanos ^ (pid << 32) ^ 0x9e3779b97f4a7c15) as u64;
    // splitmix64 PRNG step
    state = state.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^= z >> 31;
    (z >> 11) as f64 / ((1u64 << 53) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_quotes_not_empty() {
        let builtin = get_builtin_quotes();
        assert!(!builtin.is_empty(), "Builtin quotes list must not be empty");

        for (i, entry) in builtin.iter().enumerate() {
            assert!(
                !entry.quote.trim().is_empty(),
                "Quote at index {i} has empty text"
            );
        }

        let (q, a) = select_from_quotes(&builtin).expect("Should pick a quote from builtins");
        assert!(!q.is_empty());
        assert!(!a.is_empty());
    }

    #[test]
    fn test_chance_percentage_parsing() {
        let c1 = ChanceValue::Str("2%".to_string());
        assert_eq!(c1.to_percentage(), Some(2.0));

        let c2 = ChanceValue::Str(" 5.5 % ".to_string());
        assert_eq!(c2.to_percentage(), Some(5.5));

        let c3 = ChanceValue::Num(10.0);
        assert_eq!(c3.to_percentage(), Some(10.0));

        let c4 = ChanceValue::Num(0.0);
        assert_eq!(c4.to_percentage(), None);
    }

    #[test]
    fn test_json_deserialization_formats() {
        let json_str = r#"[
            { "quote": "Quote one", "author": "Author A", "chance": "2%" },
            { "text": "Quote two", "by": "Author B", "weight": 5.0 },
            { "цитата": "Quote three", "автор": "Author C", "шанс": "10%" },
            "Just a simple string quote"
        ]"#;

        let parsed = parse_quotes_json(json_str).expect("should parse");
        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed[0].quote, "Quote one");
        assert_eq!(parsed[0].author.as_deref(), Some("Author A"));
        assert_eq!(parsed[0].chance, Some(ChanceValue::Str("2%".to_string())));

        assert_eq!(parsed[1].quote, "Quote two");
        assert_eq!(parsed[1].author.as_deref(), Some("Author B"));
        assert_eq!(parsed[1].chance, Some(ChanceValue::Num(5.0)));

        assert_eq!(parsed[2].quote, "Quote three");
        assert_eq!(parsed[2].author.as_deref(), Some("Author C"));

        assert_eq!(parsed[3].quote, "Just a simple string quote");
        assert_eq!(parsed[3].author, None);
    }

    #[test]
    fn test_weighted_selection_100_percent() {
        let quotes = vec![
            QuoteEntry {
                quote: "Rare Guaranteed".to_string(),
                author: Some("Tester".to_string()),
                chance: Some(ChanceValue::Str("100%".to_string())),
            },
            QuoteEntry {
                quote: "Never chosen".to_string(),
                author: Some("Tester".to_string()),
                chance: None,
            },
        ];

        for _ in 0..20 {
            let (q, _) = select_from_quotes(&quotes).unwrap();
            assert_eq!(q, "Rare Guaranteed");
        }
    }
}
