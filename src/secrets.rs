use regex::Regex;

pub struct RedactResult {
    pub content: String,
    pub warnings: Vec<(usize, String)>,
}

pub fn redact_secrets(content: &str) -> RedactResult {
    let mut warnings = Vec::new();
    
    // Compile regexes
    let openai_re = Regex::new(r#"sk-(?:proj-)?[a-zA-Z0-9_]{40,120}"#).unwrap();
    let github_re = Regex::new(r#"ghp_[a-zA-Z0-9]{36}|github_pat_[a-zA-Z0-9_]{82}"#).unwrap();
    let aws_id_re = Regex::new(r#"AKIA[0-9A-Z]{16}"#).unwrap();
    let aws_secret_re = Regex::new(r#"(?i)(aws_secret_access_key\s*[:=]\s*['\ns]?[']?)[a-zA-Z0-9/+=]{40}(['\ns]?[']?)"#).unwrap();
    let stripe_re = Regex::new(r#"sk_live_[0-9a-zA-Z]{24}"#).unwrap();
    let generic_key_re = Regex::new(r#"(?i)(api_key|api_token|client_secret|client_key|token|secret_key|db_password)(\s*[:=]\s*['\"])[a-zA-Z0-9_\-]{32,96}(['\"])"#).unwrap();

    let mut lines = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let mut new_line = line.to_string();

        if openai_re.is_match(&new_line) {
            new_line = openai_re.replace_all(&new_line, "[REDACTED-OPENAI-KEY]").to_string();
            warnings.push((line_num, "OpenAI API Key detected".to_string()));
        }

        if github_re.is_match(&new_line) {
            new_line = github_re.replace_all(&new_line, "[REDACTED-GITHUB-PAT]").to_string();
            warnings.push((line_num, "GitHub PAT detected".to_string()));
        }

        if aws_id_re.is_match(&new_line) {
            new_line = aws_id_re.replace_all(&new_line, "[REDACTED-AWS-ACCESS-KEY-ID]").to_string();
            warnings.push((line_num, "AWS Access Key ID detected".to_string()));
        }

        if aws_secret_re.is_match(&new_line) {
            new_line = aws_secret_re.replace_all(&new_line, "${1}[REDACTED-AWS-SECRET-KEY]${2}").to_string();
            warnings.push((line_num, "AWS Secret Access Key detected".to_string()));
        }

        if stripe_re.is_match(&new_line) {
            new_line = stripe_re.replace_all(&new_line, "[REDACTED-STRIPE-KEY]").to_string();
            warnings.push((line_num, "Stripe API Key detected".to_string()));
        }

        if generic_key_re.is_match(&new_line) {
            new_line = generic_key_re.replace_all(&new_line, "${1}${2}[REDACTED-SECRET]${3}").to_string();
            warnings.push((line_num, "Potential generic credential/token detected".to_string()));
        }

        lines.push(new_line);
    }

    RedactResult {
        content: lines.join("\n"),
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_openai_key() {
        let content = "let key = \"sk-proj-1234567890abcdef1234567890abcdef1234567890abcdef\";";
        let res = redact_secrets(content);
        assert_eq!(res.warnings.len(), 1);
        assert!(res.content.contains("[REDACTED-OPENAI-KEY]"));
    }

    #[test]
    fn test_redact_github_pat() {
        let content = "let token = \"ghp_1234567890abcdef1234567890abcdef1234\";";
        let res = redact_secrets(content);
        assert_eq!(res.warnings.len(), 1);
        assert!(res.content.contains("[REDACTED-GITHUB-PAT]"));
    }

    #[test]
    fn test_redact_generic_key() {
        let content = "db_password = \"my-super-secret-password-1234567890\";";
        let res = redact_secrets(content);
        assert_eq!(res.warnings.len(), 1);
        assert!(res.content.contains("[REDACTED-SECRET]"));
    }
}
