use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiResponseContent>,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    parts: Option<Vec<GeminiResponsePart>>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
    text: Option<String>,
}

pub struct AiGenerator {
    api_key: String,
    client: reqwest::Client,
}

impl AiGenerator {
    pub fn new() -> Result<Self> {
        let api_key = env::var("GEMINI_API_KEY").map_err(|_| {
            anyhow!(
                "GEMINI_API_KEY environment variable is not set.\n\
                 Please get an API key from Google AI Studio and set it:\n\
                 export GEMINI_API_KEY=\"your_key_here\""
            )
        })?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(45))
            .build()
            .unwrap_or_default();

        Ok(Self { api_key, client })
    }

    pub async fn generate_skill(&self, name: &str, custom_prompt: Option<&str>) -> Result<String> {
        let default_prompt = format!(
            "You are an expert software engineer and AI coding assistant. \
             Generate a high-quality SKILL.md file for the technology/package: '{}'.\n\n\
             This skill file will be placed in the project directory under onpkg_docs/ so that AI agents (like Claude/Gemini) \
             working on the codebase can immediately read it to gain all the necessary context, rules, best practices, and patterns.\n\n\
             Requirements:\n\
             1. It MUST start with YAML frontmatter containing: name, description, and metadata.version.\n\
             2. It MUST be written in clean GitHub-style Markdown.\n\
             3. It MUST contain sections like: Core Rules & Guidelines, Typical Commands, Common Patterns, and Troubleshooting.\n\
             4. Make the guidelines highly detailed and specific to '{}' (e.g. state management, routing, async behavior, config layout).\n\
             5. Do NOT wrap your response in markdown code blocks like ```markdown...```. Return the raw markdown string directly.",
            name, name
        );

        let prompt = match custom_prompt {
            Some(p) => format!("{}\n\nAdditional user guidelines:\n{}", default_prompt, p),
            None => default_prompt,
        };

        self.call_gemini(&prompt).await
    }

    pub async fn generate_template(&self, name: &str, description: &str) -> Result<String> {
        let prompt = format!(
            "You are a master software architect. Generate a template definition TOML configuration for a template/stack named: '{}'.\n\
             Description: '{}'\n\n\
             The TOML configuration MUST strictly follow the schema below:\n\n\
             name = \"{}\"\n\
             category = \"(frontend|backend|website|app|fullstack|custom)\"\n\
             description = \"Description of what this template sets up\"\n\
             version = \"1.0.0\"\n\n\
             [[variables]]\n\
             name = \"project_name\"\n\
             description = \"Name of the project\"\n\
             default = \"my-app\"\n\n\
             [[files]]\n\
             path = \"package.json\"  # or other manifest/source files\n\
             content = \"\"\"# File content here...\"\"\"\n\n\
             [[files]]\n\
             path = \"src/main.ts\"\n\
             content = \"\"\"# File content here...\"\"\"\n\n\
             Requirements:\n\
             1. The template MUST include all configuration files (like tsconfig.json, vite.config.ts, requirements.txt, pyproject.toml, pubspec.yaml, Cargo.toml etc.) and standard boilerplate code needed for the stack.\n\
             2. Use standard multiline strings \"\"\"...\"\"\" in TOML for file content.\n\
             3. Return ONLY the raw TOML content. Do NOT wrap it in ```toml...```. Do NOT include any explanations.",
            name, description, name
        );

        self.call_gemini(&prompt).await
    }

    async fn call_gemini(&self, prompt: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            self.api_key
        );

        let req_body = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
        };

        let response = self
            .client
            .post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Gemini API: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Gemini API returned HTTP status {}: {}",
                status,
                error_text
            ));
        }

        let res_body: GeminiResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse JSON response from Gemini API: {}", e))?;

        let text = res_body
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|cand| cand.content)
            .and_then(|cont| cont.parts)
            .and_then(|parts| parts.into_iter().next())
            .and_then(|part| part.text)
            .ok_or_else(|| anyhow!("Gemini API returned an empty or invalid response."))?;

        // Clean up markdown block wraps if the model ignored instructions and wrapped anyway
        let mut cleaned = text.trim();
        if cleaned.starts_with("```toml") {
            cleaned = cleaned.strip_prefix("```toml").unwrap_or(cleaned);
            if cleaned.ends_with("```") {
                cleaned = cleaned.strip_suffix("```").unwrap_or(cleaned);
            }
        } else if cleaned.starts_with("```markdown") {
            cleaned = cleaned.strip_prefix("```markdown").unwrap_or(cleaned);
            if cleaned.ends_with("```") {
                cleaned = cleaned.strip_suffix("```").unwrap_or(cleaned);
            }
        } else if cleaned.starts_with("```") && cleaned.ends_with("```") {
            cleaned = cleaned.strip_prefix("```").unwrap_or(cleaned);
            cleaned = cleaned.strip_suffix("```").unwrap_or(cleaned);
        }

        Ok(cleaned.trim().to_string())
    }
}
