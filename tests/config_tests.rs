use prism::config::*;
use std::env;

#[tokio::test]
async fn test_default_config() {
    let config = Config::default();
    
    assert_eq!(config.llm.model, "gpt-4");
    assert_eq!(config.llm.timeout, 30);
    assert_eq!(config.analysis.ambiguity_threshold, 0.7);
    assert!(config.analysis.enable_interactive);
}

#[tokio::test]
async fn test_config_with_env_var() {
    env::set_var("PRISM_API_KEY", "test-key-123");
    let config = Config::default();
    assert_eq!(config.llm.api_key, Some("test-key-123".to_string()));
    env::remove_var("PRISM_API_KEY");
}

#[tokio::test]
async fn test_config_modification() {
    let mut config = Config::default();
    
    config.set_api_key("new-api-key".to_string());
    config.set_model("gpt-3.5-turbo".to_string());
    
    assert_eq!(config.llm.api_key, Some("new-api-key".to_string()));
    assert_eq!(config.llm.model, "gpt-3.5-turbo");
}

#[test]
fn test_config_path() {
    let path = Config::config_path().unwrap();
    assert!(path.to_string_lossy().contains(".prism"));
    assert!(path.to_string_lossy().contains("config.yml"));
}