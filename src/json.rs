use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            JsonValue::Number(n) if *n >= 0.0 && n.fract() == 0.0 => Some(*n as u64),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&Vec<(String, JsonValue)>> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn to_string_pretty(&self) -> String {
        let mut buf = String::new();
        format_pretty(self, 0, &mut buf);
        buf
    }

    pub fn to_compact_string(&self) -> String {
        let mut buf = String::new();
        format_compact(self, &mut buf);
        buf
    }
}

fn escape_str(s: &str, buf: &mut String) {
    buf.push('"');
    for ch in s.chars() {
        match ch {
            '"' => buf.push_str("\\\""),
            '\\' => buf.push_str("\\\\"),
            '\n' => buf.push_str("\\n"),
            '\r' => buf.push_str("\\r"),
            '\t' => buf.push_str("\\t"),
            '\u{0008}' => buf.push_str("\\b"),
            '\u{000c}' => buf.push_str("\\f"),
            c if c.is_control() => {
                buf.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => buf.push(c),
        }
    }
    buf.push('"');
}

fn format_compact(val: &JsonValue, buf: &mut String) {
    match val {
        JsonValue::Null => buf.push_str("null"),
        JsonValue::Bool(b) => buf.push_str(if *b { "true" } else { "false" }),
        JsonValue::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                buf.push_str(&format!("{:.0}", n));
            } else {
                buf.push_str(&format!("{}", n));
            }
        }
        JsonValue::String(s) => escape_str(s, buf),
        JsonValue::Array(items) => {
            buf.push('[');
            for (idx, item) in items.iter().enumerate() {
                if idx > 0 {
                    buf.push(',');
                }
                format_compact(item, buf);
            }
            buf.push(']');
        }
        JsonValue::Object(entries) => {
            buf.push('{');
            for (idx, (k, v)) in entries.iter().enumerate() {
                if idx > 0 {
                    buf.push(',');
                }
                escape_str(k, buf);
                buf.push(':');
                format_compact(v, buf);
            }
            buf.push('}');
        }
    }
}

fn indent(level: usize, buf: &mut String) {
    for _ in 0..level {
        buf.push_str("  ");
    }
}

fn format_pretty(val: &JsonValue, level: usize, buf: &mut String) {
    match val {
        JsonValue::Null => buf.push_str("null"),
        JsonValue::Bool(b) => buf.push_str(if *b { "true" } else { "false" }),
        JsonValue::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                buf.push_str(&format!("{:.0}", n));
            } else {
                buf.push_str(&format!("{}", n));
            }
        }
        JsonValue::String(s) => escape_str(s, buf),
        JsonValue::Array(items) => {
            if items.is_empty() {
                buf.push_str("[]");
                return;
            }
            buf.push_str("[\n");
            for (idx, item) in items.iter().enumerate() {
                indent(level + 1, buf);
                format_pretty(item, level + 1, buf);
                if idx + 1 < items.len() {
                    buf.push(',');
                }
                buf.push('\n');
            }
            indent(level, buf);
            buf.push(']');
        }
        JsonValue::Object(entries) => {
            if entries.is_empty() {
                buf.push_str("{}");
                return;
            }
            buf.push_str("{\n");
            for (idx, (k, v)) in entries.iter().enumerate() {
                indent(level + 1, buf);
                escape_str(k, buf);
                buf.push_str(": ");
                format_pretty(v, level + 1, buf);
                if idx + 1 < entries.len() {
                    buf.push(',');
                }
                buf.push('\n');
            }
            indent(level, buf);
            buf.push('}');
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}:{}: {}", self.line, self.col, self.message)
    }
}

