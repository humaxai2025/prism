use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDiffAnalysis {
    pub from_commit: String,
    pub to_commit: String,
    pub changed_files: Vec<FileChange>,
    pub requirement_changes: Vec<RequirementChange>,
    pub impact_analysis: ImpactAnalysis,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub file_path: PathBuf,
    pub change_type: ChangeType,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub diff_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementChange {
    pub file_path: PathBuf,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub change_type: ChangeType,
    pub impact_score: f64,
    pub affected_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub total_files_changed: usize,
    pub requirement_files_changed: usize,
    pub estimated_impact_score: f64,
    pub high_impact_changes: Vec<String>,
    pub regression_risk: RegressionRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionRisk {
    Low,
    Medium,
    High,
    Critical,
}

pub struct GitIntegration {
    repo_path: PathBuf,
}

impl GitIntegration {
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }

    pub async fn analyze_requirement_changes(
        &self,
        from_commit: &str,
        to_commit: &str,
    ) -> Result<GitDiffAnalysis> {
        // Validate that we're in a git repository
        self.validate_git_repo()?;

        // Get the diff between commits
        let changed_files = self.get_changed_files(from_commit, to_commit)?;
        
        // Filter for requirement-related files
        let requirement_files = self.filter_requirement_files(&changed_files);
        
        // Analyze each requirement file change
        let mut requirement_changes = Vec::new();
        for file_change in &requirement_files {
            let req_change = self.analyze_requirement_file_change(
                &file_change.file_path,
                from_commit,
                to_commit,
            ).await?;
            requirement_changes.push(req_change);
        }

        // Perform impact analysis
        let impact_analysis = self.calculate_impact_analysis(&changed_files, &requirement_changes);

        // Generate recommendations
        let recommendations = self.generate_recommendations(&requirement_changes, &impact_analysis);

        Ok(GitDiffAnalysis {
            from_commit: from_commit.to_string(),
            to_commit: to_commit.to_string(),
            changed_files,
            requirement_changes,
            impact_analysis,
            recommendations,
        })
    }

    pub fn get_current_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(&self.repo_path)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get current branch: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    pub fn get_recent_commits(&self, count: usize) -> Result<Vec<CommitInfo>> {
        let output = Command::new("git")
            .args(&["log", &format!("-{}", count), "--pretty=format:%H|%s|%an|%ad", "--date=iso"])
            .current_dir(&self.repo_path)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get recent commits: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }

        let commits_text = String::from_utf8(output.stdout)?;
        let mut commits = Vec::new();

        for line in commits_text.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                commits.push(CommitInfo {
                    hash: parts[0].to_string(),
                    message: parts[1].to_string(),
                    author: parts[2].to_string(),
                    date: parts[3].to_string(),
                });
            }
        }

        Ok(commits)
    }

    pub fn get_modified_requirements_since_commit(&self, since_commit: &str) -> Result<Vec<PathBuf>> {
        let output = Command::new("git")
            .args(&["diff", "--name-only", since_commit, "HEAD"])
            .current_dir(&self.repo_path)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get modified files: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }

        let files_text = String::from_utf8(output.stdout)?;
        let mut requirement_files = Vec::new();

        for line in files_text.lines() {
            let path = PathBuf::from(line.trim());
            if self.is_requirement_file(&path) {
                requirement_files.push(path);
            }
        }

        Ok(requirement_files)
    }

    fn validate_git_repo(&self) -> Result<()> {
        let git_dir = self.repo_path.join(".git");
        if !git_dir.exists() {
            return Err(anyhow!("Not a git repository: {}", self.repo_path.display()));
        }

        // Check if git command is available
        let output = Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(&self.repo_path)
            .output();

        match output {
            Ok(result) if result.status.success() => Ok(()),
            Ok(_) => Err(anyhow!("Git command failed")),
            Err(_) => Err(anyhow!("Git command not available")),
        }
    }

    fn get_changed_files(&self, from_commit: &str, to_commit: &str) -> Result<Vec<FileChange>> {
        let output = Command::new("git")
            .args(&["diff", "--name-status", from_commit, to_commit])
            .current_dir(&self.repo_path)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get changed files: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }

        let diff_text = String::from_utf8(output.stdout)?;
        let mut changes = Vec::new();

        for line in diff_text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let status = parts[0];
                let file_path = PathBuf::from(parts[1]);

                let change_type = match status {
                    "A" => ChangeType::Added,
                    "M" => ChangeType::Modified,
                    "D" => ChangeType::Deleted,
                    "R100" => ChangeType::Renamed,
                    _ if status.starts_with('R') => ChangeType::Renamed,
                    _ => ChangeType::Modified,
                };

                // Get detailed diff for this file
                let diff_content = self.get_file_diff(&file_path, from_commit, to_commit)?;
                let (lines_added, lines_removed) = self.count_diff_lines(&diff_content);

                changes.push(FileChange {
                    file_path,
                    change_type,
                    lines_added,
                    lines_removed,
                    diff_content,
                });
            }
        }

        Ok(changes)
    }

    fn get_file_diff(&self, file_path: &Path, from_commit: &str, to_commit: &str) -> Result<String> {
        let output = Command::new("git")
            .args(&["diff", from_commit, to_commit, "--", file_path.to_str().unwrap_or("")])
            .current_dir(&self.repo_path)
            .output()?;

        Ok(String::from_utf8(output.stdout).unwrap_or_default())
    }

    fn count_diff_lines(&self, diff_content: &str) -> (usize, usize) {
        let mut added = 0;
        let mut removed = 0;

        for line in diff_content.lines() {
            if line.starts_with('+') && !line.starts_with("+++") {
                added += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                removed += 1;
            }
        }

        (added, removed)
    }

    fn filter_requirement_files(&self, files: &[FileChange]) -> Vec<FileChange> {
        files.iter()
            .filter(|f| self.is_requirement_file(&f.file_path))
            .cloned()
            .collect()
    }

    fn is_requirement_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Check for requirement-related patterns
        path_str.contains("requirement") ||
        path_str.contains("user-story") ||
        path_str.contains("user_story") ||
        path_str.contains("spec") ||
        path_str.contains("feature") ||
        file_name.starts_with("req-") ||
        file_name.starts_with("req_") ||
        file_name.starts_with("us-") ||
        file_name.starts_with("us_") ||
        // Check for supported extensions
        (path.extension().and_then(|e| e.to_str()) == Some("md") && 
         (path_str.contains("req") || path_str.contains("story"))) ||
        path.extension().and_then(|e| e.to_str()) == Some("txt") && 
         (path_str.contains("req") || path_str.contains("story"))
    }

    async fn analyze_requirement_file_change(
        &self,
        file_path: &Path,
        from_commit: &str,
        to_commit: &str,
    ) -> Result<RequirementChange> {
        // Get old and new content
        let old_content = self.get_file_content_at_commit(file_path, from_commit).await?;
        let new_content = self.get_file_content_at_commit(file_path, to_commit).await?;

        let change_type = match (&old_content, &new_content) {
            (None, Some(_)) => ChangeType::Added,
            (Some(_), None) => ChangeType::Deleted,
            (Some(_), Some(_)) => ChangeType::Modified,
            (None, None) => ChangeType::Modified, // Shouldn't happen, but handle gracefully
        };

        // Calculate impact score based on content changes
        let impact_score = self.calculate_change_impact_score(&old_content, &new_content);

        // Extract affected requirements (simplified)
        let affected_requirements = self.extract_requirement_ids(&new_content.as_deref().unwrap_or(""));

        Ok(RequirementChange {
            file_path: file_path.to_path_buf(),
            old_content,
            new_content,
            change_type,
            impact_score,
            affected_requirements,
        })
    }

    async fn get_file_content_at_commit(&self, file_path: &Path, commit: &str) -> Result<Option<String>> {
        let output = Command::new("git")
            .args(&["show", &format!("{}:{}", commit, file_path.display())])
            .current_dir(&self.repo_path)
            .output()?;

        if output.status.success() {
            Ok(Some(String::from_utf8(output.stdout)?))
        } else {
            // File might not exist at this commit
            Ok(None)
        }
    }

    fn calculate_change_impact_score(&self, old_content: &Option<String>, new_content: &Option<String>) -> f64 {
        match (old_content, new_content) {
            (None, Some(_)) => 0.8, // New file - high impact
            (Some(_), None) => 1.0, // Deleted file - highest impact
            (Some(old), Some(new)) => {
                // Calculate based on content similarity
                let old_lines: Vec<&str> = old.lines().collect();
                let new_lines: Vec<&str> = new.lines().collect();
                
                let total_lines = old_lines.len().max(new_lines.len()) as f64;
                if total_lines == 0.0 {
                    return 0.0;
                }
                
                // Simple line-based diff (can be improved with proper diff algorithm)
                let changed_lines = self.count_changed_lines(&old_lines, &new_lines) as f64;
                (changed_lines / total_lines).min(1.0)
            }
            (None, None) => 0.0,
        }
    }

    fn count_changed_lines(&self, old_lines: &[&str], new_lines: &[&str]) -> usize {
        let old_set: std::collections::HashSet<_> = old_lines.iter().collect();
        let new_set: std::collections::HashSet<_> = new_lines.iter().collect();
        
        let added = new_set.difference(&old_set).count();
        let removed = old_set.difference(&new_set).count();
        
        added + removed
    }

    fn extract_requirement_ids(&self, content: &str) -> Vec<String> {
        let mut ids = Vec::new();
        
        // Simple regex patterns for common requirement ID formats
        let patterns = vec![
            regex::Regex::new(r"(?i)req-?(\d+)").unwrap(),
            regex::Regex::new(r"(?i)requirement[_\s-]*(\d+)").unwrap(),
            regex::Regex::new(r"(?i)us-?(\d+)").unwrap(),
            regex::Regex::new(r"(?i)user[_\s-]*story[_\s-]*(\d+)").unwrap(),
        ];

        for pattern in patterns {
            for cap in pattern.captures_iter(content) {
                if let Some(id) = cap.get(1) {
                    ids.push(format!("REQ-{}", id.as_str()));
                }
            }
        }

        ids.sort();
        ids.dedup();
        ids
    }

    fn calculate_impact_analysis(&self, changed_files: &[FileChange], requirement_changes: &[RequirementChange]) -> ImpactAnalysis {
        let total_files_changed = changed_files.len();
        let requirement_files_changed = requirement_changes.len();
        
        // Calculate estimated impact score
        let avg_requirement_impact = if requirement_changes.is_empty() {
            0.0
        } else {
            requirement_changes.iter()
                .map(|rc| rc.impact_score)
                .sum::<f64>() / requirement_changes.len() as f64
        };

        let estimated_impact_score = if total_files_changed == 0 {
            0.0
        } else {
            let requirement_weight = 0.8; // Requirements changes are weighted heavily
            let file_weight = 0.2;
            
            (requirement_files_changed as f64 / total_files_changed as f64 * requirement_weight * avg_requirement_impact) +
            (total_files_changed as f64 / 100.0 * file_weight) // Normalize by arbitrary factor
        }.min(1.0);

        // Identify high-impact changes
        let high_impact_changes = requirement_changes.iter()
            .filter(|rc| rc.impact_score > 0.7)
            .map(|rc| format!("{} (impact: {:.1}%)", rc.file_path.display(), rc.impact_score * 100.0))
            .collect();

        // Determine regression risk
        let regression_risk = if estimated_impact_score > 0.8 {
            RegressionRisk::Critical
        } else if estimated_impact_score > 0.6 {
            RegressionRisk::High
        } else if estimated_impact_score > 0.3 {
            RegressionRisk::Medium
        } else {
            RegressionRisk::Low
        };

        ImpactAnalysis {
            total_files_changed,
            requirement_files_changed,
            estimated_impact_score,
            high_impact_changes,
            regression_risk,
        }
    }

    fn generate_recommendations(&self, requirement_changes: &[RequirementChange], impact_analysis: &ImpactAnalysis) -> Vec<String> {
        let mut recommendations = Vec::new();

        if requirement_changes.is_empty() {
            recommendations.push("No requirement files were changed in this commit range.".to_string());
            return recommendations;
        }

        // High-impact change recommendations
        if !impact_analysis.high_impact_changes.is_empty() {
            recommendations.push("🔴 High-impact requirement changes detected:".to_string());
            for change in &impact_analysis.high_impact_changes {
                recommendations.push(format!("  • Review {}", change));
            }
            recommendations.push("  • Consider running full test suite".to_string());
            recommendations.push("  • Update related documentation".to_string());
        }

        // Regression risk recommendations
        match impact_analysis.regression_risk {
            RegressionRisk::Critical => {
                recommendations.push("🚨 CRITICAL: Very high regression risk detected".to_string());
                recommendations.push("  • Require stakeholder approval before deployment".to_string());
                recommendations.push("  • Perform comprehensive testing".to_string());
                recommendations.push("  • Consider phased rollout".to_string());
            }
            RegressionRisk::High => {
                recommendations.push("⚠️  HIGH: Significant regression risk".to_string());
                recommendations.push("  • Run integration tests".to_string());
                recommendations.push("  • Review with technical lead".to_string());
            }
            RegressionRisk::Medium => {
                recommendations.push("🟡 MEDIUM: Moderate regression risk".to_string());
                recommendations.push("  • Run affected test suites".to_string());
            }
            RegressionRisk::Low => {
                recommendations.push("🟢 LOW: Minimal regression risk".to_string());
            }
        }

        // File-specific recommendations
        for req_change in requirement_changes {
            match req_change.change_type {
                ChangeType::Added => {
                    recommendations.push(format!("📝 New requirement file added: {}", req_change.file_path.display()));
                    recommendations.push("  • Ensure proper review and validation".to_string());
                }
                ChangeType::Deleted => {
                    recommendations.push(format!("🗑️  Requirement file deleted: {}", req_change.file_path.display()));
                    recommendations.push("  • Verify related code/tests are also updated".to_string());
                }
                ChangeType::Modified => {
                    if req_change.impact_score > 0.5 {
                        recommendations.push(format!("✏️  Significant changes to: {}", req_change.file_path.display()));
                        recommendations.push("  • Re-run PRISM analysis".to_string());
                        recommendations.push("  • Update related artifacts".to_string());
                    }
                }
                ChangeType::Renamed => {
                    recommendations.push(format!("📁 Requirement file renamed: {}", req_change.file_path.display()));
                    recommendations.push("  • Update references and links".to_string());
                }
            }
        }

        // General recommendations
        recommendations.push("📋 General recommendations:".to_string());
        recommendations.push("  • Run `prism analyze --dir ./requirements` to validate changes".to_string());
        recommendations.push("  • Update requirement traceability matrix".to_string());
        recommendations.push("  • Consider impact on related systems".to_string());

        recommendations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
}