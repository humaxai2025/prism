// Simple test to verify git integration functionality
use std::process::Command;
use std::path::PathBuf;

pub fn test_git_integration() -> anyhow::Result<()> {
    println!("🧪 Testing Git Integration");
    
    // Test if we're in a git repository
    let git_status = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()?;
    
    if !git_status.status.success() {
        println!("❌ Not in a git repository or git not available");
        return Ok(());
    }
    
    println!("✅ Git repository detected");
    
    // Get current branch
    let branch_output = Command::new("git")
        .args(&["branch", "--show-current"])
        .output()?;
    
    if branch_output.status.success() {
        let branch_str = String::from_utf8_lossy(&branch_output.stdout);
        let branch = branch_str.trim();
        println!("📍 Current branch: {}", branch);
    }
    
    // Get recent commits
    let commit_output = Command::new("git")
        .args(&["log", "-5", "--oneline"])
        .output()?;
    
    if commit_output.status.success() {
        let commits = String::from_utf8_lossy(&commit_output.stdout);
        println!("📜 Recent commits:");
        for line in commits.lines() {
            println!("   {}", line);
        }
    }
    
    // Test file change detection
    let diff_output = Command::new("git")
        .args(&["diff", "--name-status", "HEAD~1", "HEAD"])
        .output()?;
    
    if diff_output.status.success() {
        let changes = String::from_utf8_lossy(&diff_output.stdout);
        if !changes.is_empty() {
            println!("🔄 Files changed in last commit:");
            for line in changes.lines() {
                println!("   {}", line);
            }
        } else {
            println!("📝 No changes in last commit");
        }
    }
    
    // Check for requirement-like files
    let req_files = find_requirement_files()?;
    println!("📋 Found {} potential requirement files:", req_files.len());
    for file in req_files.iter().take(5) { // Show first 5
        println!("   {}", file.display());
    }
    if req_files.len() > 5 {
        println!("   ... and {} more", req_files.len() - 5);
    }
    
    Ok(())
}

fn find_requirement_files() -> anyhow::Result<Vec<PathBuf>> {
    use walkdir::WalkDir;
    
    let mut req_files = Vec::new();
    
    for entry in WalkDir::new(".").follow_links(false).max_depth(3) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let path = entry.path();
            if is_requirement_file(path) {
                req_files.push(path.to_path_buf());
            }
        }
    }
    
    Ok(req_files)
}

fn is_requirement_file(path: &std::path::Path) -> bool {
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
    // README files often contain requirements
    file_name == "readme.md" ||
    // Check for supported extensions with requirement-like names
    (path.extension().and_then(|e| e.to_str()) == Some("md") && 
     (path_str.contains("req") || path_str.contains("story")))
}

// Simple test for git diff functionality
pub fn test_git_diff_simulation(from_commit: &str, to_commit: &str) -> anyhow::Result<()> {
    println!("🔍 Testing git diff simulation: {}..{}", from_commit, to_commit);
    
    // Get changed files
    let diff_output = Command::new("git")
        .args(&["diff", "--name-status", from_commit, to_commit])
        .output()?;
    
    if !diff_output.status.success() {
        println!("❌ Git diff failed: {}", String::from_utf8_lossy(&diff_output.stderr));
        return Ok(());
    }
    
    let changes = String::from_utf8_lossy(&diff_output.stdout);
    let mut requirement_changes = Vec::new();
    let mut total_changes = 0;
    
    for line in changes.lines() {
        if line.trim().is_empty() {
            continue;
        }
        
        total_changes += 1;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let _status = parts[0]; // A, M, D, etc.
            let file_path = PathBuf::from(parts[1]);
            
            if is_requirement_file(&file_path) {
                requirement_changes.push(file_path);
            }
        }
    }
    
    println!("📊 Analysis Results:");
    println!("   Total files changed: {}", total_changes);
    println!("   Requirement files changed: {}", requirement_changes.len());
    
    if !requirement_changes.is_empty() {
        println!("🔄 Changed requirement files:");
        let req_changes_count = requirement_changes.len();
        
        for file in &requirement_changes {
            println!("   📄 {}", file.display());
        }
        
        // Simple impact assessment
        let impact = match req_changes_count {
            0 => "None",
            1..=2 => "Low",
            3..=5 => "Medium",
            _ => "High"
        };
        
        println!("⚖️  Estimated impact: {}", impact);
        
        // Generate simple recommendations
        println!("📋 Recommendations:");
        if requirement_changes.len() > 0 {
            println!("   • Review changed requirements with stakeholders");
            println!("   • Run full PRISM analysis on updated files");
            println!("   • Update related test cases and documentation");
            if requirement_changes.len() > 3 {
                println!("   • Consider staged rollout due to multiple requirement changes");
            }
        }
    } else {
        println!("✅ No requirement files changed - low impact");
    }
    
    Ok(())
}