use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub ambiguities: Vec<Ambiguity>,
    pub entities: ExtractedEntities,
    pub uml_diagrams: Option<UmlDiagrams>,
    pub pseudocode: Option<String>,
    pub test_cases: Option<TestCases>,
    pub improved_requirements: Option<String>,
    pub completeness_analysis: Option<CompletenessAnalysis>,
    pub user_story_validation: Option<UserStoryValidation>,
    pub nfr_suggestions: Option<Vec<NonFunctionalRequirement>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ambiguity {
    pub text: String,
    pub reason: String,
    pub suggestions: Vec<String>,
    pub severity: AmbiguitySeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AmbiguitySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for AmbiguitySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AmbiguitySeverity::Critical => write!(f, "Critical"),
            AmbiguitySeverity::High => write!(f, "High"),
            AmbiguitySeverity::Medium => write!(f, "Medium"),
            AmbiguitySeverity::Low => write!(f, "Low"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntities {
    pub actors: Vec<String>,
    pub actions: Vec<String>,
    pub objects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UmlDiagrams {
    pub use_case: Option<String>,
    pub sequence: Option<String>,
    pub class_diagram: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCases {
    pub happy_path: Vec<String>,
    pub negative_cases: Vec<String>,
    pub edge_cases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletenessAnalysis {
    pub missing_actors: Vec<String>,
    pub missing_success_criteria: Vec<String>,
    pub missing_nf_considerations: Vec<String>,
    pub completeness_score: f32,
    pub gaps_identified: Vec<Gap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gap {
    pub category: String,
    pub description: String,
    pub suggestions: Vec<String>,
    pub priority: GapPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GapPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStoryValidation {
    pub is_valid_format: bool,
    pub actor_quality: ValidationResult,
    pub goal_quality: ValidationResult,
    pub reason_quality: ValidationResult,
    pub business_value_score: f32,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub score: f32,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonFunctionalRequirement {
    pub category: NfrCategory,
    pub requirement: String,
    pub rationale: String,
    pub acceptance_criteria: Vec<String>,
    pub priority: NfrPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum NfrCategory {
    Performance,
    Security,
    Usability,
    Reliability,
    Scalability,
    Maintainability,
    Compatibility,
    Accessibility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NfrPriority {
    MustHave,
    ShouldHave,
    CouldHave,
    WontHave,
}

#[derive(Clone)]
pub struct Analyzer {
    vague_terms: Vec<Regex>,
    passive_voice: Regex,
    conditional_incomplete: Regex,
    http_client: Client,
    config: Option<Config>,
}

#[derive(Serialize)]
struct LlmRequest {
    model: String,
    messages: Vec<LlmMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct LlmMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct LlmResponse {
    choices: Vec<LlmChoice>,
}

#[derive(Deserialize)]
struct LlmChoice {
    message: LlmResponseMessage,
}

#[derive(Deserialize)]
struct LlmResponseMessage {
    content: String,
}

impl Analyzer {
    pub fn new() -> Result<Self> {
        let vague_terms = vec![
            Regex::new(r"\b(fast|quick|slow|easy|hard|user-friendly|robust|scalable|efficient)\b")?,
            Regex::new(r"\b(better|worse|good|bad|nice|great|awesome)\b")?,
            Regex::new(r"\b(many|few|some|several|various|multiple)\b")?,
        ];

        let passive_voice = Regex::new(r"\b(should be|will be|must be|needs to be|ought to be)\s+\w+ed\b")?;
        let conditional_incomplete = Regex::new(r"\bif\b.*\bwithout\b.*\belse\b")?;

        Ok(Self {
            vague_terms,
            passive_voice,
            conditional_incomplete,
            http_client: Client::new(),
            config: None,
        })
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    pub async fn analyze(&self, text: &str) -> Result<AnalysisResult> {
        let mut ambiguities = self.detect_ambiguities(text);
        let mut entities = self.extract_entities(text);
        
        if let Some(config) = &self.config {
            if config.llm.api_key.is_some() {
                // println!("🤖 Calling AI for enhanced analysis...");
                
                // Try AI ambiguity detection with error reporting
                match self.detect_ambiguities_with_llm(text).await {
                    Ok(llm_ambiguities) => {
                        // println!("✅ AI found {} additional ambiguities", llm_ambiguities.len());
                        ambiguities.extend(llm_ambiguities);
                    }
                    Err(e) => {
                        eprintln!("⚠️  AI ambiguity detection failed: {}", e);
                        eprintln!("   Continuing with built-in analysis only");
                    }
                }
                
                // Try AI entity extraction with error reporting
                match self.extract_entities_with_llm(text).await {
                    Ok(llm_entities) => {
                        let actors_count = llm_entities.actors.len();
                        let actions_count = llm_entities.actions.len();
                        let objects_count = llm_entities.objects.len();
                        
                        entities.actors.extend(llm_entities.actors);
                        entities.actions.extend(llm_entities.actions);
                        entities.objects.extend(llm_entities.objects);
                        
                        entities.actors.sort();
                        entities.actors.dedup();
                        entities.actions.sort();
                        entities.actions.dedup();
                        entities.objects.sort();
                        entities.objects.dedup();
                        
                        // println!("✅ AI enhanced entities: +{} actors, +{} actions, +{} objects", 
                        //         actors_count, actions_count, objects_count);
                    }
                    Err(e) => {
                        eprintln!("⚠️  AI entity extraction failed: {}", e);
                        eprintln!("   Continuing with built-in analysis only");
                    }
                }
            } else {
                // println!("ℹ️  AI not configured - using built-in analysis only");
            }
        }
        
        Ok(AnalysisResult {
            ambiguities,
            entities,
            uml_diagrams: None,
            pseudocode: None,
            test_cases: None,
            improved_requirements: None,
            completeness_analysis: None,
            user_story_validation: None,
            nfr_suggestions: None,
        })
    }

    async fn detect_ambiguities_with_llm(&self, text: &str) -> Result<Vec<Ambiguity>> {
        let prompt = format!(
            "Analyze the following requirement text for ambiguities, vague terms, and unclear specifications. 
            Look for terms that lack specific criteria, passive voice that hides responsibility, 
            incomplete conditional logic, and any other sources of potential miscommunication.
            
            Requirement text:
            {}
            
            Please provide a JSON response with the following structure:
            {{
                \"ambiguities\": [
                    {{
                        \"text\": \"the ambiguous phrase\",
                        \"reason\": \"why it's ambiguous\",
                        \"suggestions\": [\"suggestion 1\", \"suggestion 2\"],
                        \"severity\": \"High|Medium|Low|Critical\"
                    }}
                ]
            }}",
            text
        );

        let response = self.call_llm(&prompt).await?;
        self.parse_ambiguities_response(&response)
    }

    async fn extract_entities_with_llm(&self, text: &str) -> Result<ExtractedEntities> {
        let prompt = format!(
            "Extract the key entities from the following requirement text. Identify:
            1. Actors (who performs actions - users, administrators, systems, services)
            2. Actions (what is being done - verbs like create, update, delete, login)
            3. Objects (what is being acted upon - nouns like account, profile, data)
            
            Requirement text:
            {}
            
            Please provide a JSON response with the following structure:
            {{
                \"actors\": [\"actor1\", \"actor2\"],
                \"actions\": [\"action1\", \"action2\"],
                \"objects\": [\"object1\", \"object2\"]
            }}",
            text
        );

        let response = self.call_llm(&prompt).await?;
        self.parse_entities_response(&response)
    }

    pub async fn call_llm(&self, prompt: &str) -> Result<String> {
        let config = self.config.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No configuration available"))?;
        
        let api_key = config.llm.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No API key configured"))?;

        match config.llm.provider.as_str() {
            "gemini" => self.call_gemini_api(prompt, api_key, &config.llm.model).await,
            "claude" => self.call_claude_api(prompt, api_key, &config.llm.model).await,
            "ollama" => self.call_ollama_api(prompt, &config.llm.model, config).await,
            "openai" | "azure" | _ => self.call_openai_api(prompt, api_key, config).await,
        }
    }

    async fn call_openai_api(&self, prompt: &str, api_key: &str, config: &crate::config::Config) -> Result<String> {
        let request = LlmRequest {
            model: config.llm.model.clone(),
            messages: vec![
                LlmMessage {
                    role: "system".to_string(),
                    content: "You are an expert software requirements analyst. Provide detailed, accurate analysis in the requested JSON format.".to_string(),
                },
                LlmMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            max_tokens: 2000,
            temperature: 0.1,
        };

        let url = config.llm.base_url.as_deref()
            .unwrap_or("https://api.openai.com/v1/chat/completions");

        let response = self.http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("OpenAI API request failed: {}", error_text));
        }

        let llm_response: LlmResponse = response.json().await?;
        
        llm_response.choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No response from LLM"))
    }

    async fn call_gemini_api(&self, prompt: &str, api_key: &str, model: &str) -> Result<String> {
        #[derive(Serialize)]
        struct GeminiRequest {
            contents: Vec<GeminiContent>,
            #[serde(rename = "generationConfig")]
            generation_config: GeminiGenerationConfig,
        }

        #[derive(Serialize)]
        struct GeminiContent {
            parts: Vec<GeminiPart>,
        }

        #[derive(Serialize)]
        struct GeminiPart {
            text: String,
        }

        #[derive(Serialize)]
        struct GeminiGenerationConfig {
            temperature: f32,
            #[serde(rename = "maxOutputTokens")]
            max_output_tokens: u32,
        }

        #[derive(Deserialize)]
        struct GeminiResponse {
            candidates: Vec<GeminiCandidate>,
        }

        #[derive(Deserialize)]
        struct GeminiCandidate {
            content: GeminiResponseContent,
        }

        #[derive(Deserialize)]
        struct GeminiResponseContent {
            parts: Vec<GeminiResponsePart>,
        }

        #[derive(Deserialize)]
        struct GeminiResponsePart {
            text: String,
        }

        let system_prompt = "You are an expert software requirements analyst. Provide detailed, accurate analysis in the requested JSON format.";
        let full_prompt = format!("{}\n\n{}", system_prompt, prompt);

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: full_prompt,
                }],
            }],
            generation_config: GeminiGenerationConfig {
                temperature: 0.1,
                max_output_tokens: 2000,
            },
        };

        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", model, api_key);

        let response = self.http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Gemini API request failed: {}", error_text));
        }

        let gemini_response: GeminiResponse = response.json().await?;
        
        gemini_response.candidates
            .first()
            .and_then(|candidate| candidate.content.parts.first())
            .map(|part| part.text.clone())
            .ok_or_else(|| anyhow::anyhow!("No response from Gemini"))
    }

    async fn call_claude_api(&self, prompt: &str, api_key: &str, model: &str) -> Result<String> {
        #[derive(Serialize)]
        struct ClaudeRequest {
            model: String,
            max_tokens: u32,
            messages: Vec<ClaudeMessage>,
        }

        #[derive(Serialize)]
        struct ClaudeMessage {
            role: String,
            content: String,
        }

        #[derive(Deserialize)]
        struct ClaudeResponse {
            content: Vec<ClaudeContent>,
        }

        #[derive(Deserialize)]
        struct ClaudeContent {
            text: String,
        }

        let request = ClaudeRequest {
            model: model.to_string(),
            max_tokens: 2000,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: format!("You are an expert software requirements analyst. Provide detailed, accurate analysis in the requested JSON format.\n\n{}", prompt),
            }],
        };

        let response = self.http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Claude API request failed: {}", error_text));
        }

        let claude_response: ClaudeResponse = response.json().await?;
        
        claude_response.content
            .first()
            .map(|content| content.text.clone())
            .ok_or_else(|| anyhow::anyhow!("No response from Claude"))
    }

    async fn call_ollama_api(&self, prompt: &str, model: &str, config: &crate::config::Config) -> Result<String> {
        #[derive(Serialize)]
        struct OllamaRequest {
            model: String,
            prompt: String,
            stream: bool,
        }

        #[derive(Deserialize)]
        struct OllamaResponse {
            response: String,
            done: bool,
        }

        let system_prompt = "You are an expert software requirements analyst. Provide detailed, accurate analysis in the requested JSON format.";
        let full_prompt = format!("{}\n\n{}", system_prompt, prompt);

        let request = OllamaRequest {
            model: model.to_string(),
            prompt: full_prompt,
            stream: false,
        };

        let base_url = config.llm.base_url.as_deref()
            .unwrap_or("http://localhost:11434/api/generate");

        let response = self.http_client
            .post(base_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Ollama API request failed: {}", error_text));
        }

        let ollama_response: OllamaResponse = response.json().await?;
        
        if !ollama_response.done {
            return Err(anyhow::anyhow!("Ollama response not complete"));
        }

        Ok(ollama_response.response)
    }

    fn parse_ambiguities_response(&self, response: &str) -> Result<Vec<Ambiguity>> {
        #[derive(Deserialize)]
        struct AmbiguityResponse {
            ambiguities: Vec<AmbiguityData>,
        }

        #[derive(Deserialize)]
        struct AmbiguityData {
            text: String,
            reason: String,
            suggestions: Vec<String>,
            severity: String,
        }

        // Debug: print raw response (uncomment for debugging)
        // println!("🔍 Raw AI response for ambiguities:");
        // println!("{}", response);
        
        // Try to extract JSON from response if it's wrapped in markdown
        let json_str = if response.contains("```json") {
            response.split("```json").nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response)
                .trim()
        } else if response.contains("```") {
            response.split("```").nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response)
                .trim()
        } else {
            response.trim()
        };

        let parsed: AmbiguityResponse = serde_json::from_str(json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse LLM response for ambiguities: {}. Raw response: {}", e, json_str))?;

        Ok(parsed.ambiguities.into_iter().map(|data| {
            let severity = match data.severity.as_str() {
                "Critical" => AmbiguitySeverity::Critical,
                "High" => AmbiguitySeverity::High,
                "Medium" => AmbiguitySeverity::Medium,
                _ => AmbiguitySeverity::Low,
            };

            Ambiguity {
                text: data.text,
                reason: data.reason,
                suggestions: data.suggestions,
                severity,
            }
        }).collect())
    }

    fn parse_entities_response(&self, response: &str) -> Result<ExtractedEntities> {
        #[derive(Deserialize)]
        struct EntityResponse {
            actors: Vec<String>,
            actions: Vec<String>,
            objects: Vec<String>,
        }

        // Debug: print raw response (uncomment for debugging)
        // println!("🔍 Raw AI response for entities:");
        // println!("{}", response);
        
        // Try to extract JSON from response if it's wrapped in markdown
        let json_str = if response.contains("```json") {
            response.split("```json").nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response)
                .trim()
        } else if response.contains("```") {
            response.split("```").nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response)
                .trim()
        } else {
            response.trim()
        };

        let parsed: EntityResponse = serde_json::from_str(json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse LLM response for entities: {}. Raw response: {}", e, json_str))?;

        Ok(ExtractedEntities {
            actors: parsed.actors,
            actions: parsed.actions,
            objects: parsed.objects,
        })
    }

    fn detect_ambiguities(&self, text: &str) -> Vec<Ambiguity> {
        let mut ambiguities = Vec::new();

        for term_regex in &self.vague_terms {
            for mat in term_regex.find_iter(text) {
                ambiguities.push(Ambiguity {
                    text: mat.as_str().to_string(),
                    reason: "Vague or subjective term that lacks specific criteria".to_string(),
                    suggestions: vec![
                        "Define specific metrics or thresholds".to_string(),
                        "Provide measurable criteria".to_string(),
                    ],
                    severity: AmbiguitySeverity::Medium,
                });
            }
        }

        for mat in self.passive_voice.find_iter(text) {
            ambiguities.push(Ambiguity {
                text: mat.as_str().to_string(),
                reason: "Passive voice hides the responsible actor".to_string(),
                suggestions: vec![
                    "Specify who is responsible for the action".to_string(),
                    "Use active voice instead".to_string(),
                ],
                severity: AmbiguitySeverity::High,
            });
        }

        ambiguities
    }

    fn extract_entities(&self, text: &str) -> ExtractedEntities {
        let actor_patterns = vec![
            Regex::new(r"\b(user|admin|administrator|customer|client|system|service)\b").unwrap(),
            Regex::new(r"\b(as a|as an)\s+(\w+)").unwrap(),
        ];

        let action_patterns = vec![
            Regex::new(r"\b(create|update|delete|add|remove|login|logout|register|submit|send|receive)\b").unwrap(),
            Regex::new(r"\b(want to|need to|should|must|will|can)\s+(\w+)").unwrap(),
        ];

        let object_patterns = vec![
            Regex::new(r"\b(account|profile|password|email|data|file|document|report|dashboard)\b").unwrap(),
            Regex::new(r"\b(shopping cart|order|product|item|category)\b").unwrap(),
        ];

        let mut actors = Vec::new();
        let mut actions = Vec::new();
        let mut objects = Vec::new();

        for pattern in &actor_patterns {
            for captures in pattern.captures_iter(text) {
                if let Some(actor) = captures.get(0) {
                    actors.push(actor.as_str().to_string());
                }
            }
        }

        for pattern in &action_patterns {
            for captures in pattern.captures_iter(text) {
                if let Some(action) = captures.get(0) {
                    actions.push(action.as_str().to_string());
                }
            }
        }

        for pattern in &object_patterns {
            for captures in pattern.captures_iter(text) {
                if let Some(object) = captures.get(0) {
                    objects.push(object.as_str().to_string());
                }
            }
        }

        actors.sort();
        actors.dedup();
        actions.sort();
        actions.dedup();
        objects.sort();
        objects.dedup();

        ExtractedEntities {
            actors,
            actions,
            objects,
        }
    }

    pub fn generate_uml_use_case(&self, entities: &ExtractedEntities) -> String {
        let mut uml = String::from("@startuml\n");
        uml.push_str("!theme aws-orange\n");
        uml.push_str("title Requirements Use Case Diagram\n\n");

        // Add styling
        uml.push_str("skinparam usecase {\n");
        uml.push_str("    BackgroundColor lightblue\n");
        uml.push_str("    BorderColor blue\n");
        uml.push_str("    ArrowColor blue\n");
        uml.push_str("}\n");
        uml.push_str("skinparam actor {\n");
        uml.push_str("    BackgroundColor lightyellow\n");
        uml.push_str("    BorderColor orange\n");
        uml.push_str("}\n\n");

        // Generate actors with more context
        for actor in &entities.actors {
            let actor_id = actor.replace(" ", "_").replace("-", "_");
            uml.push_str(&format!("actor \"{}\" as {}\n", actor, actor_id));
        }

        uml.push('\n');

        // Generate use cases with better organization
        for (i, action) in entities.actions.iter().enumerate() {
            let action_clean = action.replace("\"", "'");
            uml.push_str(&format!("usecase UC{} as \"{}\\n<color:gray><size:10>Action #{}</size></color>\"\n", i + 1, action_clean, i + 1));
        }

        uml.push('\n');

        // Create more intelligent actor-action relationships
        for actor in &entities.actors {
            let actor_id = actor.replace(" ", "_").replace("-", "_");
            for (i, action) in entities.actions.iter().enumerate() {
                // Smart relationship mapping based on common patterns
                let should_connect = self.should_actor_connect_to_action(actor, action);
                if should_connect {
                    uml.push_str(&format!("{} --> UC{}\n", actor_id, i + 1));
                }
            }
        }

        // Add system boundary if objects exist
        if !entities.objects.is_empty() {
            uml.push_str("\nrectangle \"System Boundary\" {\n");
            for (i, _) in entities.actions.iter().enumerate() {
                uml.push_str(&format!("    UC{}\n", i + 1));
            }
            uml.push_str("}\n");
        }

        // Add relationships between use cases if applicable
        if entities.actions.len() > 1 {
            uml.push_str("\n' Use case relationships\n");
            for (i, action) in entities.actions.iter().enumerate() {
                if action.contains("login") || action.contains("authenticate") {
                    // Login typically extends or is included by other actions
                    for (j, other_action) in entities.actions.iter().enumerate() {
                        if i != j && (other_action.contains("create") || other_action.contains("update") || other_action.contains("delete")) {
                            uml.push_str(&format!("UC{} <.. UC{} : <<include>>\n", j + 1, i + 1));
                        }
                    }
                }
            }
        }

        // Add notes if relevant
        if !entities.objects.is_empty() {
            uml.push_str("\nnote right of ");
            if let Some(first_actor) = entities.actors.first() {
                let actor_id = first_actor.replace(" ", "_").replace("-", "_");
                uml.push_str(&format!("{}", actor_id));
            } else {
                uml.push_str("UC1");
            }
            uml.push_str("\n  System handles:\n");
            for (i, object) in entities.objects.iter().enumerate() {
                uml.push_str(&format!("  • {}\n", object));
                if i >= 4 { // Limit to prevent overcrowding
                    uml.push_str(&format!("  • ... and {} more\n", entities.objects.len() - 5));
                    break;
                }
            }
            uml.push_str("end note\n");
        }

        uml.push_str("\n@enduml");
        uml
    }

    // Enhanced UML generation with sequence diagrams
    pub fn generate_uml_sequence(&self, entities: &ExtractedEntities) -> String {
        let mut uml = String::from("@startuml\n");
        uml.push_str("!theme aws-orange\n");
        uml.push_str("title Requirements Sequence Diagram\n\n");

        // Add styling
        uml.push_str("skinparam sequence {\n");
        uml.push_str("    ArrowColor blue\n");
        uml.push_str("    ActorBorderColor orange\n");
        uml.push_str("    LifeLineBorderColor blue\n");
        uml.push_str("    ParticipantBorderColor lightblue\n");
        uml.push_str("}\n\n");

        // Define participants
        for actor in &entities.actors {
            uml.push_str(&format!("actor \"{}\" as {}\n", actor, actor.replace(" ", "_")));
        }

        // Add system participants
        if !entities.objects.is_empty() {
            uml.push_str("participant \"System\" as System\n");
            if entities.objects.len() > 0 {
                let primary_object = &entities.objects[0];
                uml.push_str(&format!("database \"{}\\nDatabase\" as DB\n", primary_object));
            }
        }

        uml.push_str("\n");

        // Generate sequence flows
        if !entities.actors.is_empty() && !entities.actions.is_empty() {
            let primary_actor = &entities.actors[0].replace(" ", "_");
            
            uml.push_str("== Main Flow ==\n");
            uml.push_str(&format!("activate {}\n", primary_actor));
            
            for (i, action) in entities.actions.iter().enumerate() {
                let action_clean = action.replace("\"", "'");
                
                if action.contains("login") || action.contains("authenticate") {
                    uml.push_str(&format!("{} -> System : {}\n", primary_actor, action_clean));
                    uml.push_str("activate System\n");
                    uml.push_str("System -> DB : Validate credentials\n");
                    uml.push_str("activate DB\n");
                    uml.push_str("DB --> System : Validation result\n");
                    uml.push_str("deactivate DB\n");
                    uml.push_str(&format!("System --> {} : Authentication status\n", primary_actor));
                    uml.push_str("deactivate System\n");
                } else if action.contains("create") || action.contains("add") {
                    uml.push_str(&format!("{} -> System : {}\n", primary_actor, action_clean));
                    uml.push_str("activate System\n");
                    uml.push_str("System -> System : Validate input\n");
                    uml.push_str("System -> DB : Store data\n");
                    uml.push_str("activate DB\n");
                    uml.push_str("DB --> System : Confirmation\n");
                    uml.push_str("deactivate DB\n");
                    uml.push_str(&format!("System --> {} : Success response\n", primary_actor));
                    uml.push_str("deactivate System\n");
                } else if action.contains("update") || action.contains("edit") {
                    uml.push_str(&format!("{} -> System : {}\n", primary_actor, action_clean));
                    uml.push_str("activate System\n");
                    uml.push_str("System -> DB : Retrieve current data\n");
                    uml.push_str("activate DB\n");
                    uml.push_str("DB --> System : Current data\n");
                    uml.push_str("System -> System : Apply changes\n");
                    uml.push_str("System -> DB : Update data\n");
                    uml.push_str("DB --> System : Update confirmation\n");
                    uml.push_str("deactivate DB\n");
                    uml.push_str(&format!("System --> {} : Update response\n", primary_actor));
                    uml.push_str("deactivate System\n");
                } else if action.contains("delete") || action.contains("remove") {
                    uml.push_str(&format!("{} -> System : {}\n", primary_actor, action_clean));
                    uml.push_str("activate System\n");
                    uml.push_str("System -> System : Check permissions\n");
                    uml.push_str("System -> DB : Delete data\n");
                    uml.push_str("activate DB\n");
                    uml.push_str("DB --> System : Deletion confirmation\n");
                    uml.push_str("deactivate DB\n");
                    uml.push_str(&format!("System --> {} : Deletion response\n", primary_actor));
                    uml.push_str("deactivate System\n");
                } else {
                    // Generic action
                    uml.push_str(&format!("{} -> System : {}\n", primary_actor, action_clean));
                    uml.push_str("activate System\n");
                    uml.push_str("System -> System : Process request\n");
                    if !entities.objects.is_empty() {
                        uml.push_str("System -> DB : Data operation\n");
                        uml.push_str("activate DB\n");
                        uml.push_str("DB --> System : Operation result\n");
                        uml.push_str("deactivate DB\n");
                    }
                    uml.push_str(&format!("System --> {} : Response\n", primary_actor));
                    uml.push_str("deactivate System\n");
                }
                
                if i < entities.actions.len() - 1 {
                    uml.push_str("\n");
                }
            }
            
            uml.push_str(&format!("deactivate {}\n", primary_actor));
        }

        // Add alternative flows if we have error scenarios
        if entities.actions.len() > 1 {
            uml.push_str("\n== Alternative Flow (Error Handling) ==\n");
            if let Some(primary_actor) = entities.actors.first() {
                let actor_id = primary_actor.replace(" ", "_");
                uml.push_str(&format!("{} -> System : Invalid request\n", actor_id));
                uml.push_str("activate System\n");
                uml.push_str("System -> System : Validate request\n");
                uml.push_str("note right : Validation fails\n");
                uml.push_str(&format!("System --> {} : Error response\n", actor_id));
                uml.push_str("deactivate System\n");
            }
        }

        uml.push_str("\n@enduml");
        uml
    }

    // Helper method to determine if an actor should connect to an action
    fn should_actor_connect_to_action(&self, actor: &str, action: &str) -> bool {
        let actor_lower = actor.to_lowercase();
        let action_lower = action.to_lowercase();

        // Admin actors can do most actions
        if actor_lower.contains("admin") || actor_lower.contains("administrator") {
            return true;
        }

        // User actors typically do user-facing actions
        if actor_lower.contains("user") || actor_lower.contains("customer") || actor_lower.contains("client") {
            return action_lower.contains("create") 
                || action_lower.contains("update") 
                || action_lower.contains("view")
                || action_lower.contains("login")
                || action_lower.contains("register")
                || action_lower.contains("submit")
                || action_lower.contains("request");
        }

        // System actors do system-level actions
        if actor_lower.contains("system") || actor_lower.contains("service") {
            return action_lower.contains("process")
                || action_lower.contains("validate")
                || action_lower.contains("send")
                || action_lower.contains("receive")
                || action_lower.contains("generate");
        }

        // Default: connect if there's only one actor or few actors
        true
    }

    // Generate UML class diagram
    pub fn generate_uml_class_diagram(&self, entities: &ExtractedEntities) -> String {
        let mut uml = String::from("@startuml\n");
        uml.push_str("!theme aws-orange\n");
        uml.push_str("title Requirements Class Diagram\n\n");

        // Add styling
        uml.push_str("skinparam class {\n");
        uml.push_str("    BackgroundColor lightblue\n");
        uml.push_str("    BorderColor blue\n");
        uml.push_str("    ArrowColor blue\n");
        uml.push_str("}\n\n");

        // Generate entity classes
        for object in &entities.objects {
            let class_name = self.to_pascal_case(object);
            uml.push_str(&format!("class {} {{\n", class_name));
            uml.push_str("  -id: String\n");
            uml.push_str("  -status: Status\n");
            uml.push_str("  -createdAt: Date\n");
            uml.push_str("  -updatedAt: Date\n");
            uml.push_str("  --\n");
            uml.push_str("  +getId(): String\n");
            uml.push_str("  +getStatus(): Status\n");
            uml.push_str("  +validate(): boolean\n");
            uml.push_str("  +updateStatus(Status): void\n");
            
            // Add action-related methods
            for action in &entities.actions {
                let method_name = self.to_camel_case(action);
                if action.contains("create") {
                    uml.push_str(&format!("  +{}(): {}\n", method_name, class_name));
                } else if action.contains("update") || action.contains("edit") {
                    uml.push_str(&format!("  +{}(): boolean\n", method_name));
                } else if action.contains("delete") || action.contains("remove") {
                    uml.push_str(&format!("  +{}(): boolean\n", method_name));
                }
            }
            uml.push_str("}\n\n");
        }

        // Generate actor classes
        for actor in &entities.actors {
            let class_name = self.to_pascal_case(actor);
            uml.push_str(&format!("class {} {{\n", class_name));
            uml.push_str("  -userId: String\n");
            uml.push_str("  -permissions: List<String>\n");
            uml.push_str("  -sessionToken: String\n");
            uml.push_str("  --\n");
            uml.push_str("  +authenticate(Credentials): boolean\n");
            uml.push_str("  +hasPermission(String): boolean\n");
            uml.push_str("  +logout(): void\n");
            uml.push_str("}\n\n");
        }

        // Generate Status enum
        if !entities.objects.is_empty() {
            uml.push_str("enum Status {\n");
            uml.push_str("  PENDING\n");
            uml.push_str("  ACTIVE\n");
            uml.push_str("  COMPLETED\n");
            uml.push_str("  FAILED\n");
            uml.push_str("}\n\n");
        }

        // Generate service class for business logic
        if !entities.actions.is_empty() {
            uml.push_str("class BusinessService {\n");
            for action in &entities.actions {
                let method_name = self.to_camel_case(action);
                uml.push_str(&format!("  +{}(Actor, Object, Map): Result\n", method_name));
            }
            uml.push_str("  +validateInput(Map): ValidationResult\n");
            uml.push_str("  +logAction(String, String, Object): void\n");
            uml.push_str("}\n\n");
        }

        // Generate relationships
        if !entities.actors.is_empty() && !entities.objects.is_empty() {
            let first_actor = self.to_pascal_case(&entities.actors[0]);
            for object in &entities.objects {
                let object_class = self.to_pascal_case(object);
                uml.push_str(&format!("{} --> {} : manages\n", first_actor, object_class));
            }
        }

        if !entities.objects.is_empty() {
            let first_object = self.to_pascal_case(&entities.objects[0]);
            uml.push_str(&format!("{} --> Status : has\n", first_object));
        }

        if !entities.actions.is_empty() {
            uml.push_str("BusinessService --> ");
            if !entities.objects.is_empty() {
                uml.push_str(&self.to_pascal_case(&entities.objects[0]));
            } else {
                uml.push_str("Object");
            }
            uml.push_str(" : processes\n");
        }

        uml.push_str("\n@enduml");
        uml
    }

    pub fn generate_pseudocode(&self, entities: &ExtractedEntities, language: Option<&str>) -> String {
        let lang = language.unwrap_or("generic");
        let mut code = String::new();

        match lang {
            "python" => {
                code.push_str("# Generated pseudocode with business logic\n");
                code.push_str("# This pseudocode provides a foundation for implementing the requirements\n\n");
                
                code.push_str("from typing import Optional, List, Dict\nfrom dataclasses import dataclass\nfrom enum import Enum\n\n");
                
                // Generate status/state enums
                if !entities.objects.is_empty() {
                    code.push_str("class Status(Enum):\n");
                    code.push_str("    PENDING = \"pending\"\n");
                    code.push_str("    ACTIVE = \"active\"\n");
                    code.push_str("    COMPLETED = \"completed\"\n");
                    code.push_str("    FAILED = \"failed\"\n\n");
                }

                // Generate data classes for entities
                for object in &entities.objects {
                    let class_name = self.to_pascal_case(object);
                    code.push_str(&format!("@dataclass\n"));
                    code.push_str(&format!("class {}:\n", class_name));
                    code.push_str("    id: str\n");
                    code.push_str("    status: Status = Status.PENDING\n");
                    code.push_str("    created_at: Optional[str] = None\n");
                    code.push_str("    updated_at: Optional[str] = None\n");
                    code.push_str("    \n");
                    code.push_str("    def validate(self) -> bool:\n");
                    code.push_str("        \"\"\"Validate the entity data\"\"\"\n");
                    code.push_str("        return bool(self.id and len(self.id.strip()) > 0)\n");
                    code.push_str("    \n");
                    code.push_str("    def to_dict(self) -> Dict:\n");
                    code.push_str("        \"\"\"Convert to dictionary representation\"\"\"\n");
                    code.push_str("        return {\n");
                    code.push_str("            'id': self.id,\n");
                    code.push_str("            'status': self.status.value,\n");
                    code.push_str("            'created_at': self.created_at,\n");
                    code.push_str("            'updated_at': self.updated_at\n");
                    code.push_str("        }\n\n");
                }

                // Generate actor classes with methods
                for actor in &entities.actors {
                    let class_name = self.to_pascal_case(actor);
                    code.push_str(&format!("class {}:\n", class_name));
                    code.push_str("    def __init__(self, user_id: str):\n");
                    code.push_str("        self.user_id = user_id\n");
                    code.push_str("        self.permissions = []\n");
                    code.push_str("        self.session_token = None\n");
                    code.push_str("    \n");
                    code.push_str("    def authenticate(self, credentials: Dict) -> bool:\n");
                    code.push_str("        \"\"\"Authenticate the actor with provided credentials\"\"\"\n");
                    code.push_str("        if not credentials.get('username') or not credentials.get('password'):\n");
                    code.push_str("            return False\n");
                    code.push_str("        \n");
                    code.push_str("        # Validate credentials against data source\n");
                    code.push_str("        is_valid = self._validate_credentials(credentials)\n");
                    code.push_str("        \n");
                    code.push_str("        if is_valid:\n");
                    code.push_str("            self.session_token = self._generate_session_token()\n");
                    code.push_str("            self.permissions = self._load_user_permissions()\n");
                    code.push_str("        \n");
                    code.push_str("        return is_valid\n");
                    code.push_str("    \n");
                    code.push_str("    def has_permission(self, permission: str) -> bool:\n");
                    code.push_str("        \"\"\"Check if actor has specific permission\"\"\"\n");
                    code.push_str("        return permission in self.permissions\n");
                    code.push_str("    \n");
                    code.push_str("    def _validate_credentials(self, credentials: Dict) -> bool:\n");
                    code.push_str("        # Implementation: Query user database\n");
                    code.push_str("        # Check password hash, account status, etc.\n");
                    code.push_str("        pass\n");
                    code.push_str("    \n");
                    code.push_str("    def _generate_session_token(self) -> str:\n");
                    code.push_str("        # Implementation: Generate secure JWT or session token\n");
                    code.push_str("        pass\n");
                    code.push_str("    \n");
                    code.push_str("    def _load_user_permissions(self) -> List[str]:\n");
                    code.push_str("        # Implementation: Load user roles and permissions\n");
                    code.push_str("        pass\n\n");
                }

                // Generate action functions with business logic
                for action in &entities.actions {
                    let function_name = self.to_snake_case(action);
                    code.push_str(&format!("def {}(actor, target_object=None, **kwargs) -> Dict:\n", function_name));
                    code.push_str(&format!("    \"\"\"\n"));
                    code.push_str(&format!("    Execute {} action\n", action));
                    code.push_str("    \n");
                    code.push_str("    Args:\n");
                    code.push_str("        actor: The entity performing the action\n");
                    code.push_str("        target_object: The object being acted upon (optional)\n");
                    code.push_str("        **kwargs: Additional parameters\n");
                    code.push_str("    \n");
                    code.push_str("    Returns:\n");
                    code.push_str("        Dict: Result with success status and data\n");
                    code.push_str("    \"\"\"\n");
                    code.push_str("    \n");
                    code.push_str("    # Step 1: Validate preconditions\n");
                    code.push_str("    if not actor or not hasattr(actor, 'user_id'):\n");
                    code.push_str("        return {'success': False, 'error': 'Invalid actor'}\n");
                    code.push_str("    \n");
                    code.push_str("    # Step 2: Check permissions\n");
                    code.push_str(&format!("    required_permission = '{}'\n", function_name));
                    code.push_str("    if not actor.has_permission(required_permission):\n");
                    code.push_str("        return {'success': False, 'error': 'Insufficient permissions'}\n");
                    code.push_str("    \n");
                    code.push_str("    # Step 3: Validate input data\n");
                    code.push_str("    validation_result = _validate_action_input(kwargs)\n");
                    code.push_str("    if not validation_result['valid']:\n");
                    code.push_str("        return {'success': False, 'error': validation_result['error']}\n");
                    code.push_str("    \n");
                    code.push_str("    try:\n");
                    code.push_str("        # Step 4: Execute business logic\n");
                    code.push_str(&format!("        result = _execute_{}(actor, target_object, **kwargs)\n", function_name));
                    code.push_str("        \n");
                    code.push_str("        # Step 5: Update object state if applicable\n");
                    code.push_str("        if target_object:\n");
                    code.push_str("            target_object.status = Status.COMPLETED\n");
                    code.push_str("            target_object.updated_at = _get_current_timestamp()\n");
                    code.push_str("        \n");
                    code.push_str("        # Step 6: Log the action\n");
                    code.push_str(&format!("        _log_action('{}', actor.user_id, result)\n", action));
                    code.push_str("        \n");
                    code.push_str("        return {'success': True, 'data': result}\n");
                    code.push_str("        \n");
                    code.push_str("    except Exception as e:\n");
                    code.push_str("        # Step 7: Handle errors gracefully\n");
                    code.push_str(&format!("        _log_error('{}', actor.user_id, str(e))\n", action));
                    code.push_str("        return {'success': False, 'error': f'Action failed: {str(e)}'}\n\n");
                }

                // Generate helper functions
                code.push_str("# Helper functions\n\n");
                code.push_str("def _validate_action_input(input_data: Dict) -> Dict:\n");
                code.push_str("    \"\"\"Validate input parameters for any action\"\"\"\n");
                code.push_str("    # Implementation: Check required fields, data types, ranges\n");
                code.push_str("    # Return {'valid': True/False, 'error': 'message'}\n");
                code.push_str("    return {'valid': True, 'error': None}\n\n");
                
                for action in &entities.actions {
                    let function_name = self.to_snake_case(action);
                    code.push_str(&format!("def _execute_{}(actor, target_object, **kwargs):\n", function_name));
                    code.push_str(&format!("    \"\"\"Core business logic for {} action\"\"\"\n", action));
                    code.push_str("    # Implementation: Actual business logic here\n");
                    code.push_str("    # Database operations, external API calls, calculations, etc.\n");
                    code.push_str("    pass\n\n");
                }

                code.push_str("def _log_action(action_name: str, user_id: str, result):\n");
                code.push_str("    \"\"\"Log successful actions for audit trail\"\"\"\n");
                code.push_str("    # Implementation: Write to audit log, database, or monitoring system\n");
                code.push_str("    pass\n\n");

                code.push_str("def _log_error(action_name: str, user_id: str, error_msg: str):\n");
                code.push_str("    \"\"\"Log errors for troubleshooting\"\"\"\n");
                code.push_str("    # Implementation: Write to error log, monitoring system\n");
                code.push_str("    pass\n\n");

                code.push_str("def _get_current_timestamp() -> str:\n");
                code.push_str("    \"\"\"Get current timestamp in ISO format\"\"\"\n");
                code.push_str("    from datetime import datetime\n");
                code.push_str("    return datetime.now().isoformat()\n");
            }
            _ => {
                // Enhanced generic/Java-style pseudocode
                code.push_str("// Generated pseudocode with business logic\n");
                code.push_str("// This pseudocode provides a foundation for implementing the requirements\n\n");

                // Generate enums
                if !entities.objects.is_empty() {
                    code.push_str("enum Status {\n");
                    code.push_str("    PENDING,\n");
                    code.push_str("    ACTIVE,\n");
                    code.push_str("    COMPLETED,\n");
                    code.push_str("    FAILED\n");
                    code.push_str("}\n\n");
                }

                // Generate object classes
                for object in &entities.objects {
                    let class_name = self.to_pascal_case(object);
                    code.push_str(&format!("class {} {{\n", class_name));
                    code.push_str("    private String id;\n");
                    code.push_str("    private Status status;\n");
                    code.push_str("    private String createdAt;\n");
                    code.push_str("    private String updatedAt;\n");
                    code.push_str("    \n");
                    code.push_str(&format!("    public {}(String id) {{\n", class_name));
                    code.push_str("        this.id = id;\n");
                    code.push_str("        this.status = Status.PENDING;\n");
                    code.push_str("        this.createdAt = getCurrentTimestamp();\n");
                    code.push_str("    }\n");
                    code.push_str("    \n");
                    code.push_str("    public boolean validate() {\n");
                    code.push_str("        return id != null && !id.trim().isEmpty();\n");
                    code.push_str("    }\n");
                    code.push_str("    \n");
                    code.push_str("    public void updateStatus(Status newStatus) {\n");
                    code.push_str("        this.status = newStatus;\n");
                    code.push_str("        this.updatedAt = getCurrentTimestamp();\n");
                    code.push_str("    }\n");
                    code.push_str("    \n");
                    code.push_str("    // Getters and setters\n");
                    code.push_str("    public String getId() { return id; }\n");
                    code.push_str("    public Status getStatus() { return status; }\n");
                    code.push_str("}\n\n");
                }

                // Generate actor classes
                for actor in &entities.actors {
                    let class_name = self.to_pascal_case(actor);
                    code.push_str(&format!("class {} {{\n", class_name));
                    code.push_str("    private String userId;\n");
                    code.push_str("    private List<String> permissions;\n");
                    code.push_str("    private String sessionToken;\n");
                    code.push_str("    \n");
                    code.push_str(&format!("    public {}(String userId) {{\n", class_name));
                    code.push_str("        this.userId = userId;\n");
                    code.push_str("        this.permissions = new ArrayList<>();\n");
                    code.push_str("    }\n");
                    code.push_str("    \n");
                    code.push_str("    public boolean authenticate(Credentials credentials) {\n");
                    code.push_str("        if (credentials == null || !credentials.isValid()) {\n");
                    code.push_str("            return false;\n");
                    code.push_str("        }\n");
                    code.push_str("        \n");
                    code.push_str("        boolean isValid = validateCredentials(credentials);\n");
                    code.push_str("        \n");
                    code.push_str("        if (isValid) {\n");
                    code.push_str("            this.sessionToken = generateSessionToken();\n");
                    code.push_str("            this.permissions = loadUserPermissions();\n");
                    code.push_str("        }\n");
                    code.push_str("        \n");
                    code.push_str("        return isValid;\n");
                    code.push_str("    }\n");
                    code.push_str("    \n");
                    code.push_str("    public boolean hasPermission(String permission) {\n");
                    code.push_str("        return permissions.contains(permission);\n");
                    code.push_str("    }\n");
                    code.push_str("    \n");
                    code.push_str("    private boolean validateCredentials(Credentials credentials) {\n");
                    code.push_str("        // Implementation: Query user database\n");
                    code.push_str("        // Check password hash, account status, etc.\n");
                    code.push_str("        return false; // placeholder\n");
                    code.push_str("    }\n");
                    code.push_str("    \n");
                    code.push_str("    private String generateSessionToken() {\n");
                    code.push_str("        // Implementation: Generate secure JWT or session token\n");
                    code.push_str("        return null; // placeholder\n");
                    code.push_str("    }\n");
                    code.push_str("    \n");
                    code.push_str("    private List<String> loadUserPermissions() {\n");
                    code.push_str("        // Implementation: Load user roles and permissions\n");
                    code.push_str("        return new ArrayList<>(); // placeholder\n");
                    code.push_str("    }\n");
                    code.push_str("}\n\n");
                }

                // Generate service classes for actions
                code.push_str("class BusinessLogicService {\n");
                for action in &entities.actions {
                    let method_name = self.to_camel_case(action);
                    code.push_str(&format!("    public Result {}(Actor actor, Object targetObject, Map<String, Object> parameters) {{\n", method_name));
                    code.push_str("        // Step 1: Validate preconditions\n");
                    code.push_str("        if (actor == null) {\n");
                    code.push_str("            return Result.failure(\"Invalid actor\");\n");
                    code.push_str("        }\n");
                    code.push_str("        \n");
                    code.push_str("        // Step 2: Check permissions\n");
                    code.push_str(&format!("        String requiredPermission = \"{}\";\n", method_name));
                    code.push_str("        if (!actor.hasPermission(requiredPermission)) {\n");
                    code.push_str("            return Result.failure(\"Insufficient permissions\");\n");
                    code.push_str("        }\n");
                    code.push_str("        \n");
                    code.push_str("        // Step 3: Validate input\n");
                    code.push_str("        ValidationResult validation = validateInput(parameters);\n");
                    code.push_str("        if (!validation.isValid()) {\n");
                    code.push_str("            return Result.failure(validation.getError());\n");
                    code.push_str("        }\n");
                    code.push_str("        \n");
                    code.push_str("        try {\n");
                    code.push_str("            // Step 4: Execute business logic\n");
                    code.push_str(&format!("            Object result = execute{}(actor, targetObject, parameters);\n", self.to_pascal_case(action)));
                    code.push_str("            \n");
                    code.push_str("            // Step 5: Update state\n");
                    code.push_str("            if (targetObject != null) {\n");
                    code.push_str("                targetObject.updateStatus(Status.COMPLETED);\n");
                    code.push_str("            }\n");
                    code.push_str("            \n");
                    code.push_str("            // Step 6: Log action\n");
                    code.push_str(&format!("            logAction(\"{}\", actor.getUserId(), result);\n", action));
                    code.push_str("            \n");
                    code.push_str("            return Result.success(result);\n");
                    code.push_str("            \n");
                    code.push_str("        } catch (Exception e) {\n");
                    code.push_str("            // Step 7: Handle errors\n");
                    code.push_str(&format!("            logError(\"{}\", actor.getUserId(), e.getMessage());\n", action));
                    code.push_str("            return Result.failure(\"Action failed: \" + e.getMessage());\n");
                    code.push_str("        }\n");
                    code.push_str("    }\n");
                    code.push_str("    \n");
                }

                // Helper methods
                code.push_str("    private ValidationResult validateInput(Map<String, Object> input) {\n");
                code.push_str("        // Implementation: Validate input parameters\n");
                code.push_str("        return ValidationResult.valid();\n");
                code.push_str("    }\n");
                code.push_str("    \n");

                for action in &entities.actions {
                    let method_name = self.to_pascal_case(action);
                    code.push_str(&format!("    private Object execute{}(Actor actor, Object targetObject, Map<String, Object> parameters) {{\n", method_name));
                    code.push_str(&format!("        // Core business logic for {} action\n", action));
                    code.push_str("        // Database operations, external API calls, calculations, etc.\n");
                    code.push_str("        return null; // placeholder\n");
                    code.push_str("    }\n");
                    code.push_str("    \n");
                }

                code.push_str("    private void logAction(String actionName, String userId, Object result) {\n");
                code.push_str("        // Implementation: Write to audit log\n");
                code.push_str("    }\n");
                code.push_str("    \n");
                code.push_str("    private void logError(String actionName, String userId, String error) {\n");
                code.push_str("        // Implementation: Write to error log\n");
                code.push_str("    }\n");
                code.push_str("    \n");
                code.push_str("    private String getCurrentTimestamp() {\n");
                code.push_str("        return Instant.now().toString();\n");
                code.push_str("    }\n");
                code.push_str("}\n\n");

                // Result class
                code.push_str("class Result {\n");
                code.push_str("    private boolean success;\n");
                code.push_str("    private Object data;\n");
                code.push_str("    private String error;\n");
                code.push_str("    \n");
                code.push_str("    public static Result success(Object data) {\n");
                code.push_str("        Result result = new Result();\n");
                code.push_str("        result.success = true;\n");
                code.push_str("        result.data = data;\n");
                code.push_str("        return result;\n");
                code.push_str("    }\n");
                code.push_str("    \n");
                code.push_str("    public static Result failure(String error) {\n");
                code.push_str("        Result result = new Result();\n");
                code.push_str("        result.success = false;\n");
                code.push_str("        result.error = error;\n");
                code.push_str("        return result;\n");
                code.push_str("    }\n");
                code.push_str("    \n");
                code.push_str("    // Getters\n");
                code.push_str("    public boolean isSuccess() { return success; }\n");
                code.push_str("    public Object getData() { return data; }\n");
                code.push_str("    public String getError() { return error; }\n");
                code.push_str("}\n");
            }
        }

        code
    }

    // Helper methods for string case conversion
    fn to_pascal_case(&self, s: &str) -> String {
        s.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect()
    }

    fn to_snake_case(&self, s: &str) -> String {
        s.to_lowercase().replace(" ", "_").replace("-", "_")
    }

    fn to_camel_case(&self, s: &str) -> String {
        let words: Vec<&str> = s.split_whitespace().collect();
        if words.is_empty() {
            return String::new();
        }

        let mut result = words[0].to_lowercase();
        for word in &words[1..] {
            let mut chars = word.chars();
            match chars.next() {
                None => continue,
                Some(first) => result.push_str(&(first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase())),
            }
        }
        result
    }

    pub fn generate_test_cases(&self, entities: &ExtractedEntities) -> TestCases {
        let mut happy_path = Vec::new();
        let mut negative_cases = Vec::new();
        let mut edge_cases = Vec::new();

        for action in &entities.actions {
            happy_path.push(format!("Test successful execution of {}", action));
            negative_cases.push(format!("Test {} with invalid input", action));
            negative_cases.push(format!("Test {} without proper authorization", action));
            edge_cases.push(format!("Test {} with empty/null values", action));
            edge_cases.push(format!("Test {} with maximum input size", action));
        }

        TestCases {
            happy_path,
            negative_cases,
            edge_cases,
        }
    }

    pub async fn generate_improved_requirements(&self, original_text: &str, ambiguities: &[Ambiguity]) -> Result<String> {
        if let Some(config) = &self.config {
            if config.llm.api_key.is_some() {
                return self.improve_requirements_with_llm(original_text, ambiguities).await;
            }
        }
        
        // Fallback: basic improvement without AI
        let mut improved = original_text.to_string();
        improved.push_str("\n\n<!-- PRISM IMPROVEMENT NOTES -->\n");
        improved.push_str("<!-- AI not configured. Manual improvements recommended: -->\n");
        
        for (i, ambiguity) in ambiguities.iter().enumerate() {
            improved.push_str(&format!("<!-- {}: {} - {} -->\n", 
                i + 1, ambiguity.text, ambiguity.reason));
        }
        
        Ok(improved)
    }

    async fn improve_requirements_with_llm(&self, original_text: &str, ambiguities: &[Ambiguity]) -> Result<String> {
        let ambiguities_summary = ambiguities.iter()
            .map(|a| format!("- Issue: '{}'\n  Problem: {}\n  Suggestions: {}", 
                a.text, a.reason, a.suggestions.join(", ")))
            .collect::<Vec<_>>()
            .join("\n\n");

        let prompt = format!(
            "You are a requirements improvement specialist. Please rewrite the following requirements to fix all identified ambiguities and make them clearer, more specific, and more actionable.

ORIGINAL REQUIREMENTS:
{}

IDENTIFIED ISSUES TO FIX:
{}

INSTRUCTIONS:
1. Rewrite the requirements to address all identified issues
2. Make vague terms specific and measurable
3. Replace passive voice with active voice
4. Add missing details and clarifications
5. Ensure requirements are testable and implementable
6. Maintain the original intent and scope
7. Use clear, professional language
8. Keep the same overall structure and format

Please provide ONLY the improved requirements text, without explanations or comments.",
            original_text,
            ambiguities_summary
        );

        let response = self.call_llm(&prompt).await?;
        Ok(response.trim().to_string())
    }

    pub async fn analyze_completeness(&self, text: &str, entities: &ExtractedEntities) -> Result<CompletenessAnalysis> {
        let mut gaps = Vec::new();
        let mut missing_actors = Vec::new();
        let mut missing_success_criteria = Vec::new();
        let mut missing_nf_considerations = Vec::new();

        // Basic completeness checks
        if entities.actors.is_empty() {
            missing_actors.push("No actors identified - who will perform these actions?".to_string());
            gaps.push(Gap {
                category: "Actor Definition".to_string(),
                description: "No clear actors identified in the requirement".to_string(),
                suggestions: vec![
                    "Specify who will perform the actions (e.g., 'user', 'administrator', 'system')".to_string(),
                    "Define user roles and permissions".to_string(),
                ],
                priority: GapPriority::Critical,
            });
        }

        if !text.to_lowercase().contains("success") && !text.to_lowercase().contains("acceptance") && !text.to_lowercase().contains("criteria") {
            missing_success_criteria.push("No success criteria or acceptance criteria specified".to_string());
            gaps.push(Gap {
                category: "Acceptance Criteria".to_string(),
                description: "Missing clear success criteria".to_string(),
                suggestions: vec![
                    "Add 'Given-When-Then' scenarios".to_string(),
                    "Define measurable outcomes".to_string(),
                    "Specify validation criteria".to_string(),
                ],
                priority: GapPriority::High,
            });
        }

        // Check for missing non-functional considerations
        let nf_keywords = vec!["performance", "security", "usability", "reliability", "scalability"];
        let has_nf = nf_keywords.iter().any(|keyword| text.to_lowercase().contains(keyword));
        
        if !has_nf {
            missing_nf_considerations.push("No non-functional requirements considered".to_string());
            gaps.push(Gap {
                category: "Non-Functional Requirements".to_string(),
                description: "Missing performance, security, or other quality attributes".to_string(),
                suggestions: vec![
                    "Consider performance requirements (response time, throughput)".to_string(),
                    "Define security requirements (authentication, authorization)".to_string(),
                    "Specify usability requirements (user experience)".to_string(),
                ],
                priority: GapPriority::Medium,
            });
        }

        // Use AI for enhanced completeness analysis if available
        if let Some(config) = &self.config {
            if config.llm.api_key.is_some() {
                match self.analyze_completeness_with_llm(text, entities).await {
                    Ok(ai_gaps) => {
                        gaps.extend(ai_gaps);
                    }
                    Err(_) => {
                        // Fall back to basic analysis
                    }
                }
            }
        }

        // Calculate completeness score
        let total_checks = 10; // Number of completeness criteria
        let missing_count = gaps.len();
        let completeness_score = ((total_checks - missing_count.min(total_checks)) as f32 / total_checks as f32) * 100.0;

        Ok(CompletenessAnalysis {
            missing_actors,
            missing_success_criteria,
            missing_nf_considerations,
            completeness_score,
            gaps_identified: gaps,
        })
    }

    async fn analyze_completeness_with_llm(&self, text: &str, entities: &ExtractedEntities) -> Result<Vec<Gap>> {
        let prompt = format!(
            "Analyze the following requirement for completeness and identify gaps. Consider missing actors, undefined success criteria, missing non-functional requirements, and other completeness issues.

Requirement: {}

Identified entities:
- Actors: {:?}
- Actions: {:?}  
- Objects: {:?}

Please identify gaps and provide suggestions in the following JSON format:
{{
    \"gaps\": [
        {{
            \"category\": \"category name\",
            \"description\": \"what is missing\",
            \"suggestions\": [\"suggestion 1\", \"suggestion 2\"],
            \"priority\": \"Critical|High|Medium|Low\"
        }}
    ]
}}",
            text, entities.actors, entities.actions, entities.objects
        );

        let response = self.call_llm(&prompt).await?;
        self.parse_gaps_response(&response)
    }

    fn parse_gaps_response(&self, response: &str) -> Result<Vec<Gap>> {
        #[derive(Deserialize)]
        struct GapsResponse {
            gaps: Vec<GapData>,
        }

        #[derive(Deserialize)]
        struct GapData {
            category: String,
            description: String,
            suggestions: Vec<String>,
            priority: String,
        }

        let json_str = if response.contains("```json") {
            response.split("```json").nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response)
                .trim()
        } else {
            response.trim()
        };

        let parsed: GapsResponse = serde_json::from_str(json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse gaps response: {}. Raw: {}", e, json_str))?;

        Ok(parsed.gaps.into_iter().map(|data| {
            let priority = match data.priority.as_str() {
                "Critical" => GapPriority::Critical,
                "High" => GapPriority::High,
                "Medium" => GapPriority::Medium,
                _ => GapPriority::Low,
            };

            Gap {
                category: data.category,
                description: data.description,
                suggestions: data.suggestions,
                priority,
            }
        }).collect())
    }

    pub fn validate_user_story(&self, text: &str) -> UserStoryValidation {
        let user_story_pattern = regex::Regex::new(r"(?i)as\s+(?:a|an)\s+([^,]+),?\s+i\s+want\s+([^,]+?),?\s+so\s+that\s+(.+)").unwrap();
        
        if let Some(captures) = user_story_pattern.captures(text) {
            let actor = captures.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            let goal = captures.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            let reason = captures.get(3).map(|m| m.as_str().trim()).unwrap_or("");

            let actor_quality = self.validate_user_story_component(actor, "actor");
            let goal_quality = self.validate_user_story_component(goal, "goal");
            let reason_quality = self.validate_user_story_component(reason, "reason");

            let business_value_score = self.calculate_business_value_score(&reason);
            
            let mut recommendations = Vec::new();
            if !actor_quality.is_valid {
                recommendations.extend(actor_quality.suggestions.clone());
            }
            if !goal_quality.is_valid {
                recommendations.extend(goal_quality.suggestions.clone());
            }
            if !reason_quality.is_valid {
                recommendations.extend(reason_quality.suggestions.clone());
            }

            UserStoryValidation {
                is_valid_format: true,
                actor_quality,
                goal_quality,
                reason_quality,
                business_value_score,
                recommendations,
            }
        } else {
            UserStoryValidation {
                is_valid_format: false,
                actor_quality: ValidationResult {
                    is_valid: false,
                    score: 0.0,
                    issues: vec!["Not in user story format".to_string()],
                    suggestions: vec!["Use format: 'As a [user], I want [goal], so that [reason]'".to_string()],
                },
                goal_quality: ValidationResult {
                    is_valid: false,
                    score: 0.0,
                    issues: vec!["Goal not identified".to_string()],
                    suggestions: vec!["Specify what the user wants to achieve".to_string()],
                },
                reason_quality: ValidationResult {
                    is_valid: false,
                    score: 0.0,
                    issues: vec!["Business reason not provided".to_string()],
                    suggestions: vec!["Explain the business value or benefit".to_string()],
                },
                business_value_score: 0.0,
                recommendations: vec!["Convert to proper user story format: 'As a [user], I want [goal], so that [reason]'".to_string()],
            }
        }
    }

    fn validate_user_story_component(&self, component: &str, component_type: &str) -> ValidationResult {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut score: f32 = 100.0;

        if component.is_empty() {
            issues.push(format!("{} is empty", component_type));
            suggestions.push(format!("Provide a clear {}", component_type));
            score = 0.0;
        } else if component.len() < 3 {
            issues.push(format!("{} is too vague", component_type));
            suggestions.push(format!("Be more specific about the {}", component_type));
            score -= 50.0;
        }

        // Check for vague terms
        let vague_terms = ["thing", "stuff", "something", "anything", "everything"];
        if vague_terms.iter().any(|term| component.to_lowercase().contains(term)) {
            issues.push("Contains vague terms".to_string());
            suggestions.push("Replace vague terms with specific descriptions".to_string());
            score -= 30.0;
        }

        // Component-specific validation
        match component_type {
            "actor" => {
                if !component.to_lowercase().contains("user") && 
                   !component.to_lowercase().contains("admin") && 
                   !component.to_lowercase().contains("customer") &&
                   !component.to_lowercase().contains("system") {
                    suggestions.push("Consider specifying the user role (e.g., 'customer', 'administrator')".to_string());
                    score -= 10.0;
                }
            },
            "goal" => {
                if !component.contains(" ") {
                    issues.push("Goal seems too simple".to_string());
                    suggestions.push("Provide more detail about what the user wants to accomplish".to_string());
                    score -= 20.0;
                }
            },
            "reason" => {
                if !component.to_lowercase().contains("can") && 
                   !component.to_lowercase().contains("will") &&
                   !component.to_lowercase().contains("able") &&
                   !component.to_lowercase().contains("benefit") {
                    issues.push("Business value unclear".to_string());
                    suggestions.push("Explain the benefit or value this provides".to_string());
                    score -= 25.0;
                }
            },
            _ => {}
        }

        ValidationResult {
            is_valid: issues.is_empty(),
            score: score.max(0.0),
            issues,
            suggestions,
        }
    }

    fn calculate_business_value_score(&self, reason: &str) -> f32 {
        let mut score = 50.0; // Base score
        
        // Positive indicators
        let value_keywords = ["save", "increase", "improve", "reduce", "efficiency", "productivity", "revenue", "cost"];
        let value_count = value_keywords.iter()
            .filter(|keyword| reason.to_lowercase().contains(*keyword))
            .count();
        score += (value_count as f32) * 10.0;

        // Specific benefits
        if reason.to_lowercase().contains("time") {
            score += 15.0;
        }
        if reason.to_lowercase().contains("money") || reason.to_lowercase().contains("cost") {
            score += 20.0;
        }
        if reason.to_lowercase().contains("user experience") || reason.to_lowercase().contains("satisfaction") {
            score += 15.0;
        }

        // Negative indicators
        if reason.len() < 10 {
            score -= 30.0;
        }
        if reason.to_lowercase().contains("just") || reason.to_lowercase().contains("because") {
            score -= 20.0;
        }

        score.min(100.0).max(0.0)
    }

    pub async fn generate_nfr_suggestions(&self, text: &str, entities: &ExtractedEntities) -> Result<Vec<NonFunctionalRequirement>> {
        let mut nfrs = Vec::new();

        // Generate basic NFRs based on actions and objects
        for action in &entities.actions {
            match action.to_lowercase().as_str() {
                action if action.contains("login") || action.contains("authenticate") => {
                    nfrs.push(NonFunctionalRequirement {
                        category: NfrCategory::Security,
                        requirement: "The system shall implement secure authentication with multi-factor authentication options".to_string(),
                        rationale: "Login functionality requires strong security to protect user accounts".to_string(),
                        acceptance_criteria: vec![
                            "Support for 2FA/MFA authentication methods".to_string(),
                            "Password complexity requirements enforced".to_string(),
                            "Account lockout after failed attempts".to_string(),
                        ],
                        priority: NfrPriority::MustHave,
                    });

                    nfrs.push(NonFunctionalRequirement {
                        category: NfrCategory::Performance,
                        requirement: "Authentication process shall complete within 2 seconds under normal load".to_string(),
                        rationale: "Users expect quick login response times for good user experience".to_string(),
                        acceptance_criteria: vec![
                            "95% of authentication requests complete within 2 seconds".to_string(),
                            "System supports concurrent authentication requests".to_string(),
                        ],
                        priority: NfrPriority::ShouldHave,
                    });
                },
                action if action.contains("upload") => {
                    nfrs.push(NonFunctionalRequirement {
                        category: NfrCategory::Security,
                        requirement: "Uploaded files shall be scanned for malware and restricted by type and size".to_string(),
                        rationale: "File uploads pose security risks and must be controlled".to_string(),
                        acceptance_criteria: vec![
                            "All uploads scanned by antivirus".to_string(),
                            "File type restrictions enforced".to_string(),
                            "Maximum file size limits applied".to_string(),
                        ],
                        priority: NfrPriority::MustHave,
                    });

                    nfrs.push(NonFunctionalRequirement {
                        category: NfrCategory::Performance,
                        requirement: "File uploads shall support resume functionality and progress indication".to_string(),
                        rationale: "Large file uploads need reliability and user feedback".to_string(),
                        acceptance_criteria: vec![
                            "Upload progress displayed to user".to_string(),
                            "Failed uploads can be resumed".to_string(),
                            "Upload speed optimized for large files".to_string(),
                        ],
                        priority: NfrPriority::ShouldHave,
                    });
                },
                action if action.contains("search") || action.contains("find") => {
                    nfrs.push(NonFunctionalRequirement {
                        category: NfrCategory::Performance,
                        requirement: "Search results shall be returned within 1 second for 95% of queries".to_string(),
                        rationale: "Users expect fast search response times".to_string(),
                        acceptance_criteria: vec![
                            "Search index optimized for performance".to_string(),
                            "Results paginated for large datasets".to_string(),
                            "Search suggestions provided for no results".to_string(),
                        ],
                        priority: NfrPriority::MustHave,
                    });
                },
                _ => {}
            }
        }

        // Use AI for enhanced NFR generation if available
        if let Some(config) = &self.config {
            if config.llm.api_key.is_some() {
                match self.generate_nfrs_with_llm(text, entities).await {
                    Ok(ai_nfrs) => {
                        nfrs.extend(ai_nfrs);
                    }
                    Err(_) => {
                        // Continue with basic NFRs
                    }
                }
            }
        }

        Ok(nfrs)
    }

    async fn generate_nfrs_with_llm(&self, text: &str, entities: &ExtractedEntities) -> Result<Vec<NonFunctionalRequirement>> {
        let prompt = format!(
            "Based on the following functional requirement, generate relevant non-functional requirements (NFRs) for performance, security, usability, reliability, scalability, maintainability, compatibility, and accessibility.

Functional Requirement: {}

Identified entities:
- Actors: {:?}
- Actions: {:?}
- Objects: {:?}

Generate NFRs in the following JSON format:
{{
    \"nfrs\": [
        {{
            \"category\": \"Performance|Security|Usability|Reliability|Scalability|Maintainability|Compatibility|Accessibility\",
            \"requirement\": \"specific NFR statement\",
            \"rationale\": \"why this NFR is needed\",
            \"acceptance_criteria\": [\"criterion 1\", \"criterion 2\"],
            \"priority\": \"MustHave|ShouldHave|CouldHave|WontHave\"
        }}
    ]
}}",
            text, entities.actors, entities.actions, entities.objects
        );

        let response = self.call_llm(&prompt).await?;
        self.parse_nfr_response(&response)
    }

    fn parse_nfr_response(&self, response: &str) -> Result<Vec<NonFunctionalRequirement>> {
        #[derive(Deserialize)]
        struct NfrResponse {
            nfrs: Vec<NfrData>,
        }

        #[derive(Deserialize)]
        struct NfrData {
            category: String,
            requirement: String,
            rationale: String,
            acceptance_criteria: Vec<String>,
            priority: String,
        }

        let json_str = if response.contains("```json") {
            response.split("```json").nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response)
                .trim()
        } else {
            response.trim()
        };

        let parsed: NfrResponse = serde_json::from_str(json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse NFR response: {}. Raw: {}", e, json_str))?;

        Ok(parsed.nfrs.into_iter().map(|data| {
            let category = match data.category.as_str() {
                "Performance" => NfrCategory::Performance,
                "Security" => NfrCategory::Security,
                "Usability" => NfrCategory::Usability,
                "Reliability" => NfrCategory::Reliability,
                "Scalability" => NfrCategory::Scalability,
                "Maintainability" => NfrCategory::Maintainability,
                "Compatibility" => NfrCategory::Compatibility,
                "Accessibility" => NfrCategory::Accessibility,
                _ => NfrCategory::Performance,
            };

            let priority = match data.priority.as_str() {
                "MustHave" => NfrPriority::MustHave,
                "ShouldHave" => NfrPriority::ShouldHave,
                "CouldHave" => NfrPriority::CouldHave,
                "WontHave" => NfrPriority::WontHave,
                _ => NfrPriority::ShouldHave,
            };

            NonFunctionalRequirement {
                category,
                requirement: data.requirement,
                rationale: data.rationale,
                acceptance_criteria: data.acceptance_criteria,
                priority,
            }
        }).collect())
    }
}