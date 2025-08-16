use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "prism")]
#[command(about = "🔍 PRISM - AI-Powered Requirement Analyzer")]
#[command(long_about = "PRISM analyzes software requirements using smart presets and simplified commands.

QUICK START:
  prism analyze \"As a user, I want to login quickly\" --preset standard  # Smart preset
  prism improve \"As a user, I want to login quickly\"                    # Generate improved requirements  
  prism config --setup                                                  # Interactive AI setup
  prism tui                                                             # Launch interactive TUI

EXAMPLES:
  prism analyze --file requirements.txt --preset full --format markdown
  prism validate --dir ./stories --all --output validation.md
  prism dashboard --file requirements.txt --output dashboard.html
  prism trace --from-commit abc123 --to-commit def456")]
#[command(version = "1.0.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Analyze requirements and generate artifacts")]
    #[command(long_about = "Analyze software requirements with simplified options and smart presets.

INPUT OPTIONS (choose one):
  <TEXT>     Direct requirement text in quotes
  --file     Single file to analyze (.txt, .md, .rst, .pdf, .docx, .xlsx)
  --dir      Directory containing multiple requirement files

PRESET OPTIONS (recommended):
  --preset basic     Just analysis + ambiguity detection
  --preset standard  Analysis + UML + tests + pseudocode  
  --preset full      All generation options (UML, pseudo, tests, improve, NFRs)
  --preset report    Analysis optimized for markdown reports

CUSTOM GENERATION:
  --generate         Choose specific artifacts: all, uml, pseudo, tests, improve, nfr

OUTPUT OPTIONS:
  --format          Output format: json, markdown, github, jira, plain
  --output          Save results to file instead of displaying

EXAMPLES:
  prism analyze \"As a user, I want to reset my password\" --preset standard
  prism analyze --file story.txt --preset full --format markdown
  prism analyze --dir ./requirements --preset report --output analysis.md")]
    Analyze {
        #[arg(help = "Direct requirement text to analyze (use quotes for multi-word text)")]
        text: Option<String>,
        
        #[arg(short, long, help = "File to analyze (.txt, .md, .rst, .pdf, .docx, .xlsx files supported)")]
        file: Option<PathBuf>,
        
        #[arg(short, long, help = "Directory to analyze (processes all .txt, .md, .rst, .pdf, .docx, .xlsx files)")]
        dir: Option<PathBuf>,
        
        #[arg(short, long, help = "Save output to file instead of displaying on screen")]
        output: Option<PathBuf>,
        
        #[arg(long, help = "Use analysis preset", value_enum)]
        preset: Option<AnalysisPreset>,
        
        #[arg(long, help = "Generate specific artifacts", value_enum, action = clap::ArgAction::Append)]
        generate: Vec<GenerateOptions>,
        
        #[arg(long, help = "Output format", value_enum)]
        format: Option<OutputFormat>,
        
        #[arg(long, help = "Pseudocode language style (python, java, etc.)")]
        pseudo_lang: Option<String>,
        
        #[arg(long, help = "Save individual artifacts as separate files (base filename for suffixed files)")]
        save_artifacts: Option<String>,
        
        #[arg(long, help = "Use custom output template")]
        template: Option<String>,
        
        #[arg(long, help = "Add custom branding to output")]
        branding: Option<String>,
        
        #[arg(long, help = "Continue processing on errors instead of stopping")]
        continue_on_error: bool,
        
        #[arg(long, help = "Skip invalid files during directory processing")]
        skip_invalid: bool,
        
        #[arg(long, help = "Number of parallel processes for batch operations", default_value = "1")]
        parallel: usize,
    },
    
    #[command(about = "Launch interactive terminal interface")]
    #[command(long_about = "Start the interactive TUI (Terminal User Interface) with tabbed navigation:
  • 📝 Input tab: Enter and edit requirement text
  • ⚠️ Ambiguities tab: Review detected issues with suggestions  
  • 🎯 Entities tab: View extracted actors, actions, and objects
  • 📊 Output tab: See generated UML diagrams and pseudocode

KEYBOARD SHORTCUTS:
  q     Quit application
  h     Toggle help
  i     Enter editing mode
  Tab   Switch between tabs
  ↑/↓   Navigate lists")]
    Tui,
    
    #[command(about = "Generate improved requirements by fixing detected issues")]
    #[command(long_about = "Improve requirements by applying AI-powered suggestions to fix ambiguities and enhance clarity.

EXAMPLES:
  prism improve \"As a user, I want to login quickly\"
  prism improve --file requirements.txt --output improved_req.md
  prism improve --dir ./stories --format markdown")]
    Improve {
        #[arg(help = "Direct requirement text to improve (use quotes for multi-word text)")]
        text: Option<String>,
        
        #[arg(short, long, help = "File to improve (.txt, .md, .rst files supported)")]
        file: Option<PathBuf>,
        
        #[arg(short, long, help = "Directory to improve (processes all .txt, .md, .rst files)")]
        dir: Option<PathBuf>,
        
        #[arg(short, long, help = "Save improved requirements to file")]
        output: Option<PathBuf>,
        
        #[arg(long, help = "Output format", value_enum)]
        format: Option<OutputFormat>,
    },
    
    #[command(about = "Validate user stories and analyze completeness")]
    #[command(long_about = "Validate user story format, business value, and analyze requirement completeness.

VALIDATION OPTIONS:
  --story           Validate user story format and business value
  --completeness    Analyze completeness and identify gaps
  --all             Run all validation checks