impl std::error::Error for JsonError {}

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

    fn err(&self, msg: impl Into<String>) -> JsonError {
        JsonError {
            line: self.line,
            col: self.col,
            message: msg.into(),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n' {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, JsonError> {
        self.skip_whitespace();
        let ch = self
            .peek()
            .ok_or_else(|| self.err("unexpected end of JSON input"))?;

        match ch {
            '"' => self.parse_string().map(JsonValue::String),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            't' | 'f' => self.parse_bool(),
            'n' => self.parse_null(),
            '-' | '0'..='9' => self.parse_number(),
            other => Err(self.err(format!("unexpected character in JSON: {other:?}"))),
        }
    }

    fn parse_string(&mut self) -> Result<String, JsonError> {
        self.next_char();
        let mut s = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.next_char();
                return Ok(s);
            }
            if ch == '\\' {
                self.next_char();
                let escaped = self
                    .peek()
                    .ok_or_else(|| self.err("unterminated escape in JSON string"))?;
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
                                    .ok_or_else(|| self.err("unterminated \\u escape"))?,
                            );
                        }
                        let cp = u32::from_str_radix(&hex, 16)
                            .map_err(|_| self.err(format!("invalid hex in \\u: {hex}")))?;
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
        Err(self.err("unterminated JSON string"))
    }

    fn parse_array(&mut self) -> Result<JsonValue, JsonError> {
        self.next_char();
        let mut items = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek() == Some(']') {
                self.next_char();
                break;
            }
            if self.peek().is_none() {
                return Err(self.err("unterminated array in JSON"));
            }
            let val = self.parse_value()?;
            items.push(val);
            self.skip_whitespace();
            if self.peek() == Some(',') {
                self.next_char();
            } else if self.peek() == Some(']') {
                self.next_char();
                break;
            } else {
                return Err(self.err("expected ',' or ']' in JSON array"));
            }
        }
        Ok(JsonValue::Array(items))
    }

    fn parse_object(&mut self) -> Result<JsonValue, JsonError> {
        self.next_char();
        let mut entries = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek() == Some('}') {
                self.next_char();
                break;
            }
            if self.peek().is_none() {
                return Err(self.err("unterminated object in JSON"));
            }
            if self.peek() != Some('"') {
                return Err(self.err("expected string key in JSON object"));
            }
            let key = self.parse_string()?;
            self.skip_whitespace();
            if self.peek() != Some(':') {
                return Err(self.err("expected ':' after key in JSON object"));
            }
            self.next_char();
            let val = self.parse_value()?;
            entries.push((key, val));
            self.skip_whitespace();
            if self.peek() == Some(',') {
                self.next_char();
            } else if self.peek() == Some('}') {
                self.next_char();
                break;
            } else {
                return Err(self.err("expected ',' or '}' in JSON object"));
            }
        }
        Ok(JsonValue::Object(entries))
    }

    fn parse_bool(&mut self) -> Result<JsonValue, JsonError> {
        if self.consume_exact("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume_exact("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err(self.err("invalid boolean in JSON"))
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, JsonError> {
        if self.consume_exact("null") {
            Ok(JsonValue::Null)
        } else {
            Err(self.err("invalid null in JSON"))
        }
    }

    fn consume_exact(&mut self, expected: &str) -> bool {
        for ch in expected.chars() {
            if self.peek() == Some(ch) {
                self.next_char();
            } else {
                return false;
            }
        }
        true
    }

    fn parse_number(&mut self) -> Result<JsonValue, JsonError> {
        let mut raw = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '-' || ch == '+' || ch == '.' || ch == 'e' || ch == 'E'
            {
                raw.push(ch);
                self.next_char();
            } else {
                break;
            }
        }
        let n: f64 = raw
            .parse()
            .map_err(|_| self.err(format!("invalid JSON number: {raw}")))?;
        Ok(JsonValue::Number(n))
    }
}

pub fn parse(input: &str) -> Result<JsonValue, JsonError> {
    let mut parser = Parser::new(input);
    let val = parser.parse_value()?;
    parser.skip_whitespace();
    if parser.peek().is_some() {
        return Err(parser.err("unexpected trailing characters after JSON value"));
    }
    Ok(val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_format_primitives() {
        assert_eq!(parse("null").unwrap(), JsonValue::Null);
        assert_eq!(parse("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(parse("false").unwrap(), JsonValue::Bool(false));
        assert_eq!(parse("123").unwrap(), JsonValue::Number(123.0));
        assert_eq!(parse("-45.5").unwrap(), JsonValue::Number(-45.5));
        assert_eq!(
            parse("\"hello\\nworld\"").unwrap(),
            JsonValue::String("hello\nworld".into())
        );
    }

    #[test]
    fn parse_and_format_composite() {
        let input = r#"{"name":"omnifetch","versions":[1,2,3],"active":true}"#;
        let val = parse(input).unwrap();
        assert_eq!(
            val.get("name").and_then(JsonValue::as_str),
            Some("omnifetch")
        );
        assert_eq!(val.get("active").and_then(JsonValue::as_bool), Some(true));
        let arr = val.get("versions").and_then(JsonValue::as_array).unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_u64(), Some(1));
    }

    #[test]
    fn pretty_formatter_produces_valid_json() {
        let entries = vec![
            ("key".into(), JsonValue::String("value".into())),
            ("num".into(), JsonValue::Number(42.0)),
        ];
        let val = JsonValue::Object(entries);
        let pretty = val.to_string_pretty();
        let parsed = parse(&pretty).unwrap();
        assert_eq!(val, parsed);
    }

    #[test]
    fn parse_syntax_errors() {
        assert!(parse("").is_err());
        assert!(parse("{").is_err());
        assert!(parse("[1, 2,").is_err());
        assert!(parse(r#"{"key": unquoted}"#).is_err());
        assert!(parse(r#""unclosed string"#).is_err());
    }

    #[test]
    fn parse_escapes_and_scientific_numbers() {
        let input =
            r#"{"esc": "line\r\n\t\"quote\"\\", "sci": 1e3, "empty_arr": [], "empty_obj": {}}"#;
        let val = parse(input).unwrap();
        assert_eq!(
            val.get("esc").and_then(JsonValue::as_str),
            Some("line\r\n\t\"quote\"\\")
        );
        assert_eq!(val.get("sci").and_then(JsonValue::as_f64), Some(1000.0));
        assert!(
            val.get("empty_arr")
                .and_then(JsonValue::as_array)
                .unwrap()
                .is_empty()
        );
        assert!(
            val.get("empty_obj")
                .and_then(JsonValue::as_object)
                .unwrap()
                .is_empty()
        );
    }
}
