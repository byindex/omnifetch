use std::collections::HashMap;
use std::fmt;
use std::ops::Index;

#[derive(Debug, Clone, PartialEq)]
pub enum TomlValue {
    String(String),
    Ident(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<TomlValue>),
    Table(HashMap<String, TomlValue>),
}

impl TomlValue {
    pub fn get(&self, key: &str) -> Option<&TomlValue> {
        match self {
            TomlValue::Table(map) => map.get(key),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut TomlValue> {
        match self {
            TomlValue::Table(map) => map.get_mut(key),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            TomlValue::String(s) | TomlValue::Ident(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_ident(&self) -> Option<&str> {
        match self {
            TomlValue::Ident(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_quoted_str(&self) -> Option<&str> {
        match self {
            TomlValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn is_ident(&self) -> bool {
        matches!(self, TomlValue::Ident(_))
    }

    pub fn is_quoted_str(&self) -> bool {
        matches!(self, TomlValue::String(_))
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            TomlValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            TomlValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            TomlValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<TomlValue>> {
        match self {
            TomlValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_table(&self) -> Option<&HashMap<String, TomlValue>> {
        match self {
            TomlValue::Table(t) => Some(t),
            _ => None,
        }
    }
}

impl Index<&str> for TomlValue {
    type Output = TomlValue;

    fn index(&self, index: &str) -> &Self::Output {
        match self.get(index) {
            Some(v) => v,
            None => panic!("no entry found for key {index:?}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TomlError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl fmt::Display for TomlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}:{}: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for TomlError {}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn err(&self, msg: impl Into<String>) -> TomlError {
        TomlError {
            line: self.line,
            col: self.col,
            message: msg.into(),
        }
    }

    fn skip_horizontal_ws(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        if self.peek() == Some('#') {
            while let Some(ch) = self.peek() {
                if ch == '\n' {
                    break;
                }
                self.next_char();
            }
        }
    }

    fn skip_ws_and_comments_in_array(&mut self) {
        loop {
            match self.peek() {
                Some(' ' | '\t' | '\r' | '\n') => {
                    self.next_char();
                }
                Some('#') => {
                    self.skip_comment();
                }
                _ => break,
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, TomlError> {
        let quote = self.peek().ok_or_else(|| self.err("unexpected EOF"))?;
        if quote != '"' && quote != '\'' {
            return Err(self.err(format!("expected string, found {quote:?}")));
        }

        if self.peek_ahead(1) == Some(quote) && self.peek_ahead(2) == Some(quote) {
            self.next_char();
            self.next_char();
            self.next_char();
            return self.parse_multiline_string(quote);
        }

        self.next_char();
        if quote == '\'' {
            let mut s = String::new();
            while let Some(ch) = self.peek() {
                if ch == '\'' {
                    self.next_char();
                    return Ok(s);
                }
                if ch == '\n' {
                    return Err(self.err("newline in single-line literal string"));
                }
                s.push(ch);
                self.next_char();
            }
            Err(self.err("unterminated literal string"))
        } else {
            let mut s = String::new();
            while let Some(ch) = self.peek() {
                if ch == '"' {
                    self.next_char();
                    return Ok(s);
                }
                if ch == '\n' {
                    return Err(self.err("newline in single-line string"));
                }
                if ch == '\\' {
                    self.next_char();
                    let escaped = self.peek().ok_or_else(|| self.err("unterminated escape"))?;
                    self.next_char();
                    match escaped {
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        '/' => s.push('/'),
                        'b' => s.push('\u{0008}'),
                        'f' => s.push('\u{000c}'),
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        'u' => {
                            let mut hex = String::with_capacity(4);
                            for _ in 0..4 {
                                let h = self
                                    .next_char()
                                    .ok_or_else(|| self.err("unterminated \\u escape"))?;
                                hex.push(h);
                            }
                            let cp = u32::from_str_radix(&hex, 16)
                                .map_err(|_| self.err(format!("invalid hex in \\u: {hex}")))?;
                            let c = char::from_u32(cp)
                                .ok_or_else(|| self.err(format!("invalid unicode scalar: {cp}")))?;
                            s.push(c);
                        }
                        'U' => {
                            let mut hex = String::with_capacity(8);
                            for _ in 0..8 {
                                let h = self
                                    .next_char()
                                    .ok_or_else(|| self.err("unterminated \\U escape"))?;
                                hex.push(h);
                            }
                            let cp = u32::from_str_radix(&hex, 16)
                                .map_err(|_| self.err(format!("invalid hex in \\U: {hex}")))?;
                            let c = char::from_u32(cp)
                                .ok_or_else(|| self.err(format!("invalid unicode scalar: {cp}")))?;
                            s.push(c);
                        }
                        other => {
                            return Err(self.err(format!("unknown escape sequence: \\{other}")));
                        }
                    }
                } else {
                    s.push(ch);
                    self.next_char();
                }
            }
            Err(self.err("unterminated string"))
        }
    }

    fn parse_multiline_string(&mut self, quote: char) -> Result<String, TomlError> {
        if self.peek() == Some('\r') && self.peek_ahead(1) == Some('\n') {
            self.next_char();
            self.next_char();
        } else if self.peek() == Some('\n') {
            self.next_char();
        }

        let mut s = String::new();
        while let Some(ch) = self.peek() {
            if ch == quote && self.peek_ahead(1) == Some(quote) && self.peek_ahead(2) == Some(quote)
            {
                self.next_char();
                self.next_char();
                self.next_char();
                return Ok(s);
            }
            if quote == '"' && ch == '\\' {
                self.next_char();
                if self.peek() == Some('\n')
                    || (self.peek() == Some('\r') && self.peek_ahead(1) == Some('\n'))
                {
                    if self.peek() == Some('\r') {
                        self.next_char();
                    }
                    self.next_char();
                    while let Some(w) = self.peek() {
                        if w == ' ' || w == '\t' || w == '\r' || w == '\n' {
                            self.next_char();
                        } else {
                            break;
                        }
                    }
                    continue;
                }
                let escaped = self.peek().ok_or_else(|| self.err("unterminated escape"))?;
                self.next_char();
                match escaped {
                    '"' => s.push('"'),
                    '\\' => s.push('\\'),
                    '/' => s.push('/'),
                    'b' => s.push('\u{0008}'),
                    'f' => s.push('\u{000c}'),
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    'u' => {
                        let mut hex = String::with_capacity(4);
                        for _ in 0..4 {
                            hex.push(
                                self.next_char()
                                    .ok_or_else(|| self.err("unterminated \\u"))?,
                            );
                        }
                        let cp = u32::from_str_radix(&hex, 16)
                            .map_err(|_| self.err(format!("invalid hex: {hex}")))?;
                        let c = char::from_u32(cp).ok_or_else(|| self.err("invalid unicode"))?;
                        s.push(c);
                    }
                    other => return Err(self.err(format!("unknown escape: \\{other}"))),
                }
            } else {
                s.push(ch);
                self.next_char();
            }
        }
        Err(self.err("unterminated multiline string"))
    }

    fn parse_single_key(&mut self) -> Result<String, TomlError> {
        self.skip_horizontal_ws();
        match self.peek() {
            Some('"' | '\'') => self.parse_string(),
            Some(ch) if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' => {
                let mut key = String::new();
                while let Some(c) = self.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                        key.push(c);
                        self.next_char();
                    } else {
                        break;
                    }
                }
                Ok(key)
            }
            Some(ch) => Err(self.err(format!("unexpected character in key: {ch:?}"))),
            None => Err(self.err("unexpected EOF in key")),
        }
    }

    fn parse_key_path(&mut self) -> Result<Vec<String>, TomlError> {
        let mut path = Vec::new();
        loop {
            self.skip_horizontal_ws();
            let key = self.parse_single_key()?;
            path.push(key);
            self.skip_horizontal_ws();
            if self.peek() == Some('.') {
                self.next_char();
            } else {
                break;
            }
        }
        Ok(path)
    }

    fn parse_value(&mut self) -> Result<TomlValue, TomlError> {
        self.skip_horizontal_ws();
        let ch = self
            .peek()
            .ok_or_else(|| self.err("unexpected EOF while parsing value"))?;

        if ch == '"' || ch == '\'' {
            return self.parse_string().map(TomlValue::String);
        }

        if ch == '[' {
            return self.parse_array();
        }

        if ch == '{' {
            return self.parse_inline_table();
        }

        let mut raw = String::new();
        while let Some(c) = self.peek() {
            if c == ' '
                || c == '\t'
                || c == '\r'
                || c == '\n'
                || c == ','
                || c == ']'
                || c == '}'
                || c == '#'
            {
                break;
            }
            raw.push(c);
            self.next_char();
        }

        if raw.is_empty() {
            return Err(self.err("expected value"));
        }

        if raw == "true" {
            return Ok(TomlValue::Boolean(true));
        }
        if raw == "false" {
            return Ok(TomlValue::Boolean(false));
        }

        let sanitized = raw.replace('_', "");
        if (sanitized.starts_with("0x")
            || sanitized.starts_with("+0x")
            || sanitized.starts_with("-0x"))
            && sanitized.len() > 2
        {
            let (neg, hex) = if sanitized.starts_with('-') {
                (true, &sanitized[3..])
            } else if sanitized.starts_with('+') {
                (false, &sanitized[3..])
            } else {
                (false, &sanitized[2..])
            };
            if let Ok(val) = i64::from_str_radix(hex, 16) {
                return Ok(TomlValue::Integer(if neg { -val } else { val }));
            }
        }

        if (sanitized.starts_with("0o")
            || sanitized.starts_with("+0o")
            || sanitized.starts_with("-0o"))
            && sanitized.len() > 2
        {
            let (neg, oct) = if sanitized.starts_with('-') {
                (true, &sanitized[3..])
            } else if sanitized.starts_with('+') {
                (false, &sanitized[3..])
            } else {
                (false, &sanitized[2..])
            };
            if let Ok(val) = i64::from_str_radix(oct, 8) {
                return Ok(TomlValue::Integer(if neg { -val } else { val }));
            }
        }

        if (sanitized.starts_with("0b")
            || sanitized.starts_with("+0b")
            || sanitized.starts_with("-0b"))
            && sanitized.len() > 2
        {
            let (neg, bin) = if sanitized.starts_with('-') {
                (true, &sanitized[3..])
            } else if sanitized.starts_with('+') {
                (false, &sanitized[3..])
            } else {
                (false, &sanitized[2..])
            };
            if let Ok(val) = i64::from_str_radix(bin, 2) {
                return Ok(TomlValue::Integer(if neg { -val } else { val }));
            }
        }

        if let Ok(i) = sanitized.parse::<i64>() {
            return Ok(TomlValue::Integer(i));
        }

        if let Ok(f) = sanitized.parse::<f64>() {
            return Ok(TomlValue::Float(f));
        }

        if raw
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            return Ok(TomlValue::Ident(raw));
        }

        Err(self.err(format!("invalid value: {raw:?}")))
    }

    fn parse_array(&mut self) -> Result<TomlValue, TomlError> {
        self.next_char();
        let mut items = Vec::new();
        loop {
            self.skip_ws_and_comments_in_array();
            if self.peek() == Some(']') {
                self.next_char();
                break;
            }
            if self.peek().is_none() {
                return Err(self.err("unterminated array"));
            }
            let val = self.parse_value()?;
            items.push(val);
            self.skip_ws_and_comments_in_array();
            if self.peek() == Some(',') {
                self.next_char();
            } else if self.peek() == Some(']') {
                self.next_char();
                break;
            } else {
                return Err(self.err("expected ',' or ']' in array"));
            }
        }
        Ok(TomlValue::Array(items))
    }

    fn parse_inline_table(&mut self) -> Result<TomlValue, TomlError> {
        self.next_char();
        let mut map = HashMap::new();
        loop {
            self.skip_horizontal_ws();
            if self.peek() == Some('}') {
                self.next_char();
                break;
            }
            if self.peek().is_none() {
                return Err(self.err("unterminated inline table"));
            }
            let key = self.parse_single_key()?;
            self.skip_horizontal_ws();
            if self.peek() != Some('=') {
                return Err(self.err("expected '=' after key in inline table"));
            }
            self.next_char();
            self.skip_horizontal_ws();
            let val = self.parse_value()?;
            map.insert(key, val);
            self.skip_horizontal_ws();
            if self.peek() == Some(',') {
                self.next_char();
            } else if self.peek() == Some('}') {
                self.next_char();
                break;
            } else {
                return Err(self.err("expected ',' or '}' in inline table"));
            }
        }
        Ok(TomlValue::Table(map))
    }
}

fn ensure_table_exists(current: &mut HashMap<String, TomlValue>, path: &[String]) {
    if let Some((first, rest)) = path.split_first() {
        let entry = current
            .entry(first.clone())
            .or_insert_with(|| TomlValue::Table(HashMap::new()));
        let nested = match entry {
            TomlValue::Table(nested) => nested,
            _ => {
                *entry = TomlValue::Table(HashMap::new());
                match entry {
                    TomlValue::Table(nested) => nested,
                    _ => unreachable!(),
                }
            }
        };
        ensure_table_exists(nested, rest);
    }
}

fn insert_recursive(
    current: &mut HashMap<String, TomlValue>,
    path: &[String],
    val: TomlValue,
) -> Result<(), String> {
    if path.is_empty() {
        return Err("empty key path".to_string());
    }
    if path.len() == 1 {
        current.insert(path[0].clone(), val);
        return Ok(());
    }
    let (first, rest) = path.split_first().unwrap();
    let entry = current
        .entry(first.clone())
        .or_insert_with(|| TomlValue::Table(HashMap::new()));
    match entry {
        TomlValue::Table(nested) => insert_recursive(nested, rest, val),
        _ => Err(format!("cannot insert key into non-table {first:?}")),
    }
}

fn insert_value(
    root: &mut HashMap<String, TomlValue>,
    section: &[String],
    key_path: &[String],
    val: TomlValue,
    parser: &Parser,
) -> Result<(), TomlError> {
    let mut full_path = Vec::with_capacity(section.len() + key_path.len());
    full_path.extend_from_slice(section);
    full_path.extend_from_slice(key_path);

    insert_recursive(root, &full_path, val).map_err(|msg| parser.err(msg))
}

pub fn parse(input: &str) -> Result<TomlValue, TomlError> {
    let mut parser = Parser::new(input);
    let mut root = HashMap::new();
    let mut current_section: Vec<String> = Vec::new();

    while parser.pos < parser.chars.len() {
        parser.skip_horizontal_ws();
        match parser.peek() {
            None => break,
            Some('#') => {
                parser.skip_comment();
            }
            Some('\r') => {
                parser.next_char();
                if parser.peek() == Some('\n') {
                    parser.next_char();
                }
            }
            Some('\n') => {
                parser.next_char();
            }
            Some('[') => {
                parser.next_char();
                parser.skip_horizontal_ws();
                let section_path = parser.parse_key_path()?;
                parser.skip_horizontal_ws();
                if parser.peek() != Some(']') {
                    return Err(parser.err("expected ']' after table name"));
                }
                parser.next_char();
                parser.skip_horizontal_ws();
                parser.skip_comment();
                if let Some(ch) = parser.peek()
                    && ch != '\r'
                    && ch != '\n'
                {
                    return Err(parser.err("expected newline after table header"));
                }
                ensure_table_exists(&mut root, &section_path);
                current_section = section_path;
            }
            _ => {
                let key_path = parser.parse_key_path()?;
                parser.skip_horizontal_ws();
                if parser.peek() != Some('=') {
                    return Err(parser.err("expected '=' after key"));
                }
                parser.next_char();
                parser.skip_horizontal_ws();
                let val = parser.parse_value()?;
                parser.skip_horizontal_ws();
                parser.skip_comment();
                if let Some(ch) = parser.peek()
                    && ch != '\r'
                    && ch != '\n'
                {
                    return Err(parser.err("unexpected characters after value"));
                }

                insert_value(&mut root, &current_section, &key_path, val, &parser)?;
            }
        }
    }

    Ok(TomlValue::Table(root))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_key_value() {
        let input = r#"
            title = "omnifetch"
            count = 42
            ratio = 3.14
            flag = true
            disabled = false
        "#;
        let val = parse(input).unwrap();
        assert_eq!(
            val.get("title").and_then(TomlValue::as_str),
            Some("omnifetch")
        );
        assert_eq!(val.get("count").and_then(TomlValue::as_integer), Some(42));
        assert_eq!(val.get("flag").and_then(TomlValue::as_bool), Some(true));
        assert_eq!(
            val.get("disabled").and_then(TomlValue::as_bool),
            Some(false)
        );
    }

    #[test]
    fn parse_strings_with_escapes_and_unicode() {
        let input = r##"
            fill = "█"
            empty = "░"
            escaped = "hello\nworld\t\"quoted\""
            literal = 'plain\ntext'
            color = "#ff007f"
        "##;
        let val = parse(input).unwrap();
        assert_eq!(val.get("fill").and_then(TomlValue::as_str), Some("█"));
        assert_eq!(val.get("empty").and_then(TomlValue::as_str), Some("░"));
        assert_eq!(
            val.get("escaped").and_then(TomlValue::as_str),
            Some("hello\nworld\t\"quoted\"")
        );
        assert_eq!(
            val.get("literal").and_then(TomlValue::as_str),
            Some("plain\\ntext")
        );
        assert_eq!(
            val.get("color").and_then(TomlValue::as_str),
            Some("#ff007f")
        );
    }

    #[test]
    fn parse_multiline_arrays_with_comments() {
        let input = r#"
            modules = [
                # System
                "os",
                "kernel", # inline comment

                # Hardware
                "cpu",
                "memory",
            ]
        "#;
        let val = parse(input).unwrap();
        let arr = val.get("modules").and_then(TomlValue::as_array).unwrap();
        let strs: Vec<&str> = arr.iter().filter_map(TomlValue::as_str).collect();
        assert_eq!(strs, vec!["os", "kernel", "cpu", "memory"]);
    }

    #[test]
    fn parse_tables_and_subtables() {
        let input = r##"
            root_key = "root"

            [bar]
            width = 16
            fill = "#"

            [keys]
            # empty table preserved

            [nested.section]
            val = 99
        "##;
        let val = parse(input).unwrap();
        assert_eq!(
            val.get("root_key").and_then(TomlValue::as_str),
            Some("root")
        );
        assert_eq!(
            val["bar"].get("width").and_then(TomlValue::as_integer),
            Some(16)
        );
        assert!(val.get("keys").and_then(TomlValue::as_table).is_some());
        assert_eq!(
            val["nested"]["section"]
                .get("val")
                .and_then(TomlValue::as_integer),
            Some(99)
        );
    }

    #[test]
    fn parse_errors_on_malformed_syntax() {
        assert!(parse("this is not = valid = toml").is_err());
        assert!(parse("unclosed = \"string").is_err());
        assert!(parse("arr = [1, 2").is_err());
    }

    #[test]
    fn parse_default_template() {
        let val = parse(crate::config::DEFAULT_CONFIG_TEMPLATE).unwrap();
        assert_eq!(val.get("logo").and_then(TomlValue::as_str), Some("auto"));
        let mods = val.get("modules").and_then(TomlValue::as_array).unwrap();
        assert!(!mods.is_empty());
        assert_eq!(
            val["bar"].get("width").and_then(TomlValue::as_integer),
            Some(20)
        );
        assert_eq!(
            val["bar"].get("fill").and_then(TomlValue::as_str),
            Some("█")
        );
        assert_eq!(
            val["bar"].get("empty").and_then(TomlValue::as_str),
            Some("░")
        );
        assert_eq!(
            val["network"].get("weather_ip").and_then(TomlValue::as_str),
            Some("5.9.243.187")
        );
        assert_eq!(
            val["network"]
                .get("weather_host")
                .and_then(TomlValue::as_str),
            Some("wttr.in")
        );
    }
    #[test]
    fn parse_negative_numbers_and_floats() {
        let input = r#"
            neg = -42
            pos = +100
            zero = 0
            float_neg = -3.14
            float_pos = 12.34
        "#;
        let val = parse(input).unwrap();
        assert_eq!(val.get("neg").and_then(TomlValue::as_integer), Some(-42));
        assert_eq!(val.get("pos").and_then(TomlValue::as_integer), Some(100));
        assert_eq!(val.get("zero").and_then(TomlValue::as_integer), Some(0));
        assert_eq!(
            val.get("float_neg").and_then(TomlValue::as_float),
            Some(-3.14)
        );
        assert_eq!(
            val.get("float_pos").and_then(TomlValue::as_float),
            Some(12.34)
        );
    }

    #[test]
    fn parse_inline_tables_and_nested_arrays() {
        let input = r#"
            inline = { a = 1, b = "two", c = true }
            nested_arr = [[1, 2], [3, 4]]
            empty_arr = []
        "#;
        let val = parse(input).unwrap();
        let inline = val.get("inline").and_then(TomlValue::as_table).unwrap();
        assert_eq!(inline.get("a").and_then(TomlValue::as_integer), Some(1));
        assert_eq!(inline.get("b").and_then(TomlValue::as_str), Some("two"));
        assert_eq!(inline.get("c").and_then(TomlValue::as_bool), Some(true));

        let arr = val.get("nested_arr").and_then(TomlValue::as_array).unwrap();
        assert_eq!(arr.len(), 2);
        let empty = val.get("empty_arr").and_then(TomlValue::as_array).unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn parse_multiline_strings_and_escapes() {
        let input = r#"
            escaped = "line1\nline2\r\n\t\\\""
            raw = 'no\escapes\here'
        "#;
        let val = parse(input).unwrap();
        assert_eq!(
            val.get("escaped").and_then(TomlValue::as_str),
            Some("line1\nline2\r\n\t\\\"")
        );
        assert_eq!(
            val.get("raw").and_then(TomlValue::as_str),
            Some("no\\escapes\\here")
        );
    }

    #[test]
    fn parse_unquoted_identifiers_and_arrays() {
        let input = r#"
            theme = dracula
            logo = auto
            preset = modern
            modules = [ os, kernel, uptime, break, colors ]
            quoted_theme = "Adwaita-Dark"
        "#;
        let val = parse(input).unwrap();
        assert_eq!(
            val.get("theme").and_then(TomlValue::as_ident),
            Some("dracula")
        );
        assert!(val.get("theme").unwrap().is_ident());
        assert!(!val.get("theme").unwrap().is_quoted_str());

        assert_eq!(
            val.get("quoted_theme").and_then(TomlValue::as_quoted_str),
            Some("Adwaita-Dark")
        );
        assert!(val.get("quoted_theme").unwrap().is_quoted_str());
        assert!(!val.get("quoted_theme").unwrap().is_ident());

        let mods = val.get("modules").and_then(TomlValue::as_array).unwrap();
        let mod_names: Vec<&str> = mods.iter().filter_map(TomlValue::as_str).collect();
        assert_eq!(mod_names, vec!["os", "kernel", "uptime", "break", "colors"]);
    }
}
