use anyhow::Result;
use std::path::PathBuf;
use std::io;
use tokio::fs;
use walkdir::WalkDir;

use crate::analyzer::{Analyzer, AnalysisResult};
use crate::cli::{Commands, OutputFormat, AnalysisPreset, GenerateOptions};
use crate::config::Config;
use crate::ui::TuiApp;
use crate::document_processor::DocumentProcessor;

pub struct App {
    pub config: Config,
    analyzer: Analyzer,
    document_processor: DocumentProcessor,
}

impl App {
    pub async fn new() -> Result<Self> {
        let config = Config::load().await?;
        let analyzer = Analyzer::new()?.with_config(config.clone());
        let document_processor = DocumentProcessor::new();

        Ok(Self { config, analyzer, document_processor })
    }

    fn print_branded_header(&self) {
        println!("🔍 PRISM - AI-Powered Requirement Analyzer");
        println!("===========================================");
    }

    fn resolve_generation_options(&self, preset: &Option<AnalysisPreset>, generate: &Vec<GenerateOptions>) -> (bool, bool, bool, bool, bool, bool, bool) {
        let mut uml = false;
        let mut pseudo = false; 
        let mut tests = false;
        let mut improve = false;
        let mut nfr = false;
        let mut completeness = false;
        let validate_story = false;

        // Apply preset first
        if let Some(preset) = preset {
            match preset {
                AnalysisPreset::Basic => {
                    // Just basic analysis - no additional features
                }
                AnalysisPreset::Standard => {
                    uml = true;
                    pseudo = true;
                    tests = true;
                }
                AnalysisPreset::Full => {
                    uml = true;
                    pseudo = true;
                    tests = true;
                    improve = true;
                    nfr = true;
                    completeness = true;
                }
                AnalysisPreset::Report => {
                    uml = true;
                    tests = true;
                    improve = true;
                    completeness = true;
                }
            }
        }

        // Apply individual generate options (override preset)
        for option in generate {
            match option {
                GenerateOptions::All => {
                    uml = true;
                    pseudo = true;
                    tests = true;
                    improve = true;
                    nfr = true;
                }
                GenerateOptions::Uml => uml = true,
                GenerateOptions::Pseudo => pseudo = true,
                GenerateOptions::Tests => tests = true,
                GenerateOptions::Improve => improve = true,
                GenerateOptions::Nfr => nfr = true,
            }
        }

        // Smart defaults: auto-enable tests when improve is used
        if improve && !tests {
            tests = true;
        }

        (uml, pseudo, tests, improve, nfr, completeness, validate_story)
    }

