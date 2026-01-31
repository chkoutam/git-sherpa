use glob_match::glob_match;

const DEFAULT_PATTERNS: &[&str] = &[
    ".env",
    ".env.*",
    "*.pem",
    "*.key",
    "**/id_rsa",
    "**/id_rsa.pub",
    "**/credentials.json",
    "**/*.p12",
    "**/*.pfx",
];

pub fn default_patterns() -> Vec<String> {
    DEFAULT_PATTERNS.iter().map(|s| s.to_string()).collect()
}

pub fn check_sensitive_files(staged: &[String], patterns: &[String]) -> Vec<String> {
    staged
        .iter()
        .filter(|file| patterns.iter().any(|pat| glob_match(pat, file)))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_env_files() {
        let staged = vec![".env".into(), ".env.local".into(), "src/main.rs".into()];
        let patterns = default_patterns();
        let found = check_sensitive_files(&staged, &patterns);
        assert_eq!(found, vec![".env", ".env.local"]);
    }

    #[test]
    fn detects_key_files() {
        let staged = vec!["server.pem".into(), "key.key".into(), "readme.md".into()];
        let patterns = default_patterns();
        let found = check_sensitive_files(&staged, &patterns);
        assert_eq!(found, vec!["server.pem", "key.key"]);
    }

    #[test]
    fn no_false_positives() {
        let staged = vec!["src/main.rs".into(), "Cargo.toml".into()];
        let patterns = default_patterns();
        let found = check_sensitive_files(&staged, &patterns);
        assert!(found.is_empty());
    }
}
