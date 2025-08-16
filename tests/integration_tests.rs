use prism::app::App;
use prism::cli::{Commands, OutputFormat, AnalysisPreset, GenerateOptions, AiProvider};
use std::path::PathBuf;
use tokio::fs;

#[tokio::test]
async fn test_app_creation() {
    let app = App::new().await;
    assert!(app.is_ok());
}

#[tokio::test]
async fn test_text_analysis_command() {
    let mut app = App::new().await.unwrap();
    
    let command = Commands::Analyze {
        text: Some("As a user, I want to login quickly".to_string()),
        file: None,
        dir: None,
        output: None,
        preset: Some(AnalysisPreset::Basic),
        generate: vec![],
        format: Some(OutputFormat::Json),
        pseudo_lang: None,
        save_artifacts: None,
        template: None,
        branding: None,
        continue_on_error: false,
        skip_invalid: false,
        parallel: 1,
    };
    
    let result = app.run_command(command).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_file_analysis_command() {
    // Create a temporary test file
    let test_content = "As a user, I want the system to be fast and efficient";
    fs::write("temp_test.txt", test_content).await.unwrap();
    
    let mut app = App::new().await.unwrap();
    
    let command = Commands::Analyze {
        text: None,
        file: Some(PathBuf::from("temp_test.txt")),
        dir: None,
        output: None,
        preset: None,
        generate: vec![GenerateOptions::Uml, GenerateOptions::Pseudo, GenerateOptions::Tests],
        format: Some(OutputFormat::Markdown),
        pseudo_lang: Some("python".to_string()),
        save_artifacts: None,
        template: None,
        branding: None,
        continue_on_error: false,
        skip_invalid: false,
        parallel: 1,
    };
    
    let result = app.run_command(command).await;
    assert!(result.is_ok());
    
    // Clean up
    let _ = fs::remove_file("temp_test.txt").await;
}

#[tokio::test]
async fn test_output_to_file() {
    let mut app = App::new().await.unwrap();
    let output_file = PathBuf::from("test_integration_output.md");
    
    let command = Commands::Analyze {
        text: Some("The system should respond fast".to_string()),
        file: None,
        dir: None,
        output: Some(output_file.clone()),
        preset: Some(AnalysisPreset::Basic),
        generate: vec![],
        format: Some(OutputFormat::Markdown),
        pseudo_lang: None,
        save_artifacts: None,
        template: None,
        branding: None,
        continue_on_error: false,
        skip_invalid: false,
        parallel: 1,
    };
    
    let result = app.run_command(command).await;
    assert!(result.is_ok());
    
    // Check if output file was created
    assert!(output_file.exists());
    
    // Verify content
    let content = fs::read_to_string(&output_file).await.unwrap();
    assert!(content.contains("Requirement Analysis Report"));
    assert!(content.contains("fast"));
    
    // Clean up
    let _ = fs::remove_file(&output_file).await;
}

#[tokio::test]
async fn test_config_command() {
    let mut app = App::new().await.unwrap();
    
    let command = Commands::Config {
        api_key: Some("test-key".to_string()),
        model: Some("test-model".to_string()),
        provider: Some(AiProvider::OpenAI),
        setup: false,
        show: false,
        debug: false,
        test: false,
        validate_all: false,
        test_providers: false,
        set_template_dir: None,
    };
    
    let result = app.run_command(command).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_all_output_formats() {
    let formats = vec![
        OutputFormat::Json,
        OutputFormat::Markdown,
        OutputFormat::Github,
        OutputFormat::Jira,
        OutputFormat::Plain,
    ];
    
    for format in formats {
        let mut app = App::new().await.unwrap();
        
        let command = Commands::Analyze {
            text: Some("Test requirement for format".to_string()),
            file: None,
            dir: None,
            output: None,
            preset: Some(AnalysisPreset::Basic),
            generate: vec![],
            format: Some(format.clone()),
            pseudo_lang: None,
            save_artifacts: None,
            template: None,
            branding: None,
            continue_on_error: false,
            skip_invalid: false,
            parallel: 1,
        };
        
        let result = app.run_command(command).await;
        assert!(result.is_ok(), "Failed for format: {:?}", format);
    }
}

#[tokio::test]
async fn test_error_handling_nonexistent_file() {
    let mut app = App::new().await.unwrap();
    
    let command = Commands::Analyze {
        text: None,
        file: Some(PathBuf::from("nonexistent_file.txt")),
        dir: None,
        output: None,
        preset: Some(AnalysisPreset::Basic),
        generate: vec![],
        format: Some(OutputFormat::Json),
        pseudo_lang: None,
        save_artifacts: None,
        template: None,
        branding: None,
        continue_on_error: false,
        skip_invalid: false,
        parallel: 1,
    };
    
    let result = app.run_command(command).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_error_handling_nonexistent_directory() {
    let mut app = App::new().await.unwrap();
    
    let command = Commands::Analyze {
        text: None,
        file: None,
        dir: Some(PathBuf::from("nonexistent_directory")),
        output: None,
        preset: Some(AnalysisPreset::Basic),
        generate: vec![],
        format: Some(OutputFormat::Json),
        pseudo_lang: None,
        save_artifacts: None,
        template: None,
        branding: None,
        continue_on_error: false,
        skip_invalid: false,
        parallel: 1,
    };
    
    let result = app.run_command(command).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_directory_analysis() {
    // Create temporary directory with test files
    fs::create_dir_all("temp_test_dir").await.unwrap();
    fs::write("temp_test_dir/story1.md", "As a user, I want to login").await.unwrap();
    fs::write("temp_test_dir/story2.md", "As an admin, I want to manage users").await.unwrap();
    
    let mut app = App::new().await.unwrap();
    
    let command = Commands::Analyze {
        text: None,
        file: None,
        dir: Some(PathBuf::from("temp_test_dir")),
        output: None,
        preset: Some(AnalysisPreset::Basic),
        generate: vec![],
        format: Some(OutputFormat::Json),
        pseudo_lang: None,
        save_artifacts: None,
        template: None,
        branding: None,
        continue_on_error: false,
        skip_invalid: false,
        parallel: 1,
    };
    
    let result = app.run_command(command).await;
    assert!(result.is_ok());
    
    // Clean up
    let _ = fs::remove_dir_all("temp_test_dir").await;
}

#[tokio::test]
async fn test_comprehensive_analysis_with_all_features() {
    let mut app = App::new().await.unwrap();
    
    let complex_requirement = r#"
        As a user, I want to quickly access my dashboard after login.
        The system should be user-friendly and respond fast to requests.
        Data must be validated and stored securely.
        Admin users should manage accounts efficiently.
    "#;
    
    let command = Commands::Analyze {
        text: Some(complex_requirement.to_string()),
        file: None,
        dir: None,
        output: Some(PathBuf::from("comprehensive_test.md")),
        preset: Some(AnalysisPreset::Full),
        generate: vec![],
        format: Some(OutputFormat::Markdown),
        pseudo_lang: Some("python".to_string()),
        save_artifacts: None,
        template: None,
        branding: None,
        continue_on_error: false,
        skip_invalid: false,
        parallel: 1,
    };
    
    let result = app.run_command(command).await;
    assert!(result.is_ok());
    
    // Verify the output file contains all expected sections
    let content = fs::read_to_string("comprehensive_test.md").await.unwrap();
    assert!(content.contains("Detected Ambiguities"));
    assert!(content.contains("Extracted Entities"));
    assert!(content.contains("UML Use Case Diagram"));
    assert!(content.contains("Generated Pseudocode"));
    assert!(content.contains("Suggested Test Cases"));
    assert!(content.contains("@startuml"));
    assert!(content.contains("def "));  // Python-style pseudocode
    
    // Clean up
    let _ = fs::remove_file("comprehensive_test.md").await;
}

#[tokio::test]
async fn test_validate_command() {
    let mut app = App::new().await.unwrap();
    
    let command = Commands::Validate {
        text: Some("As a user, I want to login quickly".to_string()),
        file: None,
        dir: None,
        output: None,
        story: true,
        completeness: false,
        all: false,
        format: Some(OutputFormat::Json),
    };
    
    let result = app.run_command(command).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_all_command() {
    let mut app = App::new().await.unwrap();
    
    let command = Commands::Validate {
        text: Some("As a user, I want to login quickly".to_string()),
        file: None,
        dir: None,
        output: None,
        story: false,
        completeness: false,
        all: true,
        format: Some(OutputFormat::Json),
    };
    
    let result = app.run_command(command).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_trace_command() {
    let mut app = App::new().await.unwrap();
    
    let command = Commands::Trace {
        text: Some("User login requirement".to_string()),
        file: None,
        output: None,
        from_commit: Some("HEAD~1".to_string()),
        to_commit: Some("HEAD".to_string()),
        source_dir: None,
        test_dir: None,
        format: Some(OutputFormat::Json),
    };
    
    let result = app.run_command(command).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dashboard_command() {
    let mut app = App::new().await.unwrap();
    
    let command = Commands::Dashboard {
        text: Some("As a user, I want to login quickly".to_string()),
        file: None,
        dir: None,
        output: Some(PathBuf::from("test_dashboard.html")),
        template: None,
        branding: None,
        executive_summary: false,
    };
    
    let result = app.run_command(command).await;
    assert!(result.is_ok());
    
    // Clean up
    let _ = fs::remove_file("test_dashboard.html").await;
}

#[tokio::test]
async fn test_preset_combinations() {
    let presets = vec![
        AnalysisPreset::Basic,
        AnalysisPreset::Standard,
        AnalysisPreset::Full,
        AnalysisPreset::Report,
    ];
    
    for preset in presets {
        let mut app = App::new().await.unwrap();
        
        let command = Commands::Analyze {
            text: Some("Test requirement for preset".to_string()),
            file: None,
            dir: None,
            output: None,
            preset: Some(preset.clone()),
            generate: vec![],
            format: Some(OutputFormat::Json),
            pseudo_lang: None,
            save_artifacts: None,
            template: None,
            branding: None,
            continue_on_error: false,
            skip_invalid: false,
            parallel: 1,
        };
        
        let result = app.run_command(command).await;
        assert!(result.is_ok(), "Failed for preset: {:?}", preset);
    }
}

#[tokio::test]
async fn test_custom_generate_options() {
    let mut app = App::new().await.unwrap();
    
    let command = Commands::Analyze {
        text: Some("Test requirement for custom generation".to_string()),
        file: None,
        dir: None,
        output: None,
        preset: None,
        generate: vec![GenerateOptions::Uml, GenerateOptions::Tests, GenerateOptions::Improve],
        format: Some(OutputFormat::Markdown),
        pseudo_lang: None,
        save_artifacts: None,
        template: None,
        branding: None,
        continue_on_error: false,
        skip_invalid: false,
        parallel: 1,
    };
    
    let result = app.run_command(command).await;
    assert!(result.is_ok());
}