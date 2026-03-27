use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub stream: String,
    pub line: String,
    pub timestamp: String,
}

pub fn sanitize_log_line(line: &str) -> String {
    let mut next = line.to_string();
    let patterns = [
        (
            Regex::new(r#"(?i)(api[_-]?key["'\s:=]+)([^\s"',}]+)"#).expect("valid api_key regex"),
            "$1***REDACTED***",
        ),
        (
            Regex::new(r#"(?i)(master[_-]?key["'\s:=]+)([^\s"',}]+)"#)
                .expect("valid master_key regex"),
            "$1***REDACTED***",
        ),
        (
            Regex::new(r#"(?i)(authorization["'\s:=]+bearer\s+)([^\s"',}]+)"#)
                .expect("valid authorization regex"),
            "$1***REDACTED***",
        ),
    ];

    for (pattern, replacement) in patterns {
        next = pattern.replace_all(&next, replacement).into_owned();
    }

    next
}

#[cfg(test)]
mod tests {
    use super::sanitize_log_line;

    #[test]
    fn redacts_known_secret_patterns() {
        let raw = r#"api_key=sk-test master_key:root Authorization: Bearer secret-token"#;
        let sanitized = sanitize_log_line(raw);

        assert!(sanitized.contains("***REDACTED***"));
        assert!(!sanitized.contains("sk-test"));
        assert!(!sanitized.contains("secret-token"));
    }
}
