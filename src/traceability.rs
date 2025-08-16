use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::fs;
use walkdir::WalkDir;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceabilityMatrix {
    pub requirements: Vec<RequirementTrace>,
    pub coverage_summary: CoverageSummary,
    pub orphaned_code: Vec<OrphanedCode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementTrace {
    pub requirement_id: String,
    pub requirement_text: String,
    pub code_references: Vec<CodeReference>,
    pub test_references: Vec<TestReference>,
    pub coverage_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReference {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub code_snippet: String,
    pub confidence: f64,
    pub match_type: MatchType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReference {
    pub file_path: PathBuf,
    pub test_name: String,
    pub line_number: usize,
    pub test_type: TestType,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanedCode {
    pub file_path: PathBuf,
    pub function_name: String,
    pub line_number: usize,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSummary {
    pub total_requirements: usize,
    pub traced_requirements: usize,
    pub coverage_percentage: f64,
    pub code_files_analyzed: usize,
    pub test_files_analyzed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType {
    ExactMatch,
    FuzzyMatch,
    KeywordMatch,
    CommentMatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestType {
    UnitTest,
    IntegrationTest,
    EndToEndTest,
    Unknown,
}

pub struct TraceabilityAnalyzer {
    source_extensions: HashSet<String>,
    test_extensions: HashSet<String>,
    comment_patterns: HashMap<String, Regex>,
    keyword_patterns: Vec<Regex>,
}

impl TraceabilityAnalyzer {
    pub fn new() -> Self {
        let mut source_extensions = HashSet::new();
        source_extensions.insert("rs".to_string());
        source_extensions.insert("py".to_string());
        source_extensions.insert("js".to_string());
        source_extensions.insert("ts".to_string());
        source_extensions.insert("java".to_string());
        source_extensions.insert("cpp".to_string());
        source_extensions.insert("c".to_string());
        source_extensions.insert("go".to_string());

        let mut test_extensions = HashSet::new();
        test_extensions.insert("rs".to_string());  // test modules
        test_extensions.insert("py".to_string());  // test_*.py
        test_extensions.insert("js".to_string());  // *.test.js
        test_extensions.insert("ts".to_string());  // *.test.ts
        test_extensions.insert("java".to_string()); // *Test.java

        let mut comment_patterns = HashMap::new();
        comment_patterns.insert("rs".to_string(), Regex::new(r"//\s*(.+)|/\*\s*(.+?)\s*\*/").unwrap());
        comment_patterns.insert("py".to_string(), Regex::new(r"#\s*(.+)|'''\s*(.+?)\s*'''|"""\s*(.+?)\s*"""").unwrap());
        comment_patterns.insert("js".to_string(), Regex::new(r"//\s*(.+)|/\*\s*(.+?)\s*\*/").unwrap());
        comment_patterns.insert("ts".to_string(), Regex::new(r"//\s*(.+)|/\*\s*(.+?)\s*\*/").unwrap());
        comment_patterns.insert("java".to_string(), Regex::new(r"//\s*(.+)|/\*\s*(.+?)\s*\*/").unwrap());

        let keyword_patterns = vec![
            Regex::new(r"(?i)req(?:uirement)?[_\s-]*(?:id|num|number)?[_\s-]*(\w+)").unwrap(),
            Regex::new(r"(?i)user[_\s-]*story[_\s-]*(\w+)").unwrap(),
            Regex::new(r"(?i)feature[_\s-]*(\w+)").unwrap(),
            Regex::new(r"(?i)as[_\s]+a[_\s]+(\w+)").unwrap(),
            Regex::new(r"(?i)i[_\s]+want[_\s]+to[_\s]+(\w+)").unwrap(),
        ];

        Self {
            source_extensions,
            test_extensions,
            comment_patterns,
            keyword_patterns,
        }
    }

    pub async fn analyze_traceability(
        &self,
        requirements: &[String],
        source_paths: &[PathBuf],
    ) -> Result<TraceabilityMatrix> {
        let mut requirement_traces = Vec::new();
        let mut all_code_files = Vec::new();
        let mut all_test_files = Vec::new();

        // Collect all source and test files
        for source_path in source_paths {
            let (code_files, test_files) = self.collect_files(source_path).await?;
            all_code_files.extend(code_files);
            all_test_files.extend(test_files);
        }

        // Analyze each requirement
        for (idx, requirement) in requirements.iter().enumerate() {
            let requirement_id = format!("REQ-{:03}", idx + 1);
            let trace = self.trace_requirement(
                &requirement_id,
                requirement,
                &all_code_files,
                &all_test_files,
            ).await?;
            requirement_traces.push(trace);
        }

        // Calculate coverage summary
        let traced_count = requirement_traces.iter()
            .filter(|r| !r.code_references.is_empty() || !r.test_references.is_empty())
            .count();

        let coverage_summary = CoverageSummary {
            total_requirements: requirements.len(),
            traced_requirements: traced_count,
            coverage_percentage: if requirements.is_empty() { 0.0 } else {
                (traced_count as f64 / requirements.len() as f64) * 100.0
            },
            code_files_analyzed: all_code_files.len(),
            test_files_analyzed: all_test_files.len(),
        };

        // Find orphaned code (code without clear requirement links)
        let orphaned_code = self.find_orphaned_code(&all_code_files, &requirement_traces).await?;

        Ok(TraceabilityMatrix {
            requirements: requirement_traces,
            coverage_summary,
            orphaned_code,
        })
    }

    async fn collect_files(&self, source_path: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
        let mut code_files = Vec::new();
        let mut test_files = Vec::new();

        if !source_path.exists() {
            return Ok((code_files, test_files));
        }

        for entry in WalkDir::new(source_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                if self.source_extensions.contains(extension) {
                    if self.is_test_file(path) {
                        test_files.push(path.to_path_buf());
                    } else {
                        code_files.push(path.to_path_buf());
                    }
                }
            }
        }

        Ok((code_files, test_files))
    }

    fn is_test_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        path_str.contains("test") || 
        path_str.contains("spec") ||
        path_str.ends_with("_test.rs") ||
        path_str.ends_with("_test.py") ||
        path_str.ends_with(".test.js") ||
        path_str.ends_with(".test.ts") ||
        path_str.ends_with("test.java")
    }

    async fn trace_requirement(
        &self,
        requirement_id: &str,
        requirement_text: &str,
        code_files: &[PathBuf],
        test_files: &[PathBuf],
    ) -> Result<RequirementTrace> {
        let mut code_references = Vec::new();
        let mut test_references = Vec::new();

        // Extract keywords from requirement
        let keywords = self.extract_keywords(requirement_text);

        // Search in code files
        for file_path in code_files {
            let references = self.search_file_for_requirement(
                file_path,
                requirement_id,
                requirement_text,
                &keywords,
                false,
            ).await?;
            code_references.extend(references);
        }

        // Search in test files
        for file_path in test_files {
            let references = self.search_test_file(
                file_path,
                requirement_id,
                requirement_text,
                &keywords,
            ).await?;
            test_references.extend(references);
        }

        // Calculate coverage percentage
        let coverage_percentage = self.calculate_coverage(&code_references, &test_references);

        Ok(RequirementTrace {
            requirement_id: requirement_id.to_string(),
            requirement_text: requirement_text.to_string(),
            code_references,
            test_references,
            coverage_percentage,
        })
    }

    async fn search_file_for_requirement(
        &self,
        file_path: &Path,
        requirement_id: &str,
        requirement_text: &str,
        keywords: &[String],
        is_test: bool,
    ) -> Result<Vec<CodeReference>> {
        let content = match fs::read_to_string(file_path).await {
            Ok(content) => content,
            Err(_) => return Ok(Vec::new()),
        };

        let mut references = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let mut max_confidence = 0.0;
            let mut match_type = MatchType::KeywordMatch;

            // Check for exact requirement ID match
            if line.contains(requirement_id) {
                max_confidence = 0.95;
                match_type = MatchType::ExactMatch;
            }
            // Check for keyword matches
            else {
                let mut keyword_matches = 0;
                for keyword in keywords {
                    if line.to_lowercase().contains(&keyword.to_lowercase()) {
                        keyword_matches += 1;
                    }
                }

                if keyword_matches > 0 {
                    max_confidence = (keyword_matches as f64 / keywords.len() as f64) * 0.8;
                    match_type = MatchType::KeywordMatch;
                }
            }

            // Check comments for requirement references
            if let Some(extension) = file_path.extension().and_then(|e| e.to_str()) {
                if let Some(comment_regex) = self.comment_patterns.get(extension) {
                    if comment_regex.is_match(line) {
                        for keyword in keywords {
                            if line.to_lowercase().contains(&keyword.to_lowercase()) {
                                max_confidence = max_confidence.max(0.7);
                                match_type = MatchType::CommentMatch;
                            }
                        }
                    }
                }
            }

            if max_confidence > 0.5 {
                references.push(CodeReference {
                    file_path: file_path.to_path_buf(),
                    line_number: line_num + 1,
                    code_snippet: line.to_string(),
                    confidence: max_confidence,
                    match_type,
                });
            }
        }

        Ok(references)
    }

    async fn search_test_file(
        &self,
        file_path: &Path,
        requirement_id: &str,
        requirement_text: &str,
        keywords: &[String],
    ) -> Result<Vec<TestReference>> {
        let content = match fs::read_to_string(file_path).await {
            Ok(content) => content,
            Err(_) => return Ok(Vec::new()),
        };

        let mut references = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Common test function patterns
        let test_patterns = vec![
            Regex::new(r"(?i)fn\s+test_(\w+)").unwrap(),      // Rust
            Regex::new(r"(?i)def\s+test_(\w+)").unwrap(),     // Python
            Regex::new(r"(?i)it\s*\(\s*['\"](.+?)['\"]").unwrap(), // JS/TS
            Regex::new(r"(?i)test\s*\(\s*['\"](.+?)['\"]").unwrap(), // JS/TS
            Regex::new(r"(?i)@Test.*?public\s+void\s+(\w+)").unwrap(), // Java
        ];

        for (line_num, line) in lines.iter().enumerate() {
            let mut confidence = 0.0;
            let mut test_name = String::new();

            // Check if this line contains a test function
            for pattern in &test_patterns {
                if let Some(captures) = pattern.captures(line) {
                    test_name = captures.get(1).map_or(String::new(), |m| m.as_str().to_string());
                    
                    // Check for requirement references in test name or line
                    if line.contains(requirement_id) {
                        confidence = 0.95;
                    } else {
                        let mut keyword_matches = 0;
                        for keyword in keywords {
                            if line.to_lowercase().contains(&keyword.to_lowercase()) {
                                keyword_matches += 1;
                            }
                        }
                        
                        if keyword_matches > 0 {
                            confidence = (keyword_matches as f64 / keywords.len() as f64) * 0.8;
                        }
                    }
                    
                    break;
                }
            }

            if confidence > 0.5 {
                let test_type = self.determine_test_type(file_path, &test_name);
                references.push(TestReference {
                    file_path: file_path.to_path_buf(),
                    test_name,
                    line_number: line_num + 1,
                    test_type,
                    confidence,
                });
            }
        }

        Ok(references)
    }

    fn extract_keywords(&self, requirement_text: &str) -> Vec<String> {
        let mut keywords = Vec::new();
        
        // Extract common action verbs
        let action_words = vec![
            "login", "register", "authenticate", "create", "update", "delete", "save",
            "search", "filter", "sort", "upload", "download", "send", "receive",
            "validate", "verify", "process", "generate", "calculate", "display",
        ];

        for word in action_words {
            if requirement_text.to_lowercase().contains(word) {
                keywords.push(word.to_string());
            }
        }

        // Extract nouns (simplified)
        let noun_patterns = vec![
            Regex::new(r"(?i)\b(user|admin|customer|account|profile|data|file|document|report)\b").unwrap(),
            Regex::new(r"(?i)\b(system|application|interface|dashboard|form|button|field)\b").unwrap(),
        ];

        for pattern in noun_patterns {
            for cap in pattern.captures_iter(requirement_text) {
                if let Some(word) = cap.get(1) {
                    keywords.push(word.as_str().to_lowercase());
                }
            }
        }

        keywords.dedup();
        keywords
    }

    fn determine_test_type(&self, file_path: &Path, test_name: &str) -> TestType {
        let path_str = file_path.to_string_lossy().to_lowercase();
        let test_name_lower = test_name.to_lowercase();

        if path_str.contains("integration") || test_name_lower.contains("integration") {
            TestType::IntegrationTest
        } else if path_str.contains("e2e") || path_str.contains("end-to-end") || 
                  test_name_lower.contains("e2e") || test_name_lower.contains("end_to_end") {
            TestType::EndToEndTest
        } else if path_str.contains("unit") || test_name_lower.contains("unit") {
            TestType::UnitTest
        } else {
            TestType::Unknown
        }
    }

    fn calculate_coverage(&self, code_refs: &[CodeReference], test_refs: &[TestReference]) -> f64 {
        let mut coverage = 0.0;
        
        // Base coverage from code references
        if !code_refs.is_empty() {
            coverage += 40.0;
        }
        
        // Additional coverage from test references
        if !test_refs.is_empty() {
            coverage += 60.0;
        }
        
        // Bonus for high-confidence matches
        let avg_code_confidence: f64 = code_refs.iter().map(|r| r.confidence).sum::<f64>() 
            / code_refs.len().max(1) as f64;
        let avg_test_confidence: f64 = test_refs.iter().map(|r| r.confidence).sum::<f64>() 
            / test_refs.len().max(1) as f64;
        
        coverage *= (avg_code_confidence + avg_test_confidence) / 2.0;
        
        coverage.min(100.0)
    }

    async fn find_orphaned_code(
        &self,
        code_files: &[PathBuf],
        requirements: &[RequirementTrace],
    ) -> Result<Vec<OrphanedCode>> {
        let mut orphaned = Vec::new();
        let mut traced_files: HashSet<PathBuf> = HashSet::new();

        // Collect all files that are traced to requirements
        for req in requirements {
            for code_ref in &req.code_references {
                traced_files.insert(code_ref.file_path.clone());
            }
        }

        // Find files with no requirement traceability
        for file_path in code_files {
            if !traced_files.contains(file_path) {
                if let Ok(content) = fs::read_to_string(file_path).await {
                    // Simple function detection (can be improved for each language)
                    let function_patterns = vec![
                        Regex::new(r"fn\s+(\w+)").unwrap(),        // Rust
                        Regex::new(r"def\s+(\w+)").unwrap(),       // Python  
                        Regex::new(r"function\s+(\w+)").unwrap(),  // JavaScript
                        Regex::new(r"public\s+\w+\s+(\w+)\s*\(").unwrap(), // Java
                    ];

                    for (line_num, line) in content.lines().enumerate() {
                        for pattern in &function_patterns {
                            if let Some(captures) = pattern.captures(line) {
                                if let Some(func_name) = captures.get(1) {
                                    orphaned.push(OrphanedCode {
                                        file_path: file_path.clone(),
                                        function_name: func_name.as_str().to_string(),
                                        line_number: line_num + 1,
                                        description: format!("Function '{}' has no clear requirement traceability", func_name.as_str()),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(orphaned)
    }
}