EXAMPLES:
  prism validate \"As a user, I want to login\" --story
  prism validate --file story.txt --completeness
  prism validate --dir ./stories --all")]
    Validate {
        #[arg(help = "Direct requirement text to validate (use quotes for multi-word text)")]
        text: Option<String>,
        
        #[arg(short, long, help = "File to validate")]
        file: Option<PathBuf>,
        
        #[arg(short, long, help = "Directory to validate")]
        dir: Option<PathBuf>,
        
        #[arg(short, long, help = "Save output to file")]
        output: Option<PathBuf>,
        
        #[arg(long, help = "Validate user story format and business value")]
        story: bool,
        
        #[arg(long, help = "Analyze completeness and identify gaps")]
        completeness: bool,
        
        #[arg(long, help = "Run all validation checks")]
        all: bool,
        
        #[arg(long, help = "Output format", value_enum)]
        format: Option<OutputFormat>,
    },

    #[command(about = "Trace requirements to source code and tests")]
    #[command(long_about = "Trace requirements to implementation and test files using git integration.

EXAMPLES:
  prism trace --from-commit abc123 --to-commit def456
  prism trace --file requirements.txt --source-dir ./src --test-dir ./tests")]
    Trace {
        #[arg(help = "Requirements text or identifier")]
        text: Option<String>,
        
        #[arg(short, long, help = "Requirements file to trace")]
        file: Option<PathBuf>,
        
        #[arg(short, long, help = "Save output to file")]
        output: Option<PathBuf>,
        
        #[arg(long, help = "Git commit hash to compare from")]
        from_commit: Option<String>,
        
        #[arg(long, help = "Git commit hash to compare to")]
        to_commit: Option<String>,
        
        #[arg(long, help = "Source code directory to trace to")]
        source_dir: Option<PathBuf>,
        
        #[arg(long, help = "Test directory to trace to")]
        test_dir: Option<PathBuf>,
        
        #[arg(long, help = "Output format", value_enum)]
        format: Option<OutputFormat>,
    },

    #[command(about = "Generate executive dashboards and reports")]
    #[command(long_about = "Generate HTML dashboards, executive summaries, and professional reports.

EXAMPLES:
  prism dashboard --file requirements.txt --output dashboard.html
  prism dashboard --dir ./stories --template enterprise --branding \"Company Name\"")]
    Dashboard {
        #[arg(help = "Requirements text for dashboard")]
        text: Option<String>,
        
        #[arg(short, long, help = "File to generate dashboard from")]
        file: Option<PathBuf>,
        
        #[arg(short, long, help = "Directory to generate dashboard from")]
        dir: Option<PathBuf>,
        
        #[arg(short, long, help = "Output file for dashboard")]
        output: Option<PathBuf>,
        
        #[arg(long, help = "Use custom template")]
        template: Option<String>,
        
        #[arg(long, help = "Add custom branding")]
        branding: Option<String>,
        
        #[arg(long, help = "Generate executive summary")]
        executive_summary: bool,
    },

    #[command(about = "Setup and manage AI configuration")]
    #[command(long_about = "Configure PRISM for AI-powered analysis. This tool is designed to work with AI providers for enhanced analysis.

SUPPORTED AI PROVIDERS:
  • OpenAI (GPT-4, GPT-3.5-turbo, GPT-4o)
  • Google Gemini (gemini-1.5-pro, gemini-1.5-flash)
  • Anthropic Claude (claude-3-opus, claude-3-sonnet, claude-3-haiku)
  • Azure OpenAI
  • Local Ollama (llama2, codellama, mistral, etc.)

QUICK SETUP:
  prism config --setup            # Interactive setup wizard
  prism config --provider openai  # Quick OpenAI setup
  prism config --provider gemini  # Quick Gemini setup
  prism config --provider claude  # Quick Claude setup
  prism config --provider ollama  # Quick Ollama setup

MANUAL SETUP:
  prism config --api-key \"your-key\" --model \"gpt-4\" --provider openai
  prism config --api-key \"your-key\" --model \"gemini-1.5-pro\" --provider gemini
  prism config --api-key \"your-key\" --model \"claude-3-sonnet\" --provider claude

CONFIGURATION FILE: ~/.prism/config.yml")]
    Config {
        #[arg(short, long, help = "Set API key for your chosen AI provider")]
        api_key: Option<String>,
        
        #[arg(short, long, help = "Set model name (e.g., gpt-4, gemini-1.5-pro)")]
        model: Option<String>,
        
        #[arg(short, long, help = "Set AI provider", value_enum)]
        provider: Option<AiProvider>,
        
        #[arg(long, help = "Interactive setup wizard for first-time configuration")]
        setup: bool,
        
        #[arg(long, help = "Display current configuration values")]
        show: bool,
        
        #[arg(long, help = "Show config file location, status, and auto-create if missing")]
        debug: bool,
        
        #[arg(long, help = "Test current AI configuration and connection")]
        test: bool,
        
        #[arg(long, help = "Validate all configuration settings")]
        validate_all: bool,
        
        #[arg(long, help = "Test all configured AI providers")]
        test_providers: bool,
        
        #[arg(long, help = "Set custom template directory")]
        set_template_dir: Option<PathBuf>,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Json,
    Markdown,
    Jira,
    Github,
    Plain,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum AnalysisPreset {
    Basic,
    Standard,
    Full,
    Report,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum GenerateOptions {
    All,
    Uml,
    Pseudo,
    Tests,
    Improve,
    Nfr,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum AiProvider {
    OpenAI,
    Gemini,
    Azure,
    Claude,
    Ollama,
}