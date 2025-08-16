use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub llm: LlmConfig,
    pub analysis: AnalysisConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub api_key: Option<String>,
    pub model: String,
    #[serde(default = "default_provider")]
    pub provider: String,
    pub base_url: Option<String>,
    pub timeout: u64,
}

fn default_provider() -> String {
    "none".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub custom_rules: Vec<String>,
    pub ambiguity_threshold: f32,
    pub enable_interactive: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            llm: LlmConfig {
                api_key: std::env::var("PRISM_API_KEY").ok(),
                model: "".to_string(),
                provider: "none".to_string(),
                base_url: None,
                timeout: 30,
            },
            analysis: AnalysisConfig {
                custom_rules: vec![],
                ambiguity_threshold: 0.7,
                enable_interactive: true,
            },
        }
    }
}

impl Config {
    pub fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".prism").join("config.yml"))
    }

    pub async fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).await?;
            let mut config: Config = serde_yaml::from_str(&content)?;
            
            // Handle legacy configs that might not have provider field
            if config.llm.provider == "none" && config.llm.api_key.is_some() {
                // Try to detect provider based on existing configuration
                if config.llm.model.contains("gemini") {
                    config.set_provider("gemini");
                } else if config.llm.model.contains("gpt") {
                    config.set_provider("openai");
                } else if config.llm.base_url.as_ref().map_or(false, |url| url.contains("azure")) {
                    config.set_provider("azure");
                }
                // Save the updated config
                config.save().await?;
            }
            
            Ok(config)
        } else {
            let config = Config::default();
            config.save().await?;
            Ok(config)
        }
    }

    pub async fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        let content = serde_yaml::to_string(self)?;
        fs::write(&config_path, content).await?;
        
        Ok(())
    }

    pub fn set_api_key(&mut self, api_key: String) {
        self.llm.api_key = Some(api_key);
    }

    pub fn set_model(&mut self, model: String) {
        self.llm.model = model;
    }

    pub fn set_provider(&mut self, provider: &str) {
        self.llm.provider = provider.to_string();
        
        // Set default base URLs and models based on provider
        match provider {
            "openai" => {
                self.llm.base_url = Some("https://api.openai.com/v1/chat/completions".to_string());
                if self.llm.model.is_empty() {
                    self.llm.model = "gpt-4".to_string();
                }
            }
            "gemini" => {
                self.llm.base_url = Some("https://generativelanguage.googleapis.com/v1beta/models".to_string());
                if self.llm.model.is_empty() {
                    self.llm.model = "gemini-1.5-pro".to_string();
                }
            }
            "azure" => {
                // Azure requires custom base URL to be set by user
                if self.llm.model.is_empty() {
                    self.llm.model = "gpt-4".to_string();
                }
            }
            "claude" => {
                self.llm.base_url = Some("https://api.anthropic.com/v1/messages".to_string());
                if self.llm.model.is_empty() {
                    self.llm.model = "claude-3-sonnet-20240229".to_string();
                }
            }
            "ollama" => {
                self.llm.base_url = Some("http://localhost:11434/api/generate".to_string());
                if self.llm.model.is_empty() {
                    // Try to get the first available model dynamically
                    match Self::get_ollama_models() {
                        Ok(models) if !models.is_empty() => {
                            self.llm.model = models[0].clone();
                        }
                        _ => {
                            // Fallback to common default
                            self.llm.model = "llama3.1:latest".to_string();
                        }
                    }
                }
            }
            _ => {
                self.llm.base_url = None;
            }
        }
    }

    pub fn is_ai_configured(&self) -> bool {
        self.llm.api_key.is_some() && 
        !self.llm.model.is_empty() && 
        self.llm.provider != "none"
    }

    pub fn get_provider_info(&self) -> (String, Vec<String>) {
        match self.llm.provider.as_str() {
            "openai" => ("OpenAI".to_string(), vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string(), "gpt-4o".to_string()]),
            "gemini" => ("Google Gemini".to_string(), vec!["gemini-1.5-pro".to_string(), "gemini-1.5-flash".to_string()]),
            "azure" => ("Azure OpenAI".to_string(), vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()]),
            "claude" => ("Anthropic Claude".to_string(), vec!["claude-3-opus-20240229".to_string(), "claude-3-sonnet-20240229".to_string(), "claude-3-haiku-20240307".to_string()]),
            "ollama" => {
                // Try to get actual available models, fallback to defaults
                match Self::get_ollama_models() {
                    Ok(models) if !models.is_empty() => ("Local Ollama".to_string(), models),
                    _ => ("Local Ollama".to_string(), vec!["llama3.1:latest".to_string(), "llama3.1:8b".to_string(), "gemma2:latest".to_string(), "phi3:mini".to_string(), "qwen2.5-coder:latest".to_string()])
                }
            },
            _ => ("None".to_string(), vec![])
        }
    }

    pub fn get_ollama_models() -> anyhow::Result<Vec<String>> {
        use std::process::Command;
        
        // First try using ollama CLI
        if let Ok(output) = Command::new("ollama").args(&["list"]).output() {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let models: Vec<String> = output_str
                    .lines()
                    .skip(1) // Skip header
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if !parts.is_empty() && !parts[0].is_empty() {
                            Some(parts[0].to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                
                if !models.is_empty() {
                    return Ok(models);
                }
            }
        }

        // Fallback: try HTTP API
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let client = reqwest::Client::new();
            match client.get("http://localhost:11434/api/tags").send().await {
                Ok(response) if response.status().is_success() => {
                    match response.json::<serde_json::Value>().await {
                        Ok(json) => {
                            if let Some(models_array) = json.get("models").and_then(|m| m.as_array()) {
                                let models: Vec<String> = models_array
                                    .iter()
                                    .filter_map(|model| {
                                        model.get("name").and_then(|name| name.as_str()).map(|s| s.to_string())
                                    })
                                    .collect();
                                
                                if !models.is_empty() {
                                    return Ok(models);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            
            Err(anyhow::anyhow!("Could not fetch Ollama models"))
        })
    }

    pub async fn validate_all_settings(&self) -> Result<ValidationResult> {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        
        // Validate API key
        if let Some(ref api_key) = self.llm.api_key {
            if api_key.is_empty() {
                issues.push("API key is empty".to_string());
            } else if api_key.len() < 10 {
                warnings.push("API key seems too short".to_string());
            }
        } else if self.llm.provider != "ollama" && self.llm.provider != "none" {
            issues.push("API key is required for the selected provider".to_string());
        }
        
        // Validate provider
        match self.llm.provider.as_str() {
            "openai" => {
                if self.llm.model.is_empty() {
                    issues.push("Model name is required for OpenAI".to_string());
                }
                if let Some(ref api_key) = self.llm.api_key {
                    if !api_key.starts_with("sk-") {
                        warnings.push("OpenAI API keys typically start with 'sk-'".to_string());
                    }
                }
            }
            "gemini" => {
                if self.llm.model.is_empty() {
                    issues.push("Model name is required for Gemini".to_string());
                }
            }
            "claude" => {
                if self.llm.model.is_empty() {
                    issues.push("Model name is required for Claude".to_string());
                }
            }
            "azure" => {
                if self.llm.base_url.is_none() {
                    issues.push("Base URL is required for Azure OpenAI".to_string());
                }
            }
            "ollama" => {
                // Check if Ollama is available
                match Self::get_ollama_models() {
                    Ok(models) => {
                        if models.is_empty() {
                            warnings.push("No Ollama models found. Run 'ollama pull <model>' to install models".to_string());
                        } else if !models.contains(&self.llm.model) {
                            warnings.push(format!("Model '{}' not found in Ollama. Available models: {}", 
                                self.llm.model, models.join(", ")));
                        }
                    }
                    Err(_) => {
                        issues.push("Ollama server is not available. Run 'ollama serve' to start it".to_string());
                    }
                }
            }
            "none" => {
                warnings.push("AI features are disabled. Configure a provider to enable AI-powered analysis".to_string());
            }
            _ => {
                issues.push(format!("Unknown provider: {}", self.llm.provider));
            }
        }
        
        // Validate timeout
        if self.llm.timeout == 0 {
            warnings.push("Timeout is set to 0, which may cause immediate timeouts".to_string());
        } else if self.llm.timeout > 300 {
            warnings.push("Timeout is very high (>5 minutes), consider reducing it".to_string());
        }
        
        // Validate analysis settings
        if self.analysis.ambiguity_threshold < 0.0 || self.analysis.ambiguity_threshold > 1.0 {
            issues.push("Ambiguity threshold must be between 0.0 and 1.0".to_string());
        }
        
        Ok(ValidationResult {
            is_valid: issues.is_empty(),
            issues,
            warnings,
        })
    }

    pub async fn test_all_providers(&self) -> Result<ProviderTestResults> {
        let mut results = ProviderTestResults::new();
        
        let providers = vec!["openai", "gemini", "claude", "azure", "ollama"];
        
        for provider in providers {
            let test_result = self.test_provider(provider).await;
            results.add_result(provider.to_string(), test_result);
        }
        
        Ok(results)
    }

    async fn test_provider(&self, provider: &str) -> ProviderTestResult {
        let mut test_config = self.clone();
        test_config.set_provider(provider);
        
        // Skip test if no API key is configured for non-Ollama providers
        if provider != "ollama" && test_config.llm.api_key.is_none() {
            return ProviderTestResult {
                success: false,
                message: "No API key configured".to_string(),
                response_time: None,
            };
        }
        
        let start_time = std::time::Instant::now();
        
        match provider {
            "openai" => {
                // Test OpenAI connection
                if let Some(ref api_key) = test_config.llm.api_key {
                    let client = reqwest::Client::new();
                    let response = client
                        .get("https://api.openai.com/v1/models")
                        .header("Authorization", format!("Bearer {}", api_key))
                        .send()
                        .await;
                        
                    match response {
                        Ok(resp) if resp.status().is_success() => {
                            ProviderTestResult {
                                success: true,
                                message: "OpenAI connection successful".to_string(),
                                response_time: Some(start_time.elapsed().as_millis()),
                            }
                        }
                        Ok(resp) => {
                            ProviderTestResult {
                                success: false,
                                message: format!("OpenAI API error: {}", resp.status()),
                                response_time: Some(start_time.elapsed().as_millis()),
                            }
                        }
                        Err(e) => {
                            ProviderTestResult {
                                success: false,
                                message: format!("OpenAI connection failed: {}", e),
                                response_time: None,
                            }
                        }
                    }
                } else {
                    ProviderTestResult {
                        success: false,
                        message: "No API key configured".to_string(),
                        response_time: None,
                    }
                }
            }
            "ollama" => {
                // Test Ollama connection
                let client = reqwest::Client::new();
                let response = client
                    .get("http://localhost:11434/api/tags")
                    .send()
                    .await;
                    
                match response {
                    Ok(resp) if resp.status().is_success() => {
                        ProviderTestResult {
                            success: true,
                            message: "Ollama connection successful".to_string(),
                            response_time: Some(start_time.elapsed().as_millis()),
                        }
                    }
                    Ok(_) => {
                        ProviderTestResult {
                            success: false,
                            message: "Ollama server responded with error".to_string(),
                            response_time: Some(start_time.elapsed().as_millis()),
                        }
                    }
                    Err(_) => {
                        ProviderTestResult {
                            success: false,
                            message: "Ollama server not available. Run 'ollama serve'".to_string(),
                            response_time: None,
                        }
                    }
                }
            }
            _ => {
                // For other providers, just check basic configuration
                ProviderTestResult {
                    success: test_config.llm.api_key.is_some(),
                    message: if test_config.llm.api_key.is_some() { 
                        format!("{} configuration looks valid", provider)
                    } else { 
                        "No API key configured".to_string()
                    },
                    response_time: None,
                }
            }
        }
    }

    pub fn get_template_directory(&self) -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".prism").join("templates"))
    }

    pub async fn set_template_directory(&mut self, template_dir: PathBuf) -> Result<()> {
        // Validate the directory
        if !template_dir.exists() {
            return Err(anyhow::anyhow!("Template directory does not exist: {}", template_dir.display()));
        }
        
        // Create a custom field for template directory (would need to add to struct)
        // For now, we'll save it to a separate config file
        let config_dir = Self::config_path()?.parent().unwrap().to_path_buf();
        let template_config_path = config_dir.join("templates.yml");
        
        let template_config = TemplateConfig {
            template_directory: Some(template_dir),
        };
        
        let content = serde_yaml::to_string(&template_config)?;
        fs::write(template_config_path, content).await?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub template_directory: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub struct ProviderTestResults {
    pub results: std::collections::HashMap<String, ProviderTestResult>,
}

impl ProviderTestResults {
    fn new() -> Self {
        Self {
            results: std::collections::HashMap::new(),
        }
    }
    
    fn add_result(&mut self, provider: String, result: ProviderTestResult) {
        self.results.insert(provider, result);
    }
    
    pub fn get_summary(&self) -> String {
        let total = self.results.len();
        let successful = self.results.values().filter(|r| r.success).count();
        
        format!("Provider Test Results: {}/{} successful", successful, total)
    }
}

#[derive(Debug)]
pub struct ProviderTestResult {
    pub success: bool,
    pub message: String,
    pub response_time: Option<u128>,
}