use std::collections::HashMap;

/// Expands platform tokens in manifest file paths.
///
/// Tokens like `{program_files}` are replaced with actual filesystem paths.
/// Unknown tokens are left as-is.
pub struct PathResolver {
    tokens: HashMap<&'static str, String>,
}

impl PathResolver {
    pub fn new() -> Self {
        let mut tokens = HashMap::new();
        Self::add_platform_tokens(&mut tokens);
        Self { tokens }
    }

    /// Expand all tokens in a template string.
    /// Returns `None` if any token resolves to a platform-unavailable path.
    pub fn expand(&self, template: &str) -> Option<String> {
        let mut result = template.to_string();
        for (token, value) in &self.tokens {
            let placeholder = format!("{{{token}}}");
            if result.contains(placeholder.as_str()) {
                result = result.replace(placeholder.as_str(), value);
            }
        }
        // If any {token} patterns remain, they couldn't be resolved
        if result.contains('{') && result.contains('}') {
            return None;
        }
        Some(result)
    }

    #[cfg(windows)]
    fn add_platform_tokens(tokens: &mut HashMap<&'static str, String>) {
        if let Ok(val) = std::env::var("ProgramFiles") {
            tokens.insert("program_files", val.clone());
            tokens.insert("program_dir", val); // alias used in manifests
        }
        if let Ok(val) = std::env::var("ProgramFiles(x86)") {
            tokens.insert("program_files_x86", val);
        }
        if let Ok(val) = std::env::var("APPDATA") {
            tokens.insert("app_data", val);
        }
        if let Ok(val) = std::env::var("LOCALAPPDATA") {
            tokens.insert("local_app_data", val.clone());
            tokens.insert("config_dir", val); // alias used in manifests
        }
        if let Ok(val) = std::env::var("ProgramData") {
            tokens.insert("common_app_data", val);
        }
        if let Ok(val) = std::env::var("USERPROFILE") {
            tokens.insert("user_home", val);
        }
    }

    #[cfg(not(windows))]
    fn add_platform_tokens(tokens: &mut HashMap<&'static str, String>) {
        use directories::BaseDirs;

        if let Some(dirs) = BaseDirs::new() {
            tokens.insert("user_home", dirs.home_dir().to_string_lossy().into_owned());
            tokens.insert("app_data", dirs.config_dir().to_string_lossy().into_owned());
            tokens.insert(
                "local_app_data",
                dirs.data_local_dir().to_string_lossy().into_owned(),
            );
        }
        // Windows-only tokens unavailable on other platforms — left unresolved
    }

    /// Reverse of expand: convert an absolute path back to a token template.
    ///
    /// Replaces known directory prefixes with `{token}` placeholders.
    /// Prefers more specific tokens (program_files_x86 before program_files).
    pub fn tokenize(&self, path: &str) -> String {
        let mut result = path.to_string();
        // Sort by value length descending so longer (more specific) paths match first
        let mut entries: Vec<_> = self.tokens.iter().collect();
        entries.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        for (token, value) in entries {
            // Case-insensitive replacement for Windows paths
            let lower_result = result.to_lowercase();
            let lower_value = value.to_lowercase();
            if let Some(pos) = lower_result.find(&lower_value) {
                let placeholder = format!("{{{token}}}");
                result = format!(
                    "{}{}{}",
                    &result[..pos],
                    placeholder,
                    &result[pos + value.len()..]
                );
                break; // Only replace the first (most specific) match
            }
        }
        result
    }
}

impl Default for PathResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn expand_no_tokens() {
        let resolver = PathResolver::new();
        assert_eq!(
            resolver.expand("C:\\some\\path\\file.exe"),
            Some("C:\\some\\path\\file.exe".into())
        );
    }

    #[test]
    fn expand_user_home() {
        let resolver = PathResolver::new();
        let result = resolver.expand("{user_home}/test");
        assert!(result.is_some());
        let expanded = result.unwrap();
        assert!(!expanded.contains("{user_home}"));
        assert!(expanded.ends_with("/test") || expanded.ends_with("\\test"));
    }

    #[test]
    fn expand_unknown_token_returns_none() {
        let resolver = PathResolver::new();
        assert_eq!(resolver.expand("{nonexistent_token}/file.exe"), None);
    }

    #[test]
    fn expand_mixed_known_unknown() {
        let resolver = PathResolver::new();
        // If one token resolves but another doesn't, return None
        let result = resolver.expand("{user_home}/{bogus}/file");
        assert_eq!(result, None);
    }

    #[cfg(not(windows))]
    #[test]
    fn windows_only_tokens_unavailable() {
        let resolver = PathResolver::new();
        // program_files is Windows-only
        assert_eq!(resolver.expand("{program_files}/NINA/NINA.exe"), None);
    }
}
