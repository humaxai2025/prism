use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use handlebars::{Handlebars, no_escape};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub content: String,
    pub variables: Vec<TemplateVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    pub default_value: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub variables: HashMap<String, String>,
    pub branding: Option<String>,
    pub timestamp: String,
    pub version: String,
}

pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
    template_dir: Option<PathBuf>,
    built_in_templates: HashMap<String, Template>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        handlebars.register_escape_fn(no_escape);
        
        // Register custom helpers
        Self::register_helpers(&mut handlebars);
        
        Self {
            handlebars,
            template_dir: None,
            built_in_templates: Self::create_built_in_templates(),
        }
    }

    pub fn with_template_dir(template_dir: PathBuf) -> Self {
        let mut engine = Self::new();
        engine.template_dir = Some(template_dir);
        engine
    }

    fn register_helpers(handlebars: &mut Handlebars) {
        // Helper for formatting dates
        handlebars.register_helper("format_date", Box::new(|h, _, hb, _, out| {
            let format = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("%Y-%m-%d %H:%M:%S");
            let now = chrono::Local::now();
            let formatted = now.format(format);
            out.write(&formatted.to_string())?;
            Ok(())
        }));

        // Helper for uppercase
        handlebars.register_helper("uppercase", Box::new(|h, _, _, _, out| {
            let text = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
            out.write(&text.to_uppercase())?;
            Ok(())
        }));

        // Helper for pluralization
        handlebars.register_helper("pluralize", Box::new(|h, _, _, _, out| {
            let count = h.param(0).and_then(|v| v.value().as_u64()).unwrap_or(0);
            let singular = h.param(1).and_then(|v| v.value().as_str()).unwrap_or("");
            let plural = h.param(2).and_then(|v| v.value().as_str()).unwrap_or(&format!("{}s", singular));
            
            let result = if count == 1 { singular } else { plural };
            out.write(result)?;
            Ok(())
        }));
    }

    fn create_built_in_templates() -> HashMap<String, Template> {
        let mut templates = HashMap::new();
        
        // Standard template
        templates.insert("standard".to_string(), Template {
            name: "standard".to_string(),
            description: "Standard PRISM analysis report".to_string(),
            content: include_str!("../templates/standard.hbs").to_string(),
            variables: vec![
                TemplateVariable {
                    name: "title".to_string(),
                    description: "Report title".to_string(),
                    default_value: Some("Requirements Analysis Report".to_string()),
                    required: false,
                },
            ],
        });

        // Enterprise template
        templates.insert("enterprise".to_string(), Template {
            name: "enterprise".to_string(),
            description: "Enterprise-grade report with executive summary".to_string(),
            content: include_str!("../templates/enterprise.hbs").to_string(),
            variables: vec![
                TemplateVariable {
                    name: "company_name".to_string(),
                    description: "Company name for branding".to_string(),
                    default_value: None,
                    required: true,
                },
                TemplateVariable {
                    name: "project_name".to_string(),
                    description: "Project name".to_string(),
                    default_value: None,
                    required: true,
                },
                TemplateVariable {
                    name: "stakeholder".to_string(),
                    description: "Primary stakeholder".to_string(),
                    default_value: None,
                    required: false,
                },
            ],
        });

        // Dashboard template
        templates.insert("dashboard".to_string(), Template {
            name: "dashboard".to_string(),
            description: "HTML dashboard with interactive elements".to_string(),
            content: include_str!("../templates/dashboard.hbs").to_string(),
            variables: vec![
                TemplateVariable {
                    name: "project_name".to_string(),
                    description: "Project name".to_string(),
                    default_value: Some("Requirements Analysis".to_string()),
                    required: false,
                },
            ],
        });

        templates
    }

    pub async fn load_custom_templates(&mut self) -> Result<()> {
        if let Some(ref template_dir) = self.template_dir {
            if template_dir.exists() {
                let mut entries = fs::read_dir(template_dir).await?;
                
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("hbs") {
                        let name = path.file_stem()
                            .and_then(|s| s.to_str())
                            .ok_or_else(|| anyhow!("Invalid template filename"))?;
                        
                        let content = fs::read_to_string(&path).await?;
                        
                        // Register template with Handlebars
                        self.handlebars.register_template_string(name, &content)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_template(&self, name: &str) -> Option<&Template> {
        self.built_in_templates.get(name)
    }

    pub fn list_templates(&self) -> Vec<String> {
        self.built_in_templates.keys().cloned().collect()
    }

    pub fn render(&self, template_name: &str, context: &TemplateContext, data: &serde_json::Value) -> Result<String> {
        // Create rendering context
        let mut render_data = serde_json::Map::new();
        
        // Add template context variables
        for (key, value) in &context.variables {
            render_data.insert(key.clone(), serde_json::Value::String(value.clone()));
        }
        
        // Add metadata
        render_data.insert("branding".to_string(), 
            context.branding.as_ref()
                .map(|b| serde_json::Value::String(b.clone()))
                .unwrap_or(serde_json::Value::Null));
        render_data.insert("timestamp".to_string(), serde_json::Value::String(context.timestamp.clone()));
        render_data.insert("version".to_string(), serde_json::Value::String(context.version.clone()));
        
        // Add analysis data
        if let serde_json::Value::Object(data_map) = data {
            for (key, value) in data_map {
                render_data.insert(key.clone(), value.clone());
            }
        }

        let render_context = serde_json::Value::Object(render_data);

        // Try built-in template first
        if let Some(template) = self.built_in_templates.get(template_name) {
            // Register the template if not already registered
            if !self.handlebars.has_template(template_name) {
                self.handlebars.register_template_string(template_name, &template.content)
                    .map_err(|e| anyhow!("Failed to register template '{}': {}", template_name, e))?;
            }
        }

        // Render the template
        self.handlebars.render(template_name, &render_context)
            .map_err(|e| anyhow!("Failed to render template '{}': {}", template_name, e))
    }

    pub fn create_context(&self, branding: Option<String>) -> TemplateContext {
        TemplateContext {
            variables: HashMap::new(),
            branding,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.0".to_string(),
        }
    }

    pub fn validate_template_variables(&self, template_name: &str, context: &TemplateContext) -> Result<()> {
        if let Some(template) = self.built_in_templates.get(template_name) {
            for var in &template.variables {
                if var.required && !context.variables.contains_key(&var.name) {
                    return Err(anyhow!("Required template variable '{}' is missing for template '{}'", 
                        var.name, template_name));
                }
            }
        }
        Ok(())
    }
}

// Create template directory structure
pub async fn create_template_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path).await?;
    
    // Create example templates
    let examples = [
        ("custom_report.hbs", include_str!("../templates/examples/custom_report.hbs")),
        ("executive_summary.hbs", include_str!("../templates/examples/executive_summary.hbs")),
    ];
    
    for (filename, content) in &examples {
        let file_path = path.join(filename);
        if !file_path.exists() {
            fs::write(&file_path, content).await?;
        }
    }
    
    // Create README
    let readme_path = path.join("README.md");
    if !readme_path.exists() {
        let readme_content = r#"# PRISM Custom Templates

This directory contains custom Handlebars templates for PRISM output formatting.

## Template Variables

Available variables in templates:
- `{{branding}}` - Custom branding text
- `{{timestamp}}` - Current timestamp
- `{{version}}` - PRISM version
- `{{ambiguities}}` - Array of detected ambiguities
- `{{entities}}` - Extracted entities (actors, actions, objects)
- `{{improved_requirements}}` - AI-improved requirements text
- `{{uml_diagrams}}` - Generated UML diagrams
- `{{test_cases}}` - Generated test cases

## Custom Variables

You can define custom variables in your templates and pass them via CLI:
```bash
prism analyze --template custom_report --branding "MyCompany Inc."
```

## Helpers

Available Handlebars helpers:
- `{{format_date "%Y-%m-%d"}}` - Format current date
- `{{uppercase text}}` - Convert to uppercase
- `{{pluralize count "item" "items"}}` - Pluralize based on count
"#;
        fs::write(&readme_path, readme_content).await?;
    }
    
    Ok(())
}