    pub async fn run_command(&mut self, command: Commands) -> Result<()> {
        match command {
            Commands::Analyze {
                text,
                file,
                dir,
                output,
                preset,
                generate,
                format,
                pseudo_lang,
                save_artifacts,
                template,
                branding,
                continue_on_error,
                skip_invalid,
                parallel,
            } => {
                self.print_branded_header();
                
                // Resolve preset and generate options into specific flags
                let (uml, pseudo, tests, improve, nfr, completeness, validate_story) = 
                    self.resolve_generation_options(&preset, &generate);
                
                // Handle batch processing (directory) differently
                if let Some(dir_path) = &dir {
                    return self.process_directory_batch(
                        dir_path, output, format, uml, pseudo, tests, improve, 
                        save_artifacts, completeness, validate_story, nfr, pseudo_lang
                    ).await;
                }
                
                let input_text = self.get_input_text(text, file, dir.clone()).await?;
                
                if self.config.is_ai_configured() {
                    let (provider_name, _) = self.config.get_provider_info();
                    println!("🤖 Analyzing your requirements with {} ({})...", provider_name, self.config.llm.model);
                } else {
                    println!("📋 Analyzing your requirements with built-in analysis...");
                }
                
                let mut result = self.analyzer.analyze(&input_text).await?;

                if uml {
                    println!("🎨 Generating UML diagrams...");
                    let use_case = self.analyzer.generate_uml_use_case(&result.entities);
                    let sequence = self.analyzer.generate_uml_sequence(&result.entities);
                    let class_diagram = self.analyzer.generate_uml_class_diagram(&result.entities);
                    result.uml_diagrams = Some(crate::analyzer::UmlDiagrams {
                        use_case: Some(use_case),
                        sequence: Some(sequence),
                        class_diagram: Some(class_diagram),
                    });
                }

                if pseudo {
                    println!("📝 Generating pseudocode structure...");
                    let pseudocode = self.analyzer.generate_pseudocode(&result.entities, pseudo_lang.as_deref());
                    result.pseudocode = Some(pseudocode);
                }

                if tests {
                    println!("🧪 Generating test cases...");
                    let test_cases = self.analyzer.generate_test_cases(&result.entities);
                    result.test_cases = Some(test_cases);
                }

                if improve {
                    println!("✨ Generating improved requirements...");
                    match self.analyzer.generate_improved_requirements(&input_text, &result.ambiguities).await {
                        Ok(improved) => {
                            result.improved_requirements = Some(improved);
                            println!("✅ Requirements improvement completed!");
                        }
                        Err(e) => {
                            eprintln!("⚠️  Failed to generate improved requirements: {}", e);
                            eprintln!("   Continuing with analysis results only");
                        }
                    }
                }

                // New features processing
                if completeness {
                    println!("📊 Analyzing completeness and identifying gaps...");
                    let completeness_analysis = self.analyzer.analyze_completeness(&input_text, &result.entities).await?;
                    result.completeness_analysis = Some(completeness_analysis);
                }

                if validate_story {
                    println!("✅ Validating user story format and business value...");
                    let user_story_validation = self.analyzer.validate_user_story(&input_text);
                    result.user_story_validation = Some(user_story_validation);
                }

                if nfr {
                    println!("🔒 Generating non-functional requirement suggestions...");
                    let nfr_suggestions = self.analyzer.generate_nfr_suggestions(&input_text, &result.entities).await?;
                    result.nfr_suggestions = Some(nfr_suggestions);
                }

                println!("✅ Analysis completed successfully!");
                
                let mut files_saved = false;
                
                // Save individual artifacts if requested (not available for directory processing)
                if let Some(base_filename) = save_artifacts {
                    if dir.is_none() {
                        // Only save individual artifacts for single file or text analysis
                        self.save_individual_artifacts(&result, &base_filename, &input_text).await?;
                        files_saved = true;
                    } else {
                        println!("💡 Skipping individual artifacts for batch processing. Use single file analysis with --save-artifacts to generate individual files.");
                    }
                }
                
                // Save main output file or display to screen
                if let Some(output_path) = output {
                    // Always save main output when --output is specified
                    let format_to_use = format.unwrap_or(OutputFormat::Json);
                    let output_content = match format_to_use {
                        OutputFormat::Json => serde_json::to_string_pretty(&result)?,
                        OutputFormat::Markdown => self.format_as_markdown(&result, &input_text),
                        OutputFormat::Jira => self.format_as_jira(&result, &input_text),
                        OutputFormat::Github => self.format_as_github(&result, &input_text),
                        OutputFormat::Plain => self.format_as_plain(&result, &input_text),
                    };
                    
                    let absolute_path = std::fs::canonicalize(&output_path).unwrap_or(output_path.clone());
                    fs::write(&output_path, output_content).await?;
                    println!("📁 Analysis report saved: {}", absolute_path.display());
                    files_saved = true;
                } else if !files_saved {
                    // Only display to screen if no files were saved
                    self.display_result_to_screen(&result, format.unwrap_or(OutputFormat::Json), &input_text).await?;
                }
                
                if files_saved {
                    println!("🎉 Analysis complete! Review the saved files for detailed insights and recommendations.");
                }
            }
            Commands::Tui => {
                self.run_tui().await?;
            }
            Commands::Improve { text, file, dir, output, format } => {
                self.print_branded_header();
                let input_text = self.get_input_text(text, file, dir.clone()).await?;
                
                if self.config.is_ai_configured() {
                    let (provider_name, _) = self.config.get_provider_info();
                    println!("🤖 Analyzing your requirements with {} ({})...", provider_name, self.config.llm.model);
                } else {
                    println!("❌ AI configuration required for requirement improvement!");
                    println!("💡 Run 'prism config --setup' to configure AI features");
                    return Ok(());
                }
                
                // First analyze to find issues
                let analysis_result = self.analyzer.analyze(&input_text).await?;
                
                if analysis_result.ambiguities.is_empty() {
                    println!("✅ No ambiguities found - requirements are already clear!");
                    if let Some(output_path) = output {
                        let absolute_path = std::fs::canonicalize(&output_path).unwrap_or(output_path.clone());
                        fs::write(&output_path, &input_text).await?;
                        println!("📁 Original requirements saved: {} (no changes needed)", absolute_path.display());
                    } else {
                        println!("\nOriginal Requirements:\n{}", input_text);
                    }
                    return Ok(());
                }
                
                // Generate improved requirements
                println!("✨ Generating improved requirements...");
                match self.analyzer.generate_improved_requirements(&input_text, &analysis_result.ambiguities).await {
                    Ok(improved) => {
                        if let Some(output_path) = output {
                            let final_output = match format.unwrap_or(OutputFormat::Markdown) {
                                OutputFormat::Markdown => self.format_improvement_as_markdown(&input_text, &improved, &analysis_result.ambiguities),
                                _ => improved,
                            };
                            let absolute_path = std::fs::canonicalize(&output_path).unwrap_or(output_path.clone());
                            fs::write(&output_path, final_output).await?;
                            println!("📁 Improved requirements created and saved: {}", absolute_path.display());
                            println!("🎉 Analysis complete! Your requirements have been enhanced with specific, measurable criteria.");
                        } else {
                            match format.unwrap_or(OutputFormat::Markdown) {
                                OutputFormat::Markdown => {
                                    println!("{}", self.format_improvement_as_markdown(&input_text, &improved, &analysis_result.ambiguities));
                                }
                                OutputFormat::Json => {
                                    let mut result = analysis_result;
                                    result.improved_requirements = Some(improved);
                                    println!("{}", serde_json::to_string_pretty(&result)?);
                                }
                                _ => {
                                    println!("# Improved Requirements\n\n{}", improved);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to generate improved requirements: {}", e);
                        return Err(e);
                    }
                }
            }
            Commands::Config { 
                api_key, 
                model, 
                provider, 
                setup, 
                show, 
                debug, 
                test,
                validate_all,
                test_providers,
                set_template_dir,
            } => {
                if debug {
                    let config_path = Config::config_path()?;
                    println!("Configuration file path: {:?}", config_path);
                    println!("Config directory exists: {}", config_path.parent().map_or(false, |p| p.exists()));
                    println!("Config file exists: {}", config_path.exists());
                    
                    if config_path.exists() {
                        match fs::read_to_string(&config_path).await {
                            Ok(content) => {
                                println!("Config file size: {} bytes", content.len());
                                println!("Config file content:");
                                println!("{}", content);
                            }
                            Err(e) => {
                                println!("Error reading config file: {}", e);
                            }
                        }
                    } else {
                        println!("Config file does not exist. Creating default config...");
                        self.config.save().await?;
                        println!("Default config created at: {:?}", config_path);
                    }
                    return Ok(());
                }
                
                if show {
                    self.show_config_status();
                    return Ok(());
                }

                if test {
                    self.test_ai_configuration().await?;
                    return Ok(());
                }

                // Interactive setup wizard
                if setup {
                    self.run_setup_wizard().await?;
                    return Ok(());
                }

                // Manual configuration
                let mut updated = false;
                
                if let Some(ai_provider) = provider {
                    let provider_str = match ai_provider {
                        crate::cli::AiProvider::OpenAI => "openai",
                        crate::cli::AiProvider::Gemini => "gemini", 
                        crate::cli::AiProvider::Claude => "claude",
                        crate::cli::AiProvider::Azure => "azure",
                        crate::cli::AiProvider::Ollama => "ollama",
                    };
                    self.config.set_provider(provider_str);
                    updated = true;
                    
                    // If no other parameters provided, run interactive setup
                    if api_key.is_none() && model.is_none() {
                        self.setup_provider(ai_provider).await?;
                        return Ok(());
                    }
                }
                
                if let Some(key) = api_key {
                    self.config.set_api_key(key);
                    updated = true;
                }

                if let Some(model_name) = model {
                    self.config.set_model(model_name);
                    updated = true;
                }

                // Handle new config validation options
                if validate_all {
                    println!("🔍 Validating configuration...");
                    match self.config.validate_all_settings().await {
                        Ok(result) => {
                            if result.is_valid {
                                println!("✅ Configuration is valid!");
                            } else {
                                println!("❌ Configuration issues found:");
                                for issue in result.issues {
                                    println!("   • {}", issue);
                                }
                            }
                            if !result.warnings.is_empty() {
                                println!("⚠️  Warnings:");
                                for warning in result.warnings {
                                    println!("   • {}", warning);
                                }
                            }
                        }
                        Err(e) => println!("❌ Validation failed: {}", e),
                    }
                    return Ok(());
                }

                if test_providers {
                    println!("🧪 Testing all AI providers...");
                    match self.config.test_all_providers().await {
                        Ok(results) => {
                            println!("{}", results.get_summary());
                            for (provider, result) in results.results {
                                let status = if result.success { "✅" } else { "❌" };
                                let time_str = if let Some(time) = result.response_time {
                                    format!(" ({}ms)", time)
                                } else {
                                    String::new()
                                };
                                println!("{} {}: {}{}", status, provider, result.message, time_str);
                            }
                        }
                        Err(e) => println!("❌ Provider testing failed: {}", e),
                    }
                    return Ok(());
                }

                if let Some(template_dir) = set_template_dir {
                    println!("📁 Template directory feature: {}", template_dir.display());
                    println!("✅ Template directory feature implemented (placeholder)");
                    return Ok(());
                }

                if updated {
                    self.config.save().await?;
                    println!("✅ Configuration updated successfully!");
                    self.show_config_status();
                } else if !validate_all && !test_providers && set_template_dir.is_none() {
                    println!("🔧 No configuration changes specified. Use --help for options or --setup for interactive configuration.");
                }
            }
            Commands::Validate { text, file, dir, output, story, completeness, all, format } => {
                self.print_branded_header();
                let input_text = self.get_input_text(text, file, dir.clone()).await?;
                
                println!("✅ Running validation checks...");
                
                let mut result = self.analyzer.analyze(&input_text).await?;
                
                if story || all {
                    println!("📋 Validating user story format and business value...");
                    let user_story_validation = self.analyzer.validate_user_story(&input_text);
                    result.user_story_validation = Some(user_story_validation);
                }
                
                if completeness || all {
                    println!("📊 Analyzing completeness and identifying gaps...");
                    let completeness_analysis = self.analyzer.analyze_completeness(&input_text, &result.entities).await?;
                    result.completeness_analysis = Some(completeness_analysis);
                }
                
                if let Some(output_path) = output {
                    let format_to_use = format.unwrap_or(OutputFormat::Json);
                    let output_content = match format_to_use {
                        OutputFormat::Json => serde_json::to_string_pretty(&result)?,
                        OutputFormat::Markdown => self.format_as_markdown(&result, &input_text),
                        OutputFormat::Jira => self.format_as_jira(&result, &input_text),
                        OutputFormat::Github => self.format_as_github(&result, &input_text),
                        OutputFormat::Plain => self.format_as_plain(&result, &input_text),
                    };
                    
                    let absolute_path = std::fs::canonicalize(&output_path).unwrap_or(output_path.clone());
                    fs::write(&output_path, output_content).await?;
                    println!("📁 Validation report saved: {}", absolute_path.display());
                } else {
                    self.display_result_to_screen(&result, format.unwrap_or(OutputFormat::Json), &input_text).await?;
                }
            }
            Commands::Trace { text, file, output, from_commit, to_commit, source_dir, test_dir, format } => {
                self.print_branded_header();
                
                println!("🔍 Tracing requirements to implementation...");
                
                if let (Some(from), Some(to)) = (&from_commit, &to_commit) {
                    println!("📈 Git diff analysis from {} to {}", from, to);
                    println!("⚠️  Git traceability feature coming soon!");
                } else if let (Some(src), Some(test)) = (&source_dir, &test_dir) {
                    println!("📁 Scanning source directory: {:?}", src);
                    println!("🧪 Scanning test directory: {:?}", test);
                    println!("⚠️  File traceability feature coming soon!");
                } else {
                    println!("❌ Please specify either git commits (--from-commit and --to-commit) or directories (--source-dir and --test-dir)");
                }
            }
            Commands::Dashboard { text, file, dir, output, template, branding, executive_summary } => {
                self.print_branded_header();
                
                let input_text = self.get_input_text(text, file, dir.clone()).await?;
                
                println!("📊 Generating dashboard and reports...");
                
                let mut result = self.analyzer.analyze(&input_text).await?;
                
                result.uml_diagrams = Some(crate::analyzer::UmlDiagrams {
                    use_case: Some(self.analyzer.generate_uml_use_case(&result.entities)),
                    sequence: Some(self.analyzer.generate_uml_sequence(&result.entities)),
                    class_diagram: Some(self.analyzer.generate_uml_class_diagram(&result.entities)),
                });
                
                result.test_cases = Some(self.analyzer.generate_test_cases(&result.entities));
                
                if executive_summary {
                    println!("📈 Generating executive summary...");
                }
                
                if let Some(output_path) = output {
                    println!("📁 Dashboard will be saved to: {:?}", output_path);
                    println!("⚠️  HTML dashboard generation coming soon!");
                } else {
                    println!("📊 Dashboard generation requires --output parameter");
                }
            }
        }

        Ok(())
    }

    pub async fn run_tui(&mut self) -> Result<()> {
        // Check if AI is configured, if not, prompt user for setup
        if !self.config.is_ai_configured() {
            println!("🔍 Welcome to PRISM - AI-Powered Requirement Analyzer!");
            println!("====================================================");
            println!("This is your first time using PRISM or AI is not configured.");
            println!("PRISM works best with AI providers for enhanced analysis.\n");
            
            println!("Would you like to configure AI now for better results? (y/n): ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            if input.trim().to_lowercase() == "y" {
                self.run_setup_wizard().await?;
                println!("\n🎯 Starting PRISM TUI...");
            } else {
                println!("📝 You can configure AI later with: prism config --setup");
                println!("🎯 Starting PRISM TUI with built-in analysis...");
            }
        }
        
        let mut tui_app = TuiApp::new(self.analyzer.clone(), self.config.clone())?;
        tui_app.run().await
    }

    async fn get_input_text(
        &self,
        text: Option<String>,
        file: Option<PathBuf>,
        dir: Option<PathBuf>,
    ) -> Result<String> {
        if let Some(text) = text {
            return Ok(text);
        }

        if let Some(file_path) = file {
            return self.read_file(&file_path).await;
        }

        if let Some(dir_path) = dir {
            return self.read_directory(&dir_path).await;
        }

        Err(anyhow::anyhow!("No input provided. Use --text, --file, or --dir"))
    }

    async fn read_file(&self, path: &PathBuf) -> Result<String> {
        if !path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {:?}", path));
        }

        println!("📖 Reading requirements from: {}", path.display());
        
        // Use document processor for all file types
        let content = self.document_processor.extract_text_from_file(path).await?;
        
        println!("📄 Loaded {} characters from file", content.len());
        Ok(content)
    }

    async fn read_directory(&self, path: &PathBuf) -> Result<String> {
        if !path.exists() || !path.is_dir() {
            return Err(anyhow::anyhow!("Directory does not exist: {:?}", path));
        }

        println!("📁 Scanning directory: {}", path.display());
        let mut combined_content = String::new();
        let mut file_count = 0;

        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && self.document_processor.is_supported_format(path) {
                match self.document_processor.extract_text_from_file(path).await {
                    Ok(content) => {
                        println!("  📖 Reading: {}", path.display());
                        combined_content.push_str(&format!("=== {} ===\n", path.display()));
                        combined_content.push_str(&content);
                        combined_content.push_str("\n\n");
                        file_count += 1;
                    }
                    Err(e) => {
                        eprintln!("⚠️  Could not read file {:?}: {}", path, e);
                    }
                }
            }
        }

        println!("📊 Loaded {} files with {} total characters", file_count, combined_content.len());

        if combined_content.is_empty() {
            return Err(anyhow::anyhow!("No readable files (.md, .txt, .rst) found in directory"));
        }

        Ok(combined_content)
    }

    async fn display_result_to_screen(
        &self,
        result: &AnalysisResult,
        format: OutputFormat,
        input_text: &str,
    ) -> Result<()> {
        let output_content = match format {
            OutputFormat::Json => serde_json::to_string_pretty(result)?,
            OutputFormat::Markdown => self.format_as_markdown(result, input_text),
            OutputFormat::Jira => self.format_as_jira(result, input_text),
            OutputFormat::Github => self.format_as_github(result, input_text),
            OutputFormat::Plain => self.format_as_plain(result, input_text),
        };

        println!("{}", output_content);
        Ok(())
    }

    fn format_as_markdown(&self, result: &AnalysisResult, input_text: &str) -> String {
        let mut output = String::new();
        
        output.push_str("# 🔍 PRISM Requirement Analysis Report\n\n");

        output.push_str("## 📝 Analyzed Requirement\n\n");
        output.push_str(&format!("> {}\n\n", input_text.trim()));

        output.push_str("## 📊 Analysis Summary\n\n");
        output.push_str(&format!("- **Ambiguities Found:** {}\n", result.ambiguities.len()));
        output.push_str(&format!("- **Actors Identified:** {}\n", result.entities.actors.len()));
        output.push_str(&format!("- **Actions Identified:** {}\n", result.entities.actions.len()));
        output.push_str(&format!("- **Objects Identified:** {}\n\n", result.entities.objects.len()));

        output.push_str("## ⚠️ Detected Ambiguities\n\n");
        if result.ambiguities.is_empty() {
            output.push_str("✅ **No ambiguities detected - your requirements are clear!**\n\n");
        } else {
            for (i, ambiguity) in result.ambiguities.iter().enumerate() {
                let severity_icon = match ambiguity.severity {
                    crate::analyzer::AmbiguitySeverity::Critical => "🔴",
                    crate::analyzer::AmbiguitySeverity::High => "🟠",
                    crate::analyzer::AmbiguitySeverity::Medium => "🟡",
                    crate::analyzer::AmbiguitySeverity::Low => "🟢",
                };
                output.push_str(&format!("### {} Issue #{}: \"{}\"\n", severity_icon, i + 1, ambiguity.text));
                output.push_str(&format!("- **Problem:** {}\n", ambiguity.reason));
                output.push_str(&format!("- **Severity:** {:?}\n", ambiguity.severity));
                output.push_str("- **Suggested Improvements:**\n");
                for suggestion in &ambiguity.suggestions {
                    output.push_str(&format!("  - {}\n", suggestion));
                }
                output.push('\n');
            }
        }

        output.push_str("## 🎯 Extracted Entities\n\n");
        
        output.push_str("### 👥 Actors (Who performs actions)\n");
        if result.entities.actors.is_empty() {
            output.push_str("- *No actors identified*\n\n");
        } else {
            for actor in &result.entities.actors {
                output.push_str(&format!("- **{}**\n", actor));
            }
            output.push('\n');
        }
        
        output.push_str("### ⚡ Actions (What is being done)\n");
        if result.entities.actions.is_empty() {
            output.push_str("- *No actions identified*\n\n");
        } else {
            for action in &result.entities.actions {
                output.push_str(&format!("- **{}**\n", action));
            }
            output.push('\n');
        }
        
        output.push_str("### 📦 Objects (What is being acted upon)\n");
        if result.entities.objects.is_empty() {
            output.push_str("- *No objects identified*\n\n");
        } else {
            for object in &result.entities.objects {
                output.push_str(&format!("- **{}**\n", object));
            }
            output.push('\n');
        }

        if let Some(uml) = &result.uml_diagrams {
            output.push_str("## 🎨 UML Diagrams\n\n");
            
            if let Some(use_case) = &uml.use_case {
                output.push_str("### Use Case Diagram\n\n");
                output.push_str("```plantuml\n");
                output.push_str(use_case);
                output.push_str("\n```\n\n");
            }
            
            if let Some(sequence) = &uml.sequence {
                output.push_str("### Sequence Diagram\n\n");
                output.push_str("```plantuml\n");
                output.push_str(sequence);
                output.push_str("\n```\n\n");
            }
            
            if let Some(class_diagram) = &uml.class_diagram {
                output.push_str("### Class Diagram\n\n");
                output.push_str("```plantuml\n");
                output.push_str(class_diagram);
                output.push_str("\n```\n\n");
            }
        }

        if let Some(pseudocode) = &result.pseudocode {
            output.push_str("## Generated Pseudocode\n\n");
            output.push_str("```\n");
            output.push_str(pseudocode);
            output.push_str("\n```\n\n");
        }

        if let Some(tests) = &result.test_cases {
            output.push_str("## Suggested Test Cases\n\n");
            output.push_str("### Happy Path\n");
            for test in &tests.happy_path {
                output.push_str(&format!("- {}\n", test));
            }
            output.push_str("\n### Negative Cases\n");
            for test in &tests.negative_cases {
                output.push_str(&format!("- {}\n", test));
            }
            output.push_str("\n### Edge Cases\n");
            for test in &tests.edge_cases {
                output.push_str(&format!("- {}\n", test));
            }
        }

        if let Some(improved) = &result.improved_requirements {
            output.push_str("## ✨ Improved Requirements\n\n");
            output.push_str("```\n");
            output.push_str(improved);
            output.push_str("\n```\n\n");
        }

        if let Some(completeness) = &result.completeness_analysis {
            output.push_str("## 📊 Completeness Analysis\n\n");
            output.push_str(&format!("**Completeness Score: {:.1}%**\n\n", completeness.completeness_score));
            
            if !completeness.gaps_identified.is_empty() {
                output.push_str("### Identified Gaps\n\n");
                for gap in &completeness.gaps_identified {
                    let priority_emoji = match gap.priority {
                        crate::analyzer::GapPriority::Critical => "🔴",
                        crate::analyzer::GapPriority::High => "🟠", 
                        crate::analyzer::GapPriority::Medium => "🟡",
                        crate::analyzer::GapPriority::Low => "🟢",
                    };
                    output.push_str(&format!("#### {} {} - {:?}\n\n", priority_emoji, gap.category, gap.priority));
                    output.push_str(&format!("**Issue:** {}\n\n", gap.description));
                    output.push_str("**Suggestions:**\n");
                    for suggestion in &gap.suggestions {
                        output.push_str(&format!("- {}\n", suggestion));
                    }
                    output.push_str("\n");
                }
            }
        }

        if let Some(user_story) = &result.user_story_validation {
            output.push_str("## ✅ User Story Validation\n\n");
            if user_story.is_valid_format {
                output.push_str("✅ **Valid user story format detected**\n\n");
                output.push_str(&format!("**Business Value Score: {:.1}%**\n\n", user_story.business_value_score));
                
                output.push_str("### Component Analysis\n\n");
                output.push_str(&format!("**Actor Quality:** {:.1}% - {}\n", user_story.actor_quality.score,
                    if user_story.actor_quality.is_valid { "✅ Valid" } else { "❌ Issues found" }));
                output.push_str(&format!("**Goal Quality:** {:.1}% - {}\n", user_story.goal_quality.score,
                    if user_story.goal_quality.is_valid { "✅ Valid" } else { "❌ Issues found" }));
                output.push_str(&format!("**Reason Quality:** {:.1}% - {}\n\n", user_story.reason_quality.score,
                    if user_story.reason_quality.is_valid { "✅ Valid" } else { "❌ Issues found" }));
            } else {
                output.push_str("❌ **Not in valid user story format**\n\n");
            }
            
            if !user_story.recommendations.is_empty() {
                output.push_str("### Recommendations\n\n");
                for rec in &user_story.recommendations {
                    output.push_str(&format!("- {}\n", rec));
                }
                output.push_str("\n");
            }
        }

        if let Some(nfrs) = &result.nfr_suggestions {
            output.push_str("## 🔒 Non-Functional Requirements\n\n");
            let mut categories = std::collections::BTreeMap::new();
            
            // Group NFRs by category
            for nfr in nfrs {
                categories.entry(&nfr.category).or_insert(Vec::new()).push(nfr);
            }
            
            for (category, category_nfrs) in categories {
                let category_emoji = match category {
                    crate::analyzer::NfrCategory::Performance => "⚡",
                    crate::analyzer::NfrCategory::Security => "🔒",
                    crate::analyzer::NfrCategory::Usability => "👤",
                    crate::analyzer::NfrCategory::Reliability => "🛡️",
                    crate::analyzer::NfrCategory::Scalability => "📈",
                    crate::analyzer::NfrCategory::Maintainability => "🔧",
                    crate::analyzer::NfrCategory::Compatibility => "🔗",
                    crate::analyzer::NfrCategory::Accessibility => "♿",
                };
                output.push_str(&format!("### {} {:?}\n\n", category_emoji, category));
                
                for nfr in category_nfrs {
                    let priority_text = match nfr.priority {
                        crate::analyzer::NfrPriority::MustHave => "🔴 Must Have",
                        crate::analyzer::NfrPriority::ShouldHave => "🟠 Should Have",
                        crate::analyzer::NfrPriority::CouldHave => "🟡 Could Have",
                        crate::analyzer::NfrPriority::WontHave => "⚫ Won't Have",
                    };
                    output.push_str(&format!("**{}**\n\n", priority_text));
                    output.push_str(&format!("**Requirement:** {}\n\n", nfr.requirement));
                    output.push_str(&format!("**Rationale:** {}\n\n", nfr.rationale));
                    
                    if !nfr.acceptance_criteria.is_empty() {
                        output.push_str("**Acceptance Criteria:**\n");
                        for criteria in &nfr.acceptance_criteria {
                            output.push_str(&format!("- {}\n", criteria));
                        }
                        output.push_str("\n");
                    }
                }
            }
        }

        output
    }

    fn format_as_jira(&self, result: &AnalysisResult, input_text: &str) -> String {
        let mut output = String::new();
        
        output.push_str("h1. 🔍 PRISM Analysis Report\n\n");

        // Input echo section
        output.push_str("h2. 📝 Analyzed Requirement\n");
        output.push_str(&format!("{{quote}}\n{}\n{{quote}}\n\n", input_text.trim()));

        // Summary section
        output.push_str("h2. 📊 Analysis Summary\n");
        output.push_str(&format!("* Ambiguities Found: {}\n", result.ambiguities.len()));
        output.push_str(&format!("* Actors Identified: {}\n", result.entities.actors.len()));
        output.push_str(&format!("* Actions Identified: {}\n", result.entities.actions.len()));
        output.push_str(&format!("* Objects Identified: {}\n", result.entities.objects.len()));
        output.push_str("\n");

        // Entities section
        output.push_str("h2. 🎯 Extracted Entities\n");
        output.push_str("h3. 👥 Actors (Who)\n");
        if result.entities.actors.is_empty() {
            output.push_str("* No actors identified\n");
        } else {
            for actor in &result.entities.actors {
                output.push_str(&format!("* {}\n", actor));
            }
        }
        
        output.push_str("\nh3. ⚡ Actions (What)\n");
        if result.entities.actions.is_empty() {
            output.push_str("* No actions identified\n");
        } else {
            for action in &result.entities.actions {
                output.push_str(&format!("* {}\n", action));
            }
        }
        
        output.push_str("\nh3. 📦 Objects (What On)\n");
        if result.entities.objects.is_empty() {
            output.push_str("* No objects identified\n");
        } else {
            for object in &result.entities.objects {
                output.push_str(&format!("* {}\n", object));
            }
        }
        output.push_str("\n");

        // Ambiguities section
        output.push_str("h2. ⚠️ Detected Ambiguities\n");
        if result.ambiguities.is_empty() {
            output.push_str("✅ *No ambiguities detected - your requirements are clear!*\n\n");
        } else {
            for (i, ambiguity) in result.ambiguities.iter().enumerate() {
                let severity_icon = match ambiguity.severity {
                    crate::analyzer::AmbiguitySeverity::Critical => "🔴",
                    crate::analyzer::AmbiguitySeverity::High => "🟠", 
                    crate::analyzer::AmbiguitySeverity::Medium => "🟡",
                    crate::analyzer::AmbiguitySeverity::Low => "🟢",
                };
                output.push_str(&format!("h3. {} Issue #{}: \"{}\"\n", severity_icon, i + 1, ambiguity.text));
                output.push_str(&format!("* *Problem:* {}\n", ambiguity.reason));
                output.push_str(&format!("* *Severity:* {:?}\n", ambiguity.severity));
                output.push_str("* *Suggested Improvements:*\n");
                for suggestion in &ambiguity.suggestions {
                    output.push_str(&format!("** {}\n", suggestion));
                }
                output.push('\n');
            }
        }

        // Test cases section (only if generated)
        if let Some(tests) = &result.test_cases {
            output.push_str("h2. ✅ Suggested Test Cases\n");
            output.push_str("h3. 😊 Happy Path Tests\n");
            if tests.happy_path.is_empty() {
                output.push_str("* No happy path tests generated\n");
            } else {
                for test in &tests.happy_path {
                    output.push_str(&format!("- [ ] {}\n", test));
                }
            }
            
            output.push_str("\nh3. ❌ Negative Test Cases\n");
            if tests.negative_cases.is_empty() {
                output.push_str("* No negative test cases generated\n");
            } else {
                for test in &tests.negative_cases {
                    output.push_str(&format!("- [ ] {}\n", test));
                }
            }
            
            output.push_str("\nh3. 🔍 Edge Case Tests\n");
            if tests.edge_cases.is_empty() {
                output.push_str("* No edge case tests generated\n");
            } else {
                for test in &tests.edge_cases {
                    output.push_str(&format!("- [ ] {}\n", test));
                }
            }
        }

        output
    }

    fn format_as_github(&self, result: &AnalysisResult, input_text: &str) -> String {
        let mut output = String::new();
        
        output.push_str("# Requirement Analysis Report\n\n");

        if !result.ambiguities.is_empty() {
            output.push_str("## :warning: Detected Ambiguities\n\n");
            for ambiguity in &result.ambiguities {
                let emoji = match ambiguity.severity {
                    crate::analyzer::AmbiguitySeverity::Critical => ":red_circle:",
                    crate::analyzer::AmbiguitySeverity::High => ":orange_circle:",
                    crate::analyzer::AmbiguitySeverity::Medium => ":yellow_circle:",
                    crate::analyzer::AmbiguitySeverity::Low => ":green_circle:",
                };
                output.push_str(&format!("### {} {}\n", emoji, ambiguity.text));
                output.push_str(&format!("**Reason:** {}\n\n", ambiguity.reason));
                output.push_str("**Suggestions:**\n");
                for suggestion in &ambiguity.suggestions {
                    output.push_str(&format!("- {}\n", suggestion));
                }
                output.push('\n');
            }
        }

        output.push_str("## :mag: Extracted Entities\n\n");
        output.push_str(&format!("**:bust_in_silhouette: Actors:** {}\n\n", result.entities.actors.join(", ")));
        output.push_str(&format!("**:zap: Actions:** {}\n\n", result.entities.actions.join(", ")));
        output.push_str(&format!("**:package: Objects:** {}\n\n", result.entities.objects.join(", ")));

        if let Some(tests) = &result.test_cases {
            output.push_str("## :white_check_mark: Test Cases Checklist\n\n");
            output.push_str("### Happy Path\n");
            for test in &tests.happy_path {
                output.push_str(&format!("- [ ] {}\n", test));
            }
            output.push_str("\n### Negative Cases\n");
            for test in &tests.negative_cases {
                output.push_str(&format!("- [ ] {}\n", test));
            }
            output.push_str("\n### Edge Cases\n");
            for test in &tests.edge_cases {
                output.push_str(&format!("- [ ] {}\n", test));
            }
        }

        output
    }

    fn format_as_plain(&self, result: &AnalysisResult, input_text: &str) -> String {
        let mut output = String::new();
        
        output.push_str("REQUIREMENT ANALYSIS REPORT\n");
        output.push_str("===========================\n\n");

        output.push_str("DETECTED AMBIGUITIES:\n");
        for (i, ambiguity) in result.ambiguities.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, ambiguity.text));
            output.push_str(&format!("   Reason: {}\n", ambiguity.reason));
            output.push_str(&format!("   Severity: {:?}\n", ambiguity.severity));
            output.push_str("   Suggestions:\n");
            for suggestion in &ambiguity.suggestions {
                output.push_str(&format!("   - {}\n", suggestion));
            }
            output.push('\n');
        }

        output.push_str("EXTRACTED ENTITIES:\n");
        output.push_str(&format!("Actors: {}\n", result.entities.actors.join(", ")));
        output.push_str(&format!("Actions: {}\n", result.entities.actions.join(", ")));
        output.push_str(&format!("Objects: {}\n\n", result.entities.objects.join(", ")));

        if let Some(tests) = &result.test_cases {
            output.push_str("SUGGESTED TEST CASES:\n");
            output.push_str("Happy Path:\n");
            for test in &tests.happy_path {
                output.push_str(&format!("- {}\n", test));
            }
            output.push_str("\nNegative Cases:\n");
            for test in &tests.negative_cases {
                output.push_str(&format!("- {}\n", test));
            }
            output.push_str("\nEdge Cases:\n");
            for test in &tests.edge_cases {
                output.push_str(&format!("- {}\n", test));
            }
        }

        output
    }

    fn show_config_status(&self) {
        println!("🔧 Current PRISM Configuration");
        println!("============================");
        
        let (provider_name, models) = self.config.get_provider_info();
        println!("📡 AI Provider: {}", provider_name);
        
        if self.config.is_ai_configured() {
            println!("🔑 API Key: Configured ✅");
            println!("🤖 Model: {}", self.config.llm.model);
            if let Some(url) = &self.config.llm.base_url {
                println!("🌐 Base URL: {}", url);
            }
            println!("⏱️  Timeout: {}s", self.config.llm.timeout);
            println!("\n✅ AI features are ready to use!");
        } else {
            println!("🔑 API Key: Not configured ❌");
            println!("🤖 Model: {}", if self.config.llm.model.is_empty() { "Not set" } else { &self.config.llm.model });
            println!("\n⚠️  AI features are disabled. Run 'prism config --setup' to configure.");
        }
        
        println!("\n📝 Analysis Settings:");
        println!("  • Ambiguity threshold: {}", self.config.analysis.ambiguity_threshold);
        println!("  • Interactive mode: {}", self.config.analysis.enable_interactive);
        println!("  • Custom rules: {}", self.config.analysis.custom_rules.len());
    }

    pub async fn run_setup_wizard(&mut self) -> Result<()> {
        println!("🚀 PRISM AI Configuration Wizard");
        println!("=================================");
        println!("PRISM is designed to work with AI providers for enhanced requirement analysis.");
        println!("Without AI configuration, you'll only get basic built-in analysis.\n");

        println!("Would you like to configure AI analysis? (y/n): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() != "y" {
            println!("📝 Skipping AI configuration. You can run 'prism config --setup' anytime to configure later.");
            println!("✨ PRISM will use built-in analysis features only.");
            return Ok(());
        }

        println!("\n🤖 Choose your AI provider:");
        println!("1. OpenAI (GPT-4, GPT-3.5-turbo, GPT-4o)");
        println!("2. Google Gemini (gemini-1.5-pro, gemini-1.5-flash)"); 
        println!("3. Anthropic Claude (claude-3-opus, claude-3-sonnet, claude-3-haiku)");
        println!("4. Azure OpenAI");
        println!("5. Local Ollama (llama2, codellama, mistral, etc.)");
        println!("\nEnter choice (1-5): ");
        
        input.clear();
        std::io::stdin().read_line(&mut input)?;
        
        let provider = match input.trim() {
            "1" => crate::cli::AiProvider::OpenAI,
            "2" => crate::cli::AiProvider::Gemini,
            "3" => crate::cli::AiProvider::Claude,
            "4" => crate::cli::AiProvider::Azure,
            "5" => crate::cli::AiProvider::Ollama,
            _ => {
                println!("❌ Invalid choice. Please run the wizard again.");
                return Ok(());
            }
        };

        self.setup_provider(provider).await?;
        Ok(())
    }

    async fn setup_provider(&mut self, provider: crate::cli::AiProvider) -> Result<()> {
        let provider_str = match provider {
            crate::cli::AiProvider::OpenAI => "openai",
            crate::cli::AiProvider::Gemini => "gemini", 
            crate::cli::AiProvider::Claude => "claude",
            crate::cli::AiProvider::Azure => "azure",
            crate::cli::AiProvider::Ollama => "ollama",
        };

        self.config.set_provider(provider_str);
        let (provider_name, models) = self.config.get_provider_info();

        println!("\n🔧 Configuring {} Provider", provider_name);
        println!("{}========================{}", "=".repeat(provider_name.len()), "=".repeat(9));

        // Get API key (not needed for Ollama)
        if !matches!(provider, crate::cli::AiProvider::Ollama) {
            println!("🔑 Enter your {} API key: ", provider_name);
            let mut api_key = String::new();
            std::io::stdin().read_line(&mut api_key)?;
            let api_key = api_key.trim().to_string();

            if api_key.is_empty() {
                println!("❌ API key cannot be empty. Configuration cancelled.");
                return Ok(());
            }

            self.config.set_api_key(api_key);
        } else {
            println!("ℹ️  Ollama runs locally - no API key required");
            // Set a placeholder API key for Ollama
            self.config.set_api_key("ollama-local".to_string());
        }

        // Get model selection
        println!("\n🤖 Available models for {}:", provider_name);
        for (i, model) in models.iter().enumerate() {
            println!("{}. {}", i + 1, model);
        }
        
        println!("Enter choice (1-{}) or custom model name: ", models.len());
        let mut model_input = String::new();
        std::io::stdin().read_line(&mut model_input)?;
        let model_input = model_input.trim();

        let selected_model = if let Ok(choice) = model_input.parse::<usize>() {
            if choice > 0 && choice <= models.len() {
                models[choice - 1].clone()
            } else {
                println!("❌ Invalid choice. Using default model.");
                models.first().unwrap_or(&"gpt-4".to_string()).clone()
            }
        } else {
            model_input.to_string()
        };

        self.config.set_model(selected_model.clone());

        // Special handling for Azure and Ollama
        if matches!(provider, crate::cli::AiProvider::Azure) {
            println!("\n🌐 Enter your Azure OpenAI endpoint URL:");
            println!("(e.g., https://your-resource.openai.azure.com/openai/deployments/your-deployment)");
            let mut url = String::new();
            std::io::stdin().read_line(&mut url)?;
            let url = url.trim();
            if !url.is_empty() {
                self.config.llm.base_url = Some(url.to_string());
            }
        } else if matches!(provider, crate::cli::AiProvider::Ollama) {
            println!("\n🌐 Enter your Ollama server URL (or press Enter for default http://localhost:11434):");
            let mut url = String::new();
            std::io::stdin().read_line(&mut url)?;
            let url = url.trim();
            if !url.is_empty() {
                self.config.llm.base_url = Some(format!("{}/api/generate", url));
            }
            // Default URL is already set in set_provider
        }

        // Save configuration
        self.config.save().await?;

        println!("\n✅ {} configuration completed successfully!", provider_name);
        println!("🤖 Model: {}", selected_model);
        if matches!(provider, crate::cli::AiProvider::Ollama) {
            println!("🔑 API Key: Not required (local)");
        } else {
            println!("🔑 API Key: Configured");
        }
        if let Some(url) = &self.config.llm.base_url {
            println!("🌐 Base URL: {}", url);
        }
        println!("\n🎉 PRISM is now ready for AI-powered analysis!");
        println!("💡 Try: prism analyze \"As a user, I want to login quickly\"");

        Ok(())
    }

    fn format_improvement_as_markdown(&self, original: &str, improved: &str, ambiguities: &[crate::analyzer::Ambiguity]) -> String {
        let mut output = String::new();
        
        output.push_str("# 🔍 PRISM Requirements Improvement Report\n\n");
        
        output.push_str("## 📝 Improved Requirements\n\n");
        output.push_str("```\n");
        output.push_str(improved);
        output.push_str("\n```\n\n");
        
        output.push_str("## 📊 Issues Fixed\n\n");
        output.push_str(&format!("**Total Issues Addressed:** {}\n\n", ambiguities.len()));
        
        for (i, ambiguity) in ambiguities.iter().enumerate() {
            let severity_icon = match ambiguity.severity {
                crate::analyzer::AmbiguitySeverity::Critical => "🔴",
                crate::analyzer::AmbiguitySeverity::High => "🟠",
                crate::analyzer::AmbiguitySeverity::Medium => "🟡",
                crate::analyzer::AmbiguitySeverity::Low => "🟢",
            };
            output.push_str(&format!("### {} Issue #{}: \"{}\"\n", severity_icon, i + 1, ambiguity.text));
            output.push_str(&format!("- **Problem:** {}\n", ambiguity.reason));
            output.push_str(&format!("- **Severity:** {:?}\n", ambiguity.severity));
            output.push_str("- **Applied Solutions:**\n");
            for suggestion in &ambiguity.suggestions {
                output.push_str(&format!("  - {}\n", suggestion));
            }
            output.push('\n');
        }
        
        output.push_str("## 📋 Original Requirements (For Reference)\n\n");
        output.push_str("<details>\n");
        output.push_str("<summary>Click to view original requirements</summary>\n\n");
        output.push_str("```\n");
        output.push_str(original);
        output.push_str("\n```\n\n");
        output.push_str("</details>\n\n");
        
        output.push_str("---\n");
        output.push_str("*Generated by PRISM - AI-Powered Requirement Analyzer* 🔍✨\n");
        
        output
    }

    async fn test_ai_configuration(&mut self) -> Result<()> {
        println!("🧪 Testing AI Configuration...\n");
        
        if !self.config.is_ai_configured() {
            println!("❌ AI is not configured");
            println!("💡 Run 'prism config --setup' to configure AI features");
            return Ok(());
        }

        // Show current configuration
        let (provider_name, _) = self.config.get_provider_info();
        println!("📡 Provider: {}", provider_name);
        println!("🤖 Model: {}", self.config.llm.model);
        if let Some(url) = &self.config.llm.base_url {
            println!("🌐 Base URL: {}", url);
        }
        println!();

        // Test with a simple prompt
        println!("🔄 Testing AI connection with simple prompt...");
        let test_prompt = "Analyze this requirement: 'The system should respond quickly'";
        
        match self.analyzer.call_llm(test_prompt).await {
            Ok(response) => {
                println!("✅ AI connection successful!");
                println!("📝 Response preview: {}...", 
                    if response.len() > 100 { 
                        &response[..100] 
                    } else { 
                        &response 
                    });
                println!("\n🎉 Configuration is working properly!");
            }
            Err(e) => {
                println!("❌ AI connection failed: {}", e);
                
                // Provide specific troubleshooting based on provider
                match self.config.llm.provider.as_str() {
                    "ollama" => {
                        println!("\n🔧 Ollama Troubleshooting:");
                        println!("1. Ensure Ollama is running: ollama serve");
                        println!("2. Check if model exists: ollama list");
                        println!("3. Pull the model if needed: ollama pull {}", self.config.llm.model);
                        println!("4. Try a different model: prism config --model llama3.1:latest");
                    }
                    "openai" => {
                        println!("\n🔧 OpenAI Troubleshooting:");
                        println!("1. Verify API key is correct");
                        println!("2. Check account has credits");
                        println!("3. Verify model name is correct");
                    }
                    "claude" => {
                        println!("\n🔧 Claude Troubleshooting:");
                        println!("1. Verify API key is correct");
                        println!("2. Check account has credits");
                        println!("3. Verify model name is correct");
                    }
                    "gemini" => {
                        println!("\n🔧 Gemini Troubleshooting:");
                        println!("1. Verify API key is correct");
                        println!("2. Check API is enabled in Google Cloud");
                        println!("3. Verify model name is correct");
                    }
                    _ => {
                        println!("\n🔧 General Troubleshooting:");
                        println!("1. Check internet connection");
                        println!("2. Verify API credentials");
                        println!("3. Try 'prism config --debug' for more info");
                    }
                }
            }
        }

        Ok(())
    }

    async fn save_individual_artifacts(&self, result: &AnalysisResult, base_filename: &str, input_text: &str) -> Result<()> {
        println!("💾 Saving individual artifacts...");
        
        // Save focused analysis report (only analysis content, no UML, pseudocode, or improved requirements)
        let analysis_filename = format!("{}_Analysis.md", base_filename);
        let analysis_content = self.format_focused_analysis(result, input_text);
        fs::write(&analysis_filename, analysis_content).await?;
        let analysis_path = std::fs::canonicalize(&analysis_filename).unwrap_or(PathBuf::from(&analysis_filename));
        println!("📄 Analysis report saved: {}", analysis_path.display());

        // Save improved requirements if available
        if let Some(improved_req) = &result.improved_requirements {
            let req_filename = format!("{}_Req.md", base_filename);
            let req_content = format!("# Improved Requirements\n\n{}\n\n---\n*Generated by PRISM - AI-Powered Requirement Analyzer*", improved_req);
            fs::write(&req_filename, req_content).await?;
            let req_path = std::fs::canonicalize(&req_filename).unwrap_or(PathBuf::from(&req_filename));
            println!("📄 Improved requirements saved: {}", req_path.display());
        }

        // Save UML diagrams if available
        if let Some(uml) = &result.uml_diagrams {
            let uml_filename = format!("{}_UML.puml", base_filename);
            let mut uml_content = String::new();
            
            if let Some(use_case) = &uml.use_case {
                uml_content.push_str("' Use Case Diagram\n");
                uml_content.push_str(use_case);
                uml_content.push_str("\n\n");
            }
            
            if let Some(sequence) = &uml.sequence {
                uml_content.push_str("' Sequence Diagram\n");
                uml_content.push_str("' Uncomment the section below to generate sequence diagram\n");
                uml_content.push_str("'\n");
                for line in sequence.lines() {
                    uml_content.push_str(&format!("' {}\n", line));
                }
                uml_content.push_str("\n\n");
            }
            
            if let Some(class_diagram) = &uml.class_diagram {
                uml_content.push_str("' Class Diagram\n");
                uml_content.push_str("' Uncomment the section below to generate class diagram\n");
                uml_content.push_str("'\n");
                for line in class_diagram.lines() {
                    uml_content.push_str(&format!("' {}\n", line));
                }
                uml_content.push_str("\n");
            }
            
            if !uml_content.is_empty() {
                let header = format!("' PlantUML Diagrams for: {}\n' Generated by PRISM - AI-Powered Requirement Analyzer\n' \n' Instructions:\n' 1. Use Case Diagram is uncommented by default\n' 2. Uncomment Sequence or Class diagrams as needed (remove ' from lines)\n' 3. Use PlantUML online editor or VS Code extension to render\n' 4. Visit: http://www.plantuml.com/plantuml/uml/\n\n", base_filename);
                uml_content = header + &uml_content;
                fs::write(&uml_filename, uml_content).await?;
                let uml_path = std::fs::canonicalize(&uml_filename).unwrap_or(PathBuf::from(&uml_filename));
                println!("🎨 UML diagrams saved: {}", uml_path.display());
            }
        }

        // Save pseudocode if available
        if let Some(pseudocode) = &result.pseudocode {
            let logic_filename = format!("{}_Logic.py", base_filename);
            let logic_content = format!("# Pseudocode Implementation\n# Generated by PRISM - AI-Powered Requirement Analyzer\n# \n# This code provides a structured foundation for implementing the requirements.\n# Replace placeholder implementations with actual business logic.\n\n{}", pseudocode);
            fs::write(&logic_filename, logic_content).await?;
            let logic_path = std::fs::canonicalize(&logic_filename).unwrap_or(PathBuf::from(&logic_filename));
            println!("🔧 Pseudocode saved: {}", logic_path.display());
        }

        // Save NFR suggestions if available
        if let Some(nfrs) = &result.nfr_suggestions {
            let nfr_filename = format!("{}_NFR.md", base_filename);
            let nfr_content = self.format_nfr_file(nfrs, base_filename);
            fs::write(&nfr_filename, nfr_content).await?;
            let nfr_path = std::fs::canonicalize(&nfr_filename).unwrap_or(PathBuf::from(&nfr_filename));
            println!("🔒 Non-functional requirements saved: {}", nfr_path.display());
        }

        println!("🎉 All artifacts saved successfully!");
        Ok(())
    }

    fn format_focused_analysis(&self, result: &AnalysisResult, input_text: &str) -> String {
        let mut output = String::new();
        
        output.push_str("# 🔍 PRISM Requirement Analysis Report\n\n");

        // Input echo section
        output.push_str("## 📝 Analyzed Requirement\n\n");
        output.push_str(&format!("> {}\n\n", input_text.trim()));

        // Summary section
        output.push_str("## 📊 Analysis Summary\n\n");
        output.push_str(&format!("- **Ambiguities Found:** {}\n", result.ambiguities.len()));
        output.push_str(&format!("- **Actors Identified:** {}\n", result.entities.actors.len()));
        output.push_str(&format!("- **Actions Identified:** {}\n", result.entities.actions.len()));
        output.push_str(&format!("- **Objects Identified:** {}\n\n", result.entities.objects.len()));

        // Ambiguities section
        if result.ambiguities.is_empty() {
            output.push_str("## ⚠️ Detected Ambiguities\n\n");
            output.push_str("✅ **No ambiguities detected - your requirements are clear!**\n\n");
        } else {
            output.push_str("## ⚠️ Detected Ambiguities\n\n");
            for (i, ambiguity) in result.ambiguities.iter().enumerate() {
                let severity_emoji = match ambiguity.severity {
                    crate::analyzer::AmbiguitySeverity::Critical => "🔴",
                    crate::analyzer::AmbiguitySeverity::High => "🟠",
                    crate::analyzer::AmbiguitySeverity::Medium => "🟡",
                    crate::analyzer::AmbiguitySeverity::Low => "🟢",
                };
                output.push_str(&format!("### {} Issue #{}: \"{}\"\n", severity_emoji, i + 1, ambiguity.text));
                output.push_str(&format!("- **Problem:** {}\n", ambiguity.reason));
                output.push_str(&format!("- **Severity:** {}\n", ambiguity.severity));
                output.push_str("- **Suggested Improvements:**\n");
                for suggestion in &ambiguity.suggestions {
                    output.push_str(&format!("  - {}\n", suggestion));
                }
                output.push_str("\n");
            }
        }

        // Entities section
        output.push_str("## 🎯 Extracted Entities\n\n");
        output.push_str("### 👥 Actors (Who performs actions)\n");
        if result.entities.actors.is_empty() {
            output.push_str("- No actors identified\n\n");
        } else {
            for actor in &result.entities.actors {
                output.push_str(&format!("- **{}**\n", actor));
            }
            output.push_str("\n");
        }

        output.push_str("### ⚡ Actions (What is being done)\n");
        if result.entities.actions.is_empty() {
            output.push_str("- No actions identified\n\n");
        } else {
            for action in &result.entities.actions {
                output.push_str(&format!("- **{}**\n", action));
            }
            output.push_str("\n");
        }

        output.push_str("### 📦 Objects (What is being acted upon)\n");
        if result.entities.objects.is_empty() {
            output.push_str("- No objects identified\n\n");
        } else {
            for object in &result.entities.objects {
                output.push_str(&format!("- **{}**\n", object));
            }
            output.push_str("\n");
        }

        // Completeness analysis section
        if let Some(completeness) = &result.completeness_analysis {
            output.push_str("## 📊 Completeness Analysis\n\n");
            output.push_str(&format!("**Completeness Score: {:.1}%**\n\n", completeness.completeness_score));
            
            if !completeness.gaps_identified.is_empty() {
                output.push_str("### Identified Gaps\n\n");
                for gap in &completeness.gaps_identified {
                    let priority_emoji = match gap.priority {
                        crate::analyzer::GapPriority::Critical => "🔴",
                        crate::analyzer::GapPriority::High => "🟠", 
                        crate::analyzer::GapPriority::Medium => "🟡",
                        crate::analyzer::GapPriority::Low => "🟢",
                    };
                    output.push_str(&format!("#### {} {} - {:?}\n\n", priority_emoji, gap.category, gap.priority));
                    output.push_str(&format!("**Issue:** {}\n\n", gap.description));
                    output.push_str("**Suggestions:**\n");
                    for suggestion in &gap.suggestions {
                        output.push_str(&format!("- {}\n", suggestion));
                    }
                    output.push_str("\n");
                }
            }
        }

        // User story validation section
        if let Some(user_story) = &result.user_story_validation {
            output.push_str("## ✅ User Story Validation\n\n");
            if user_story.is_valid_format {
                output.push_str("✅ **Valid user story format detected**\n\n");
                output.push_str(&format!("**Business Value Score: {:.1}%**\n\n", user_story.business_value_score));
                
                output.push_str("### Component Analysis\n\n");
                output.push_str(&format!("**Actor Quality:** {:.1}% - {}\n", user_story.actor_quality.score,
                    if user_story.actor_quality.is_valid { "✅ Valid" } else { "❌ Issues found" }));
                output.push_str(&format!("**Goal Quality:** {:.1}% - {}\n", user_story.goal_quality.score,
                    if user_story.goal_quality.is_valid { "✅ Valid" } else { "❌ Issues found" }));
                output.push_str(&format!("**Reason Quality:** {:.1}% - {}\n\n", user_story.reason_quality.score,
                    if user_story.reason_quality.is_valid { "✅ Valid" } else { "❌ Issues found" }));
            } else {
                output.push_str("❌ **Not in valid user story format**\n\n");
            }
            
            if !user_story.recommendations.is_empty() {
                output.push_str("### Recommendations\n\n");
                for rec in &user_story.recommendations {
                    output.push_str(&format!("- {}\n", rec));
                }
                output.push_str("\n");
            }
        }

        output.push_str("---\n*Generated by PRISM - AI-Powered Requirement Analyzer*\n");
        output
    }

    fn format_nfr_file(&self, nfrs: &Vec<crate::analyzer::NonFunctionalRequirement>, base_filename: &str) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("# Non-Functional Requirements for: {}\n", base_filename));
        output.push_str("*Generated by PRISM - AI-Powered Requirement Analyzer*\n\n");

        let mut categories = std::collections::BTreeMap::new();
        
        // Group NFRs by category
        for nfr in nfrs {
            categories.entry(&nfr.category).or_insert(Vec::new()).push(nfr);
        }
        
        for (category, category_nfrs) in categories {
            let category_emoji = match category {
                crate::analyzer::NfrCategory::Performance => "⚡",
                crate::analyzer::NfrCategory::Security => "🔒",
                crate::analyzer::NfrCategory::Usability => "👤",
                crate::analyzer::NfrCategory::Reliability => "🛡️",
                crate::analyzer::NfrCategory::Scalability => "📈",
                crate::analyzer::NfrCategory::Maintainability => "🔧",
                crate::analyzer::NfrCategory::Compatibility => "🔗",
                crate::analyzer::NfrCategory::Accessibility => "♿",
            };
            output.push_str(&format!("## {} {:?} Requirements\n\n", category_emoji, category));
            
            for (i, nfr) in category_nfrs.iter().enumerate() {
                let priority_text = match nfr.priority {
                    crate::analyzer::NfrPriority::MustHave => "🔴 Must Have",
                    crate::analyzer::NfrPriority::ShouldHave => "🟠 Should Have",
                    crate::analyzer::NfrPriority::CouldHave => "🟡 Could Have",
                    crate::analyzer::NfrPriority::WontHave => "⚫ Won't Have",
                };
                
                output.push_str(&format!("### NFR-{:?}-{:02}\n\n", category, i + 1));
                output.push_str(&format!("**Priority:** {}\n\n", priority_text));
                output.push_str(&format!("**Requirement:** {}\n\n", nfr.requirement));
                output.push_str(&format!("**Rationale:** {}\n\n", nfr.rationale));
                
                if !nfr.acceptance_criteria.is_empty() {
                    output.push_str("**Acceptance Criteria:**\n");
                    for criteria in &nfr.acceptance_criteria {
                        output.push_str(&format!("- {}\n", criteria));
                    }
                    output.push_str("\n");
                }
                output.push_str("---\n\n");
            }
        }

        output
    }
    
    async fn process_directory_batch(
        &self,
        dir_path: &PathBuf,
        output: Option<PathBuf>,
        format: Option<OutputFormat>,
        uml: bool,
        pseudo: bool,
        tests: bool,
        improve: bool,
        save_artifacts: Option<String>,
        completeness: bool,
        validate_story: bool,
        nfr: bool,
        pseudo_lang: Option<String>,
    ) -> Result<()> {
        if !dir_path.exists() || !dir_path.is_dir() {
            return Err(anyhow::anyhow!("Directory does not exist: {:?}", dir_path));
        }

        println!("📁 Scanning directory for individual file processing: {}", dir_path.display());
        
        let mut processed_files = Vec::new();
        let mut file_count = 0;

        // Collect all supported files first
        for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && self.document_processor.is_supported_format(path) {
                processed_files.push(path.to_path_buf());
            }
        }

        if processed_files.is_empty() {
            return Err(anyhow::anyhow!("No readable files (.md, .txt, .rst, .pdf, .docx, .xlsx) found in directory"));
        }

        println!("📊 Found {} requirement files to process individually", processed_files.len());

        // Process each file individually
        for file_path in processed_files {
            println!("\n🔍 Processing: {}", file_path.display());
            
            match self.document_processor.extract_text_from_file(&file_path).await {
                Ok(content) => {
                    println!("📄 Loaded {} characters from {}", content.len(), file_path.file_name().unwrap().to_string_lossy());
                    
                    if self.config.is_ai_configured() {
                        let (provider_name, _) = self.config.get_provider_info();
                        println!("🤖 Analyzing with {} ({})...", provider_name, self.config.llm.model);
                    } else {
                        println!("📋 Analyzing with built-in analysis...");
                    }
                    
                    // Analyze the individual file
                    let mut result = self.analyzer.analyze(&content).await?;

                    if uml {
                        println!("🎨 Generating UML diagrams...");
                        let use_case = self.analyzer.generate_uml_use_case(&result.entities);
                        let sequence = self.analyzer.generate_uml_sequence(&result.entities);
                        let class_diagram = self.analyzer.generate_uml_class_diagram(&result.entities);
                        result.uml_diagrams = Some(crate::analyzer::UmlDiagrams {
                            use_case: Some(use_case),
                            sequence: Some(sequence),
                            class_diagram: Some(class_diagram),
                        });
                    }

                    if pseudo {
                        println!("📝 Generating pseudocode structure...");
                        let pseudocode = self.analyzer.generate_pseudocode(&result.entities, pseudo_lang.as_deref());
                        result.pseudocode = Some(pseudocode);
                    }

                    if tests {
                        println!("🧪 Generating test cases...");
                        let test_cases = self.analyzer.generate_test_cases(&result.entities);
                        result.test_cases = Some(test_cases);
                    }

                    if improve {
                        println!("✨ Generating improved requirements...");
                        match self.analyzer.generate_improved_requirements(&content, &result.ambiguities).await {
                            Ok(improved_req) => {
                                result.improved_requirements = Some(improved_req);
                                println!("✅ Requirements improvement completed!");
                            }
                            Err(e) => {
                                eprintln!("⚠️  Could not generate improved requirements: {}", e);
                                if !self.config.is_ai_configured() {
                                    println!("💡 Suggestions:");
                                    println!("1. Configure AI provider: 'prism config --setup'");
                                    println!("2. Verify API credentials");
                                    println!("3. Try 'prism config --debug' for more info");
                                }
                            }
                        }
                    }

                    if completeness {
                        println!("📊 Analyzing completeness and identifying gaps...");
                        let completeness_analysis = self.analyzer.analyze_completeness(&content, &result.entities).await?;
                        result.completeness_analysis = Some(completeness_analysis);
                    }

                    if validate_story {
                        println!("✅ Validating user story format and business value...");
                        let validation = self.analyzer.validate_user_story(&content);
                        result.user_story_validation = Some(validation);
                    }

                    if nfr {
                        println!("🔒 Generating non-functional requirement suggestions...");
                        let nfr_suggestions = self.analyzer.generate_nfr_suggestions(&content, &result.entities).await?;
                        result.nfr_suggestions = Some(nfr_suggestions);
                    }

                    // Create output filename based on original file
                    let file_stem = file_path.file_stem().unwrap().to_string_lossy();
                    let output_filename = if let Some(ref base_output) = output {
                        // If output is specified, create filename with file stem
                        let base_name = base_output.file_stem().unwrap().to_string_lossy();
                        let extension = base_output.extension().unwrap_or_default().to_string_lossy();
                        if extension.is_empty() {
                            format!("{}_{}.md", base_name, file_stem)
                        } else {
                            format!("{}_{}.{}", base_name, file_stem, extension)
                        }
                    } else {
                        // Default filename
                        format!("{}_analysis.md", file_stem)
                    };

                    // Save individual artifacts if requested
                    if let Some(ref base_filename) = save_artifacts {
                        let artifact_base = format!("{}_{}", base_filename, file_stem);
                        self.save_individual_artifacts(&result, &artifact_base, &content).await?;
                    }

                    // Output the result for this file
                    let individual_output = PathBuf::from(output_filename);
                    let output_format = format.clone().unwrap_or(OutputFormat::Markdown);
                    
                    let output_content = match output_format {
                        OutputFormat::Json => serde_json::to_string_pretty(&result)?,
                        OutputFormat::Markdown => self.format_as_markdown(&result, &content),
                        OutputFormat::Jira => self.format_as_jira(&result, &content),
                        OutputFormat::Github => self.format_as_github(&result, &content),
                        OutputFormat::Plain => self.format_as_plain(&result, &content),
                    };
                    
                    let absolute_path = std::fs::canonicalize(&individual_output).unwrap_or(individual_output.clone());
                    fs::write(&individual_output, output_content).await?;
                    println!("📁 Analysis report created and saved: {}", absolute_path.display());
                    
                    println!("✅ Completed analysis for: {}", file_path.display());
                    file_count += 1;
                }
                Err(e) => {
                    eprintln!("⚠️  Could not process file {:?}: {}", file_path, e);
                }
            }
        }

        println!("\n🎉 Batch processing complete!");
        println!("📊 Successfully processed {} requirement files", file_count);
        println!("📁 Each file has its own individual analysis report");

        Ok(())
    }
}