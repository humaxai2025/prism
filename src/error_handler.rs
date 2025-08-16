use anyhow::{Result, anyhow};
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ProcessingError {
    pub file_path: Option<PathBuf>,
    pub error_type: ErrorType,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    FileNotFound,
    FileCorrupted,
    UnreadableFormat,
    ApiError,
    NetworkError,
    ConfigurationError,
    ProcessingTimeout,
}

pub struct ErrorHandler {
    continue_on_error: bool,
    skip_invalid: bool,
    errors: Vec<ProcessingError>,
    warnings: Vec<String>,
}

impl ErrorHandler {
    pub fn new(continue_on_error: bool, skip_invalid: bool) -> Self {
        Self {
            continue_on_error,
            skip_invalid,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn handle_error(&mut self, error: ProcessingError) -> Result<bool> {
        let should_continue = match error.error_type {
            ErrorType::FileNotFound | ErrorType::FileCorrupted | ErrorType::UnreadableFormat => {
                if self.skip_invalid {
                    self.warnings.push(format!("⚠️  Skipped invalid file: {} - {}", 
                        error.file_path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "unknown".to_string()),
                        error.message));
                    true
                } else if self.continue_on_error {
                    eprintln!("❌ Error processing {}: {}", 
                        error.file_path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "unknown".to_string()),
                        error.message);
                    self.errors.push(error);
                    true
                } else {
                    self.errors.push(error);
                    false
                }
            }
            ErrorType::ApiError | ErrorType::NetworkError => {
                if self.continue_on_error {
                    eprintln!("⚠️  API/Network error: {} - continuing with basic analysis", error.message);
                    self.errors.push(error);
                    true
                } else {
                    self.errors.push(error);
                    false
                }
            }
            ErrorType::ConfigurationError => {
                // Configuration errors are always critical
                self.errors.push(error);
                false
            }
            ErrorType::ProcessingTimeout => {
                if self.continue_on_error {
                    eprintln!("⚠️  Processing timeout: {} - skipping", error.message);
                    self.errors.push(error);
                    true
                } else {
                    self.errors.push(error);
                    false
                }
            }
        };

        Ok(should_continue)
    }

    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn get_summary(&self) -> ErrorSummary {
        let mut error_counts = HashMap::new();
        for error in &self.errors {
            *error_counts.entry(format!("{:?}", error.error_type)).or_insert(0) += 1;
        }

        ErrorSummary {
            total_errors: self.errors.len(),
            total_warnings: self.warnings.len(),
            error_counts,
            errors: self.errors.clone(),
            warnings: self.warnings.clone(),
        }
    }

    pub fn print_summary(&self) {
        if self.has_errors() || self.has_warnings() {
            println!("\n📊 Processing Summary");
            println!("===================");
            
            if self.has_warnings() {
                println!("⚠️  Warnings: {}", self.warnings.len());
                for warning in &self.warnings {
                    println!("   {}", warning);
                }
            }

            if self.has_errors() {
                println!("❌ Errors: {}", self.errors.len());
                let mut error_counts = HashMap::new();
                for error in &self.errors {
                    *error_counts.entry(format!("{:?}", error.error_type)).or_insert(0) += 1;
                }
                
                for (error_type, count) in error_counts {
                    println!("   {}: {}", error_type, count);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ErrorSummary {
    pub total_errors: usize,
    pub total_warnings: usize,
    pub error_counts: HashMap<String, usize>,
    pub errors: Vec<ProcessingError>,
    pub warnings: Vec<String>,
}

// Helper functions for creating common errors
impl ProcessingError {
    pub fn file_not_found(path: PathBuf) -> Self {
        Self {
            file_path: Some(path),
            error_type: ErrorType::FileNotFound,
            message: "File not found".to_string(),
            recoverable: true,
        }
    }

    pub fn file_corrupted(path: PathBuf, details: String) -> Self {
        Self {
            file_path: Some(path),
            error_type: ErrorType::FileCorrupted,
            message: format!("File corrupted: {}", details),
            recoverable: true,
        }
    }

    pub fn unreadable_format(path: PathBuf, format: String) -> Self {
        Self {
            file_path: Some(path),
            error_type: ErrorType::UnreadableFormat,
            message: format!("Unsupported or unreadable format: {}", format),
            recoverable: true,
        }
    }

    pub fn api_error(message: String) -> Self {
        Self {
            file_path: None,
            error_type: ErrorType::ApiError,
            message: format!("AI API error: {}", message),
            recoverable: true,
        }
    }

    pub fn network_error(message: String) -> Self {
        Self {
            file_path: None,
            error_type: ErrorType::NetworkError,
            message: format!("Network error: {}", message),
            recoverable: true,
        }
    }

    pub fn config_error(message: String) -> Self {
        Self {
            file_path: None,
            error_type: ErrorType::ConfigurationError,
            message: format!("Configuration error: {}", message),
            recoverable: false,
        }
    }

    pub fn timeout_error(path: Option<PathBuf>, message: String) -> Self {
        Self {
            file_path: path,
            error_type: ErrorType::ProcessingTimeout,
            message: format!("Processing timeout: {}", message),
            recoverable: true,
        }
    }
}