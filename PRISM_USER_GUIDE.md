# 📘 PRISM User Guide

**Complete Reference for AI-Powered Requirement Analysis**

PRISM is the comprehensive SDLC requirements tool that revolutionizes software requirement management. It analyzes requirements to detect ambiguities with severity levels, generates AI-improved specifications, validates user stories with business value scoring, creates professional UML diagrams (Use Case, Sequence, Class), produces structured pseudocode with business logic, generates comprehensive test cases, suggests non-functional requirements across 8 categories, and performs completeness analysis with gap identification.

**Current Version:** 1.0.0

---

## 📋 Table of Contents

- [🚀 Getting Started](#-getting-started)
- [🔧 Command Reference](#-command-reference)
- [🤖 AI Provider Configuration](#-ai-provider-configuration)
- [📊 Analysis Features](#-analysis-features)
- [📄 Output Formats](#-output-formats)
- [🗂️ File Support](#️-file-support)
- [⚙️ Advanced Usage](#️-advanced-usage)
- [🔄 Integration Examples](#-integration-examples)
- [🛠️ Troubleshooting](#️-troubleshooting)
- [💡 Best Practices](#-best-practices)

---

## 🚀 Getting Started

### Installation

1. **Prerequisites**: Ensure you have Rust installed
2. **Build**: `cargo build --release`
3. **Location**: Executable will be at `./target/release/prism.exe` (Windows) or `./target/release/prism` (Unix)

### First Time Setup

```bash
# Quick start - configure AI provider
prism config --setup

# Launch interactive TUI
prism tui

# Test basic analysis (works without AI)
prism analyze "As a user, I want to login quickly"
```

### Quick Examples

```bash
# Quick requirement improvement
prism improve "As a user, I want to login quickly"

# Smart presets for different analysis levels
prism analyze "As a user, I want to login quickly" --preset basic      # Just analysis + ambiguities
prism analyze "As a user, I want to login quickly" --preset standard   # UML + tests + pseudocode  
prism analyze "As a user, I want to login quickly" --preset full       # All features enabled

# Specialized commands for focused tasks
prism validate "As a user, I want to login" --story --completeness
prism dashboard --file requirements.txt --output dashboard.html
prism trace --from-commit HEAD~5 --to-commit HEAD

# Generate separate artifact files
prism analyze --file story.txt --preset full --save-artifacts "user_story"

# Process multiple files with smart presets
prism analyze --dir ./requirements --preset report --output analysis_report.md

# Interactive AI setup wizard
prism config --setup

# Launch interactive TUI
prism tui

# Get help
prism --help
```

### Configuration File

PRISM stores configuration in `~/.prism/config.yml`:

```yaml
llm:
  api_key: your-api-key
  model: gpt-4
  provider: openai
  base_url: https://api.openai.com/v1/chat/completions
  timeout: 30
analysis:
  custom_rules: []
  ambiguity_threshold: 0.7
  enable_interactive: true
```

---

## 🔧 Command Reference

### `prism analyze`

Analyze requirements using smart presets or custom generation options.

#### Basic Usage with Presets (Recommended)
```bash
prism analyze "As a user, I want to login quickly" --preset basic      # Just analysis + ambiguities
prism analyze --file requirements.txt --preset standard   # UML + tests + pseudocode
prism analyze --dir ./user-stories/ --preset full         # All features enabled
prism analyze --file requirements.txt --preset report     # Optimized for reports
```

#### Input Options (choose one)
- `<TEXT>` - Direct requirement text in quotes
- `--file <PATH>` - Single file (.txt, .md, .rst, .pdf, .docx, .xlsx)
- `--dir <PATH>` - Directory containing requirement files

#### Smart Presets (Recommended)
- `--preset basic` - Just analysis + ambiguity detection
- `--preset standard` - Analysis + UML + tests + pseudocode
- `--preset full` - All generation options (UML, pseudo, tests, improve, NFRs)
- `--preset report` - Analysis optimized for markdown reports

#### Custom Generation Options
- `--generate all` - Generate all artifacts
- `--generate uml` - Generate PlantUML diagrams (Use Case, Sequence, Class)
- `--generate pseudo` - Generate structured pseudocode
- `--generate tests` - Generate comprehensive test cases
- `--generate improve` - Generate improved requirements using AI
- `--generate nfr` - Generate non-functional requirements

#### Output Options
- `--format <FORMAT>` - Output format: json, markdown, github, jira, plain (default: json)
- `--output <FILE>` - Save results to file instead of displaying
- `--save-artifacts <BASE_NAME>` - Save individual artifacts as separate files
- `--pseudo-lang <LANG>` - Pseudocode language style (python, java, generic)

#### Complete Example (New Simplified Approach)
```bash
prism analyze \
  --file user-story.txt \
  --preset full \
  --format markdown \
  --save-artifacts "login_feature" \
  --output full_analysis.md
```

#### Custom Mix Example
```bash
prism analyze \
  --file user-story.txt \
  --generate uml \
  --generate tests \
  --generate improve \
  --format markdown \
  --output custom_analysis.md
```

Both generate up to 5 files:
- `full_analysis.md` - Complete analysis report
- `login_feature_Analysis.md` - Focused analysis
- `login_feature_Req.md` - Improved requirements
- `login_feature_UML.puml` - PlantUML diagrams
- `login_feature_Logic.py` - Pseudocode implementation
- `login_feature_NFR.md` - Non-functional requirements

### `prism improve`

Focus specifically on improving requirement quality by fixing detected ambiguities.

#### Basic Usage
```bash
prism improve "As a user, I want to login quickly"
prism improve --file requirements.txt --output improved.md
prism improve --dir ./stories --format markdown
```

#### Input Options
- `<TEXT>` - Direct requirement text
- `--file <PATH>` - File to improve
- `--dir <PATH>` - Directory to process

#### Output Options
- `--output <FILE>` - Save improved requirements to file
- `--format <FORMAT>` - Output format (default: markdown)

### `prism validate`

Validate user stories and analyze requirement completeness.

#### Basic Usage
```bash
prism validate "As a user, I want to login quickly" --story
prism validate --file requirements.txt --completeness
prism validate --dir ./user-stories --all --output validation-report.md
```

#### Input Options
- `<TEXT>` - Direct requirement text
- `--file <PATH>` - File to validate
- `--dir <PATH>` - Directory to validate

#### Validation Options
- `--story` - Validate user story format and business value
- `--completeness` - Analyze completeness and identify gaps  
- `--all` - Run all validation checks

#### Output Options
- `--output <FILE>` - Save validation results to file
- `--format <FORMAT>` - Output format (default: json)

### `prism trace`

Trace requirements to source code and tests using git integration.

#### Basic Usage
```bash
prism trace --from-commit HEAD~5 --to-commit HEAD
prism trace --file requirements.txt --source-dir ./src --test-dir ./tests
```

#### Input Options
- `<TEXT>` - Requirements text or identifier
- `--file <PATH>` - Requirements file to trace

#### Git Integration
- `--from-commit <HASH>` - Git commit hash to compare from
- `--to-commit <HASH>` - Git commit hash to compare to

#### Directory Integration
- `--source-dir <PATH>` - Source code directory to trace to
- `--test-dir <PATH>` - Test directory to trace to

#### Output Options
- `--output <FILE>` - Save traceability results to file
- `--format <FORMAT>` - Output format (default: json)

### `prism dashboard`

Generate executive dashboards and reports with HTML output.

#### Basic Usage
```bash
prism dashboard --file requirements.txt --output dashboard.html
prism dashboard --dir ./requirements --executive-summary --output executive-report.html
```

#### Input Options
- `<TEXT>` - Requirements text for dashboard
- `--file <PATH>` - File to generate dashboard from
- `--dir <PATH>` - Directory to generate dashboard from

#### Dashboard Options
- `--template <NAME>` - Use custom template
- `--branding <TEXT>` - Add custom branding
- `--executive-summary` - Generate executive summary

#### Output Options
- `--output <FILE>` - Output file for dashboard (required)

### `prism config`

Setup and manage AI configuration with multiple provider support.

#### Interactive Setup (Recommended)
```bash
prism config --setup
```

#### Provider Configuration
```bash
prism config --provider openai --api-key "your-key" --model "gpt-4"
prism config --provider claude --api-key "your-key" --model "claude-3-sonnet"
prism config --provider ollama  # Local AI, no API key needed
```

#### Configuration Management
- `--show` - Display current configuration values
- `--debug` - Show config file location and status
- `--test` - Test current AI configuration
- `--validate-all` - Validate all configuration settings
- `--test-providers` - Test all configured AI providers

### `prism tui`

Launch the interactive Terminal User Interface with tabbed navigation.

```bash
prism tui
```

#### TUI Features
- **📝 Input Tab**: Enter and edit requirement text
- **⚠️ Ambiguities Tab**: Review detected issues with suggestions
- **🎯 Entities Tab**: View extracted actors, actions, and objects
- **📊 Output Tab**: See generated UML diagrams and pseudocode

#### Keyboard Shortcuts
- `q` - Quit application
- `h` - Toggle help
- `i` - Enter editing mode
- `Tab` - Switch between tabs
- `↑/↓` - Navigate lists

### `prism config`

Setup and manage AI configuration for enhanced analysis.

#### Configuration Commands
```bash
# Interactive setup wizard
prism config --setup

# Show current configuration
prism config --show

# Debug configuration issues
prism config --debug

# Test AI connection
prism config --test

# Manual configuration
prism config --api-key "your-key" --model "gpt-4" --provider openai
```

#### Provider Quick Setup
```bash
prism config --provider openai    # Interactive OpenAI setup
prism config --provider gemini    # Interactive Gemini setup
prism config --provider claude    # Interactive Claude setup
prism config --provider azure     # Interactive Azure setup
prism config --provider ollama    # Interactive Ollama setup
```

---

## 🤖 AI Provider Configuration

### OpenAI Configuration

```bash
prism config --provider openai
# Enter API key when prompted
# Select from: gpt-4, gpt-3.5-turbo, gpt-4o
```

**Models Available:**
- `gpt-4` - Most capable, best results
- `gpt-4o` - Optimized version of GPT-4
- `gpt-3.5-turbo` - Faster, cost-effective

**API Key:** Get from https://platform.openai.com/api-keys

### Google Gemini Configuration

```bash
prism config --provider gemini
# Enter API key when prompted
# Select from: gemini-1.5-pro, gemini-1.5-flash
```

**Models Available:**
- `gemini-1.5-pro` - Best performance
- `gemini-1.5-flash` - Faster responses

**API Key:** Get from https://aistudio.google.com/

### Anthropic Claude Configuration

```bash
prism config --provider claude
# Enter API key when prompted
# Select from: claude-3-opus, claude-3-sonnet, claude-3-haiku
```

**Models Available:**
- `claude-3-opus` - Most capable
- `claude-3-sonnet` - Balanced performance
- `claude-3-haiku` - Fastest

**API Key:** Get from https://console.anthropic.com/

### Azure OpenAI Configuration

```bash
prism config --provider azure
# Enter API key and endpoint URL when prompted
```

**Configuration Requirements:**
- API Key from Azure portal
- Endpoint URL: `https://your-resource.openai.azure.com/openai/deployments/your-deployment`
- Model deployment name

### Local Ollama Configuration

```bash
prism config --provider ollama
# No API key required
# Automatically detects available models
```

**Setup Requirements:**
1. Install Ollama: https://ollama.ai
2. Start Ollama: `ollama serve`
3. Pull models: `ollama pull llama3.1:latest`

**Popular Models:**
- `llama3.1:latest` - Meta's latest model
- `llama3.1:8b` - Smaller, faster version
- `gemma2:latest` - Google's Gemma model
- `qwen2.5-coder:latest` - Code-focused model
- `phi3:mini` - Microsoft's compact model

---

## 📊 Analysis Features

### Ambiguity Detection

PRISM detects multiple types of ambiguities with severity levels:

#### Severity Levels
- **🔴 Critical** - Major issues that block implementation
- **🟠 High** - Significant issues requiring immediate attention
- **🟡 Medium** - Moderate issues that should be addressed
- **🟢 Low** - Minor improvements suggested

#### Detection Categories
- **Vague Terms**: "fast", "quickly", "user-friendly", "robust"
- **Passive Voice**: "should be done", "will be handled"
- **Missing Actors**: Who performs the actions?
- **Undefined Success Criteria**: What defines success?
- **Ambiguous Quantities**: "many", "few", "some"

### Entity Extraction

PRISM identifies and categorizes key entities:

#### Actors (Who)
- Users, administrators, systems, services
- Role-based identification
- Permission and responsibility mapping

#### Actions (What)
- Verbs and operations: create, update, delete, login
- Business processes and workflows
- User interactions and system operations

#### Objects (What On)
- Data entities: accounts, profiles, documents
- System components: databases, APIs, services
- Business objects: orders, products, categories

### Completeness Analysis

Identify gaps and missing requirements with scoring:

#### Gap Categories
- **Actor Definition**: Missing user roles (Critical/High priority)
- **Acceptance Criteria**: Undefined success conditions (High/Medium priority)
- **Non-Functional Requirements**: Performance, security considerations (Medium priority)
- **Error Handling**: Exception scenarios (Medium/Low priority)
- **Business Rules**: Validation logic (Low priority)

#### Completeness Scoring
- Numerical score (0-100%)
- Gap identification with priorities
- Specific recommendations
- Missing component analysis

### User Story Validation

Validate user story format and business value:

#### Format Validation
- Proper "As a...I want...So that" structure
- Actor quality assessment (0-100% score)
- Goal clarity evaluation (0-100% score)
- Business reason validation (0-100% score)

#### Business Value Scoring
- Quantified value assessment (0-100%)
- Benefit identification keywords
- ROI and impact indicators
- Priority recommendations

#### Component Analysis
- **Actor Quality**: Role specificity and clarity
- **Goal Quality**: Action clarity and measurability
- **Reason Quality**: Business value articulation

### Non-Functional Requirements (NFR)

Automated NFR generation across 8 categories:

#### 🔒 Security
- Authentication requirements (MFA, password complexity)
- Authorization patterns (role-based access)
- Data protection (encryption, audit trails)
- Input validation and sanitization

#### ⚡ Performance
- Response time requirements (< 2 seconds)
- Throughput specifications (requests/second)
- Scalability considerations (concurrent users)
- Resource utilization (CPU, memory)

#### 👤 Usability
- User experience requirements (intuitive interface)
- Accessibility standards (WCAG compliance)
- Interface guidelines (responsive design)
- Error message clarity and help text

#### 🛡️ Reliability
- Availability requirements (99.9% uptime)
- Error recovery mechanisms
- Fault tolerance patterns
- Data consistency guarantees

#### 📈 Scalability
- Growth projections (user/data volume)
- Load handling capabilities
- Resource scaling strategies
- Performance under load

#### 🔧 Maintainability
- Code quality standards (documentation)
- Change management processes
- Technical debt prevention
- Monitoring and logging

#### 🔗 Compatibility
- Platform support (browsers, OS)
- Integration requirements (APIs)
- Backward compatibility
- Version management

#### ♿ Accessibility
- Screen reader support
- Keyboard navigation
- Visual accessibility (contrast, fonts)
- Language localization

### UML Diagram Generation

PRISM generates professional PlantUML diagrams:

#### Use Case Diagrams
- Smart actor-action relationship mapping
- System boundary identification
- Include/extend relationships
- Professional styling with AWS Orange theme

#### Sequence Diagrams
- Complete interaction flows with lifelines
- Authentication and validation patterns
- Database interaction steps
- Error handling scenarios
- Alternative paths for failures

#### Class Diagrams
- Entity classes with attributes and methods
- Service classes for business logic
- Enums and value objects (Status, Priority)
- Relationships and dependencies
- Method signatures with parameters

### Pseudocode Generation

Structured code foundations in multiple languages:

#### Python Style
```python
@dataclass
class User:
    id: str
    status: Status = Status.PENDING
    
    def authenticate(self, credentials: Dict) -> bool:
        # Validate credentials against data source
        # Generate session token
        # Load user permissions
        pass

def login_user(actor, credentials, **kwargs) -> Dict:
    # Step 1: Validate preconditions
    # Step 2: Check permissions
    # Step 3: Execute business logic with error handling
    # Step 4: Log action for audit trail
    pass
```

#### Java Style
```java
class User {
    private String id;
    private Status status;
    
    public boolean authenticate(Credentials credentials) {
        // Permission validation
        // Session token generation
        // Business logic execution
        return isValid;
    }
}

class BusinessLogicService {
    public Result loginUser(Actor actor, Object targetObject, Map<String, Object> parameters) {
        // Comprehensive permission checks
        // Error handling and logging
        // State management
        return Result.success(result);
    }
}
```

### Test Case Generation

Comprehensive test coverage across three categories:

#### Happy Path Tests
- Successful execution scenarios
- Normal user workflows
- Expected positive outcomes
- Standard business rules validation

#### Negative Test Cases
- Invalid input handling
- Authorization failures
- Business rule violations
- Error condition responses
- Boundary value violations

#### Edge Cases
- Null/empty input handling
- Maximum input sizes
- Concurrent access scenarios
- Network failure conditions
- Database connection issues

---

## 📄 Output Formats

### JSON Format

Structured data format for programmatic processing:

```json
{
  "ambiguities": [
    {
      "text": "quickly",
      "reason": "Vague or subjective term that lacks specific criteria",
      "suggestions": ["Define specific metrics or thresholds"],
      "severity": "Medium"
    }
  ],
  "entities": {
    "actors": ["user"],
    "actions": ["want to", "login"],
    "objects": ["account"]
  },
  "completeness_analysis": {
    "completeness_score": 65.0,
    "gaps_identified": [
      {
        "category": "Acceptance Criteria",
        "description": "Missing clear success criteria",
        "suggestions": ["Add Given-When-Then scenarios"],
        "priority": "High"
      }
    ]
  },
  "user_story_validation": {
    "is_valid_format": true,
    "business_value_score": 45.0,
    "actor_quality": {
      "score": 80.0,
      "is_valid": true
    }
  },
  "nfr_suggestions": [
    {
      "category": "Performance",
      "requirement": "Authentication shall complete within 2 seconds",
      "priority": "MustHave"
    }
  ]
}
```

### Markdown Format

Human-readable format with rich formatting:

```markdown
# 🔍 PRISM Requirement Analysis Report

## 📝 Analyzed Requirement
> As a user, I want to login quickly

## 📊 Analysis Summary
- **Ambiguities Found:** 1
- **Completeness Score:** 65%
- **Business Value Score:** 45%

## ⚠️ Detected Ambiguities
### 🟡 Issue #1: "quickly"
- **Problem:** Vague or subjective term that lacks specific criteria
- **Severity:** Medium
- **Suggested Improvements:**
  - Define specific metrics (e.g., "within 2 seconds")
  - Provide measurable performance criteria

## 🔒 Non-Functional Requirements
### ⚡ Performance
**🟠 Should Have**
**Requirement:** Authentication process shall complete within 2 seconds
**Rationale:** Users expect quick login for good experience
```

### GitHub Format

Optimized for GitHub Issues and Pull Requests:

```markdown
## :warning: Detected Ambiguities

### :yellow_circle: quickly
**Reason:** Vague term lacking specific criteria

**Suggestions:**
- Define specific metrics (e.g., "within 2 seconds")
- Provide measurable performance criteria

## :white_check_mark: Test Cases Checklist

### Happy Path
- [ ] Test successful user login with valid credentials
- [ ] Test login response time under normal conditions

### Negative Cases  
- [ ] Test login with invalid credentials
- [ ] Test login without proper authorization

### Edge Cases
- [ ] Test login with empty/null credentials
- [ ] Test login with maximum input size
```

### Jira Format

Compatible with Jira ticket formatting:

```
h1. 🔍 PRISM Analysis Report

h2. ⚠️ Detected Ambiguities
h3. 🟡 Issue #1: "quickly"
* *Problem:* Vague term lacking specific criteria
* *Severity:* Medium
* *Suggestions:*
** Define specific metrics (e.g., "within 2 seconds")

h2. ✅ Test Cases
h3. Happy Path Tests
- [ ] Test successful user login
- [ ] Test login performance

h3. Negative Cases
- [ ] Test invalid credentials
- [ ] Test authorization failures
```

### Plain Text Format

Simple text output for basic environments:

```
REQUIREMENT ANALYSIS REPORT
===========================

ANALYZED REQUIREMENT:
As a user, I want to login quickly

ANALYSIS SUMMARY:
- Ambiguities Found: 1
- Completeness Score: 65%
- Business Value Score: 45%

DETECTED AMBIGUITIES:
1. "quickly"
   Reason: Vague term lacking specific criteria
   Severity: Medium
   Suggestions:
   - Define specific metrics (e.g., "within 2 seconds")
   - Provide measurable performance criteria

NON-FUNCTIONAL REQUIREMENTS:
Performance:
- Authentication process shall complete within 2 seconds (Should Have)
  Rationale: Users expect quick login for good experience
```

---

## 🗂️ File Support

PRISM supports multiple file formats for requirement input:

### Text Files
- `.txt` - Plain text files
- `.md` - Markdown files with formatting
- `.rst` - reStructuredText files

### Document Files
- `.pdf` - PDF documents (text extraction)
- `.docx` - Microsoft Word documents
- `.xlsx` - Excel spreadsheets (text content)

### Usage Examples
```bash
# Single file analysis
prism analyze --file requirements.txt
prism analyze --file user_stories.md  
prism analyze --file specifications.pdf
prism analyze --file backlog.xlsx

# Directory processing (all supported formats)
prism analyze --dir ./documentation/
prism analyze --dir ./user-stories/ --format markdown --output report.md
```

### File Processing Features
- **Automatic Format Detection**: PRISM detects file types automatically
- **Text Extraction**: Extracts readable text from PDF, DOCX, and XLSX files
- **Directory Scanning**: Recursively processes all supported files in directories
- **Content Aggregation**: Combines multiple files into unified analysis
- **File Progress**: Shows processing progress for large directories
- **Error Handling**: Gracefully handles unreadable or corrupted files

---

## ⚙️ Advanced Usage

### Individual Artifact Generation

Save each analysis component as separate files:

```bash
# Complete artifact generation
prism analyze --file user_story.txt \
  --uml --pseudo --tests --improve --completeness --nfr \
  --save-artifacts "feature_name"
```

**Generated Files:**
- `feature_name_Analysis.md` - Complete analysis report
- `feature_name_Req.md` - Improved requirements only
- `feature_name_UML.puml` - PlantUML diagrams
- `feature_name_Logic.py` - Structured pseudocode
- `feature_name.nfr` - Non-functional requirements

### Batch Processing

Process multiple requirements efficiently:

```bash
#!/bin/bash
# Process all user stories in directory
for file in user-stories/*.md; do
  base=$(basename "$file" .md)
  prism analyze --file "$file" \
    --improve --uml --pseudo \
    --save-artifacts "stories/$base"
done
```

### Custom Analysis Workflows

```bash
# 1. Quick validation check
prism analyze --dir ./requirements \
  --validate-story --completeness \
  --format plain --output validation.log

# 2. Design artifact generation  
prism analyze --file epic.txt \
  --uml --pseudo --tests \
  --save-artifacts "epic_design"

# 3. Complete documentation pipeline
prism analyze --file requirements.txt \
  --uml --pseudo --tests --improve --completeness --nfr \
  --format markdown --save-artifacts "project_docs" \
  --output comprehensive_analysis.md
```

### Environment Configuration

Set environment variables for automation:

```bash
export PRISM_API_KEY="your-api-key"
export PRISM_MODEL="gpt-4"
export PRISM_PROVIDER="openai"

# Run analysis without manual configuration
prism analyze --file requirements.txt --improve
```

### Performance Optimization

```bash
# Fast analysis with efficient models
prism config --model "gpt-3.5-turbo" --timeout 15

# Batch directory processing
prism analyze --dir ./requirements \
  --format json --output batch_results.json

# Parallel processing script
find ./stories -name "*.md" | xargs -P 4 -I {} \
  prism analyze --file {} --improve --output {}.improved
```

---

## 🔄 Integration Examples

### CI/CD Pipeline Integration

#### GitHub Actions
```yaml
name: Requirements Analysis
on: 
  push:
    paths: ['requirements/**', 'user-stories/**']
  pull_request:
    paths: ['requirements/**', 'user-stories/**']

jobs:
  analyze-requirements:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build PRISM
        run: cargo build --release
      - name: Analyze Requirements
        env:
          PRISM_API_KEY: ${{ secrets.OPENAI_API_KEY }}
        run: |
          ./target/release/prism analyze \
            --dir ./requirements \
            --completeness --validate-story \
            --format github --output analysis.md
      - name: Check Analysis Results
        run: |
          if grep -q "Critical\|High" analysis.md; then
            echo "❌ Critical or High severity issues found!"
            cat analysis.md
            exit 1
          else
            echo "✅ Requirements analysis passed!"
          fi
      - name: Upload Analysis Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: requirements-analysis
          path: |
            analysis.md
            *_Analysis.md
            *_UML.puml
            *_Logic.py
```

#### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "🔍 Validating requirements before commit..."

# Check if requirements directory exists
if [ -d "./requirements" ]; then
  ./target/release/prism analyze \
    --dir ./requirements \
    --validate-story --completeness \
    --format plain --output /tmp/req_validation.log
  
  # Check for critical issues
  if grep -q "Critical" /tmp/req_validation.log; then
    echo "❌ Critical issues found in requirements!"
    echo "Please fix before committing:"
    grep -A 3 "Critical" /tmp/req_validation.log
    exit 1
  fi
  
  # Check completeness score
  score=$(grep "Completeness Score:" /tmp/req_validation.log | grep -o '[0-9]*\.[0-9]*')
  if (( $(echo "$score < 70" | bc -l) )); then
    echo "⚠️ Completeness score below 70%: $score%"
    echo "Consider improving requirements before committing"
    exit 1
  fi
fi

echo "✅ Requirements validation passed!"
```

#### Jenkins Pipeline
```groovy
pipeline {
    agent any
    
    environment {
        PRISM_API_KEY = credentials('openai-api-key')
    }
    
    stages {
        stage('Build PRISM') {
            steps {
                sh 'cargo build --release'
            }
        }
        
        stage('Validate Requirements') {
            steps {
                sh '''
                    ./target/release/prism analyze \
                        --dir ./requirements \
                        --validate-story --completeness \
                        --format plain --output validation_report.txt
                '''
            }
        }
        
        stage('Generate Documentation') {
            steps {
                sh '''
                    ./target/release/prism analyze \
                        --dir ./requirements \
                        --uml --pseudo --improve \
                        --save-artifacts "release_$(date +%Y%m%d)" \
                        --format markdown --output release_docs.md
                '''
                
                archiveArtifacts artifacts: '*_Analysis.md,*_UML.puml,*_Logic.py,release_docs.md'
            }
        }
        
        stage('Quality Gate') {
            steps {
                script {
                    def validationReport = readFile 'validation_report.txt'
                    if (validationReport.contains('Critical')) {
                        error('Critical issues found in requirements')
                    }
                }
            }
        }
    }
}
```

### Build Script Integration

```bash
#!/bin/bash
# build.sh - Complete build pipeline with requirements validation

set -e

echo "🔍 PRISM Requirements Validation Pipeline"
echo "========================================"

# Step 1: Validate all requirements
echo "📋 Step 1: Validating requirements..."
./target/release/prism analyze \
  --dir ./requirements \
  --validate-story --completeness \
  --format plain --output validation.log

# Check for blocking issues
if grep -q "Critical\|High" validation.log; then
  echo "❌ Blocking issues found in requirements!"
  echo "Please review and fix before proceeding:"
  grep -A 2 "Critical\|High" validation.log
  exit 1
fi

# Step 2: Generate design artifacts
echo "🎨 Step 2: Generating design artifacts..."
./target/release/prism analyze \
  --dir ./requirements \
  --uml --pseudo --tests \
  --save-artifacts "build_$(date +%Y%m%d_%H%M%S)"

# Step 3: Create documentation
echo "📚 Step 3: Creating comprehensive documentation..."
./target/release/prism analyze \
  --dir ./requirements \
  --uml --pseudo --tests --improve --completeness --nfr \
  --format markdown --output docs/requirements_analysis.md

# Step 4: Build application
echo "🏗️ Step 4: Building application..."
cargo build --release

# Step 5: Run tests
echo "🧪 Step 5: Running tests..."
cargo test

echo "✅ Build pipeline completed successfully!"
echo "📄 Documentation: docs/requirements_analysis.md"
echo "🎨 Design artifacts: *_UML.puml, *_Logic.py"
```

### Documentation Generation Automation

```bash
#!/bin/bash
# generate-project-docs.sh

PROJECT_NAME="MyProject"
DATE=$(date +%Y%m%d)
DOCS_DIR="docs/generated"

echo "📚 Generating comprehensive project documentation for $PROJECT_NAME"
echo "=================================================================="

# Create documentation directory
mkdir -p "$DOCS_DIR"

# Process each requirement category
categories=("user-stories" "epics" "features" "nfr")

for category in "${categories[@]}"; do
  if [ -d "./$category" ]; then
    echo "📝 Processing $category..."
    
    # Generate comprehensive analysis
    ./target/release/prism analyze \
      --dir "./$category" \
      --uml --pseudo --tests --improve --completeness --nfr \
      --format markdown \
      --save-artifacts "$DOCS_DIR/${category}_${DATE}" \
      --output "$DOCS_DIR/${category}_analysis.md"
    
    echo "✅ $category documentation generated"
  fi
done

# Generate project overview
echo "📊 Generating project overview..."
./target/release/prism analyze \
  --dir "./requirements" \
  --completeness --validate-story \
  --format markdown \
  --output "$DOCS_DIR/project_overview.md"

# Create index file
cat > "$DOCS_DIR/README.md" << EOF
# $PROJECT_NAME - Requirements Documentation

Generated on $(date)

## 📁 Documentation Structure

### Analysis Reports
$(for category in "${categories[@]}"; do
  if [ -f "$DOCS_DIR/${category}_analysis.md" ]; then
    echo "- [${category^} Analysis](./${category}_analysis.md)"
  fi
done)

### Design Artifacts
$(find "$DOCS_DIR" -name "*.puml" -exec basename {} \; | sed 's/^/- /')

### Implementation Guides  
$(find "$DOCS_DIR" -name "*_Logic.py" -exec basename {} \; | sed 's/^/- /')

### Project Overview
- [Project Overview](./project_overview.md)

---
*Generated with PRISM - AI-Powered Requirement Analyzer*
EOF

echo "🎉 Documentation generation complete!"
echo "📍 Location: $DOCS_DIR/"
echo "🔗 Index: $DOCS_DIR/README.md"
```

### API Integration Examples

```python
# requirements_validator.py
import subprocess
import json
import sys
from pathlib import Path

class PrismValidator:
    def __init__(self, prism_path="./target/release/prism"):
        self.prism_path = prism_path
    
    def validate_requirements(self, req_dir):
        """Validate all requirements in directory"""
        cmd = [
            self.prism_path, "analyze",
            "--dir", str(req_dir),
            "--validate-story", "--completeness",
            "--format", "json"
        ]
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        if result.returncode == 0:
            return json.loads(result.stdout)
        else:
            raise Exception(f"PRISM validation failed: {result.stderr}")
    
    def get_quality_metrics(self, analysis_result):
        """Extract quality metrics from analysis"""
        metrics = {
            "ambiguity_count": len(analysis_result.get("ambiguities", [])),
            "completeness_score": 0,
            "business_value_score": 0,
            "critical_issues": 0,
            "high_issues": 0
        }
        
        # Extract completeness score
        if "completeness_analysis" in analysis_result:
            metrics["completeness_score"] = analysis_result["completeness_analysis"]["completeness_score"]
        
        # Extract business value score
        if "user_story_validation" in analysis_result:
            metrics["business_value_score"] = analysis_result["user_story_validation"]["business_value_score"]
        
        # Count severity levels
        for ambiguity in analysis_result.get("ambiguities", []):
            severity = ambiguity.get("severity", "").lower()
            if severity == "critical":
                metrics["critical_issues"] += 1
            elif severity == "high":
                metrics["high_issues"] += 1
        
        return metrics
    
    def check_quality_gates(self, metrics):
        """Check if requirements meet quality gates"""
        gates = {
            "min_completeness": 70,
            "min_business_value": 50,
            "max_critical_issues": 0,
            "max_high_issues": 2
        }
        
        issues = []
        
        if metrics["completeness_score"] < gates["min_completeness"]:
            issues.append(f"Completeness score too low: {metrics['completeness_score']}% < {gates['min_completeness']}%")
        
        if metrics["business_value_score"] < gates["min_business_value"]:
            issues.append(f"Business value score too low: {metrics['business_value_score']}% < {gates['min_business_value']}%")
        
        if metrics["critical_issues"] > gates["max_critical_issues"]:
            issues.append(f"Too many critical issues: {metrics['critical_issues']} > {gates['max_critical_issues']}")
        
        if metrics["high_issues"] > gates["max_high_issues"]:
            issues.append(f"Too many high severity issues: {metrics['high_issues']} > {gates['max_high_issues']}")
        
        return len(issues) == 0, issues

if __name__ == "__main__":
    validator = PrismValidator()
    
    # Validate requirements
    try:
        analysis = validator.validate_requirements("./requirements")
        metrics = validator.get_quality_metrics(analysis)
        passed, issues = validator.check_quality_gates(metrics)
        
        print(f"📊 Requirements Quality Metrics:")
        print(f"   Completeness: {metrics['completeness_score']:.1f}%")
        print(f"   Business Value: {metrics['business_value_score']:.1f}%")
        print(f"   Critical Issues: {metrics['critical_issues']}")
        print(f"   High Issues: {metrics['high_issues']}")
        
        if passed:
            print("✅ All quality gates passed!")
            sys.exit(0)
        else:
            print("❌ Quality gate failures:")
            for issue in issues:
                print(f"   - {issue}")
            sys.exit(1)
            
    except Exception as e:
        print(f"❌ Validation failed: {e}")
        sys.exit(1)
```

---

## 🛠️ Troubleshooting

### Common Issues and Solutions

#### AI Configuration Issues

**Problem**: "AI not configured" message
```
❌ AI is not configured
💡 Run 'prism config --setup' to configure AI features
```

**Solutions**:
1. Run the setup wizard: `prism config --setup`
2. Manual configuration: `prism config --api-key "your-key" --provider openai`
3. Check configuration: `prism config --show`
4. Debug config file: `prism config --debug`

#### API Connection Failures

**Problem**: "AI connection failed" with API errors

**For OpenAI**:
1. Verify API key is correct and active
2. Check account has sufficient credits
3. Verify model name: `prism config --model gpt-4`
4. Test connection: `prism config --test`
5. Check rate limits and usage quotas

**For Gemini**:
1. Verify API key from Google AI Studio
2. Check API is enabled in Google Cloud Console
3. Verify model name: `prism config --model gemini-1.5-pro`
4. Check regional availability

**For Claude**:
1. Verify API key from Anthropic Console
2. Check account has sufficient credits
3. Verify model name: `prism config --model claude-3-sonnet-20240229`
4. Check API rate limits

**For Ollama**:
1. Ensure Ollama is running: `ollama serve`
2. Check if model exists: `ollama list`
3. Pull model if needed: `ollama pull llama3.1:latest`
4. Verify server URL: `prism config --debug`
5. Check model compatibility

#### File Processing Issues

**Problem**: "No readable files found in directory"

**Solutions**:
1. Check file extensions are supported: `.txt`, `.md`, `.rst`, `.pdf`, `.docx`, `.xlsx`
2. Verify file permissions are readable
3. Check directory path is correct and exists
4. Use absolute paths if relative paths fail
5. Test with single file first: `prism analyze --file test.txt`

**Problem**: "Failed to extract text from file"

**Solutions**:
1. Verify file is not corrupted
2. Check file format is supported
3. For PDF files: ensure text is selectable (not scanned image)
4. For DOCX files: check file is valid Word document
5. Try converting file to plain text format

#### Configuration File Issues

**Problem**: Config file corruption or missing

**Solutions**:
1. Check config location: `prism config --debug`
2. Reset config: Delete `~/.prism/config.yml` and run `prism config --setup`
3. Manual config creation:
```yaml
llm:
  api_key: "your-key"
  model: "gpt-4"
  provider: "openai"
  base_url: null
  timeout: 30
analysis:
  custom_rules: []
  ambiguity_threshold: 0.7
  enable_interactive: true
```
4. Check file permissions on config directory

#### Performance Issues

**Problem**: Slow analysis or timeouts

**Solutions**:
1. Increase timeout: Edit `~/.prism/config.yml`, set `timeout: 60`
2. Use faster models:
   - OpenAI: `gpt-3.5-turbo`
   - Gemini: `gemini-1.5-flash`
   - Claude: `claude-3-haiku`
3. Process smaller chunks: Split large files
4. Use local models: Ollama with smaller models (`phi3:mini`)
5. Check network connection stability

#### Memory and Resource Issues

**Problem**: Out of memory or high resource usage

**Solutions**:
1. Process files individually instead of directories
2. Use lighter AI models (haiku, flash variants)
3. Reduce analysis scope (disable --uml, --pseudo, --tests)
4. Close other resource-intensive applications
5. Increase system swap space if needed

### Debug Commands

```bash
# Check complete configuration status
prism config --debug

# Test AI connection with simple prompt
prism config --test

# Show current configuration values
prism config --show

# Verify file can be processed
prism analyze --file test.txt --format plain

# Test directory processing
prism analyze --dir ./test-requirements --format plain --output debug.log

# Check executable permissions
ls -la ./target/release/prism*
```

### Environment Variables

PRISM supports environment variables for configuration:

```bash
# API Configuration
export PRISM_API_KEY="your-api-key"
export PRISM_MODEL="gpt-4"  
export PRISM_PROVIDER="openai"
export PRISM_TIMEOUT="30"

# Debug Configuration
export PRISM_LOG_LEVEL="debug"  # If implemented
export PRISM_CONFIG_PATH="./custom-config.yml"  # Custom config location
```

### Logging and Debug Output

Enable verbose logging for troubleshooting:

```bash
# Current behavior shows error messages for failed operations
# AI failures are reported with specific provider troubleshooting

# Example debug workflow:
prism config --debug  # Check configuration
prism config --test   # Test AI connection
prism analyze "simple test" --format plain  # Basic functionality test
```

### Provider-Specific Troubleshooting

#### OpenAI Issues
```bash
# Test API key validity
curl -H "Authorization: Bearer $PRISM_API_KEY" \
     -H "Content-Type: application/json" \
     "https://api.openai.com/v1/models"

# Common issues:
# - Invalid API key format (should start with sk-)
# - Insufficient credits
# - Rate limiting
# - Model not available for account type
```

#### Gemini Issues  
```bash
# Test API key
curl "https://generativelanguage.googleapis.com/v1beta/models?key=$PRISM_API_KEY"

# Common issues:
# - API not enabled in Google Cloud
# - Invalid API key format
# - Regional restrictions
# - Quota exceeded
```

#### Claude Issues
```bash
# Test API key (requires more complex curl)
# Common issues:
# - Invalid API key format
# - Account not approved for API access
# - Rate limiting
# - Model not available
```

#### Ollama Issues
```bash
# Check Ollama status
ollama list
ollama ps
curl http://localhost:11434/api/tags

# Common issues:
# - Ollama not running (ollama serve)
# - Model not downloaded (ollama pull model-name)
# - Port conflicts (default 11434)
# - Insufficient disk space for models
```

---

## 💡 Best Practices

### Requirement Writing Guidelines

#### Use Specific Language
```bash
# ❌ Vague
"The system should be fast and user-friendly"

# ✅ Specific  
"The system shall respond to user queries within 2 seconds and provide contextual help tooltips on all form fields"
```

#### Define Clear Actors
```bash
# ❌ Unclear
"Someone should be able to create reports"

# ✅ Clear
"As a business analyst, I should be able to create monthly sales reports with filtering by region and date range"
```

#### Include Success Criteria
```bash
# ❌ Missing criteria
"As a user, I want to login"

# ✅ With criteria  
"As a user, I want to login using my email and password so that I can access my personalized dashboard within 3 seconds, with the system remembering my preferences"
```

#### Add Acceptance Criteria
```bash
# ✅ Complete user story with acceptance criteria
"As a customer support agent, I want to search for customer records by phone number so that I can quickly assist callers.

Acceptance Criteria:
- Given a valid phone number, When I enter it in the search field, Then the system displays the customer record within 1 second
- Given an invalid phone number format, When I attempt to search, Then the system shows a clear error message
- Given no matching records, When I search, Then the system suggests alternative search options"
```

### Analysis Workflow Best Practices

#### 1. Start with Basic Analysis
```bash
# Begin with simple analysis to understand current state
prism analyze --file requirements.txt --format markdown
```

#### 2. Validate User Stories
```bash
# Check format and business value
prism analyze --file user_stories.txt --validate-story --completeness --format markdown
```

#### 3. Identify Gaps
```bash
# Comprehensive gap analysis  
prism analyze --file requirements.txt --completeness --nfr --format markdown
```

#### 4. Generate Design Artifacts
```bash
# Create UML and pseudocode for design
prism analyze --file requirements.txt --uml --pseudo --tests --save-artifacts "design"
```

#### 5. Improve Quality
```bash
# Generate improved version
prism improve --file requirements.txt --output improved_requirements.md
```

#### 6. Create Complete Documentation
```bash
# Generate comprehensive documentation package
prism analyze --file requirements.txt \
  --uml --pseudo --tests --improve --completeness --nfr \
  --save-artifacts "project_v1" \
  --format markdown --output comprehensive_analysis.md
```

### File Organization Best Practices

#### Project Structure
```
project/
├── requirements/
│   ├── user-stories/
│   │   ├── US001_user_login.md
│   │   ├── US002_password_reset.md  
│   │   └── US003_profile_management.md
│   ├── epics/
│   │   ├── E001_authentication_system.md
│   │   └── E002_user_management.md
│   ├── nfr/
│   │   ├── performance_requirements.md
│   │   ├── security_requirements.md
│   │   └── usability_requirements.md
│   └── analysis/
│       ├── US001_Analysis.md
│       ├── US001_UML.puml
│       ├── US001_Logic.py
│       ├── E001_Analysis.md
│       └── project_overview.md
├── docs/
│   └── generated/
└── scripts/
    ├── analyze_requirements.sh
    └── generate_docs.sh
```

#### Naming Conventions
```bash
# User Story Files
US001_user_login.md              # User Story 001: User Login
US002_password_reset.md           # User Story 002: Password Reset

# Epic Files  
E001_authentication_system.md    # Epic 001: Authentication System

# Artifact Files (generated by --save-artifacts)
US001_Analysis.md                 # Analysis report
US001_Req.md                     # Improved requirements  
US001_UML.puml                   # UML diagrams
US001_Logic.py                   # Pseudocode implementation
US001.nfr                        # Non-functional requirements

# Analysis Reports
project_analysis_20241201.md     # Dated analysis reports
sprint_3_requirements_analysis.md # Sprint-specific analysis
```

#### Content Templates

**User Story Template:**
```markdown
# US001: User Login

## User Story
As a registered user, I want to login using my email and password so that I can access my personalized dashboard and saved preferences.

## Acceptance Criteria
- Given valid credentials, When I submit the login form, Then I am redirected to my dashboard within 2 seconds
- Given invalid credentials, When I submit the login form, Then I see a clear error message without revealing which field was incorrect
- Given I check "Remember me", When I return to the site within 30 days, Then I am automatically logged in

## Business Rules  
- Maximum 5 failed login attempts before account lockout
- Password must meet complexity requirements
- Session expires after 24 hours of inactivity

## Notes
- Consider integration with social login providers
- Audit all login attempts for security monitoring
```

### AI Provider Selection Guidelines

#### For Highest Quality Analysis
- **OpenAI GPT-4**: Best overall quality, comprehensive analysis
- **Claude 3 Opus**: Excellent for complex requirements and detailed explanations
- **Gemini 1.5 Pro**: Strong performance with good context handling

#### For Speed and Efficiency
- **OpenAI GPT-3.5-turbo**: Fast responses, good quality, cost-effective
- **Claude 3 Haiku**: Quick analysis with solid accuracy
- **Gemini 1.5 Flash**: Optimized for speed while maintaining quality

#### For Privacy and Security
- **Ollama (Local)**: Complete data privacy, no external API calls
  - **Recommended Models**: `llama3.1:latest`, `qwen2.5-coder:latest`, `gemma2:latest`
- **Azure OpenAI**: Enterprise-grade security and compliance
- **Self-hosted solutions**: Deploy models in your own infrastructure

#### Cost Considerations
- **Most Cost-Effective**: Ollama (local, free after model download)
- **Balanced Cost/Quality**: OpenAI GPT-3.5-turbo, Gemini 1.5 Flash
- **Premium Quality**: GPT-4, Claude Opus (higher cost, best results)

### Performance Optimization

#### For Large Requirements Documents
```bash
# Process in smaller chunks
split -l 50 large_requirements.txt chunk_

# Analyze chunks individually
for chunk in chunk_*; do
  prism analyze --file "$chunk" --improve --output "${chunk}.improved"
done

# Combine results
cat chunk_*.improved > complete_improved_requirements.txt
```

#### Batch Processing Optimization
```bash
#!/bin/bash
# Optimized batch processing script

# Set efficient configuration
prism config --model "gpt-3.5-turbo" --timeout 20

# Process files in parallel (adjust -P based on API rate limits)
find requirements/ -name "*.md" | xargs -P 3 -I {} bash -c '
  base=$(basename "{}" .md)
  prism analyze --file "{}" --improve --format markdown --output "improved/$base.md"
'
```

#### Configuration for Different Use Cases

```yaml
# Fast Analysis Configuration
llm:
  model: "gpt-3.5-turbo"  # or "gemini-1.5-flash" or "claude-3-haiku"  
  timeout: 15
analysis:
  ambiguity_threshold: 0.8  # Higher threshold = fewer detections, faster analysis

# Comprehensive Analysis Configuration
llm:
  model: "gpt-4"  # or "claude-3-opus" or "gemini-1.5-pro"
  timeout: 45
analysis:
  ambiguity_threshold: 0.6  # Lower threshold = more thorough analysis

# Development/Testing Configuration  
llm:
  provider: "ollama"
  model: "llama3.1:8b"  # Smaller, faster local model
  timeout: 30
analysis:
  ambiguity_threshold: 0.7
```

### Quality Gates and Metrics

#### Establish Quality Thresholds
```bash
#!/bin/bash
# quality_gate_check.sh

analysis_result=$(prism analyze --dir ./requirements --completeness --validate-story --format json)

# Extract metrics
completeness=$(echo "$analysis_result" | jq '.completeness_analysis.completeness_score')  
business_value=$(echo "$analysis_result" | jq '.user_story_validation.business_value_score')
critical_count=$(echo "$analysis_result" | jq '[.ambiguities[] | select(.severity == "Critical")] | length')

# Define thresholds
MIN_COMPLETENESS=75
MIN_BUSINESS_VALUE=60
MAX_CRITICAL=0

# Check gates
if (( $(echo "$completeness < $MIN_COMPLETENESS" | bc -l) )); then
  echo "❌ Completeness below threshold: $completeness% < $MIN_COMPLETENESS%"
  exit 1
fi

if (( $(echo "$business_value < $MIN_BUSINESS_VALUE" | bc -l) )); then
  echo "❌ Business value below threshold: $business_value% < $MIN_BUSINESS_VALUE%"
  exit 1
fi

if [ "$critical_count" -gt "$MAX_CRITICAL" ]; then
  echo "❌ Critical issues found: $critical_count > $MAX_CRITICAL"
  exit 1
fi

echo "✅ All quality gates passed!"
```

#### Continuous Improvement Process

1. **Regular Analysis**: Run weekly analysis on requirement updates
2. **Trend Tracking**: Monitor completeness and quality scores over time
3. **Team Training**: Share common issues and improvement patterns
4. **Template Updates**: Update templates based on frequent issues
5. **Process Refinement**: Adjust quality gates based on team maturity

### Integration with Development Tools

#### VS Code Integration
```json
// .vscode/tasks.json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Analyze Current File",
            "type": "shell",
            "command": "./target/release/prism",
            "args": [
                "analyze",
                "--file",
                "${file}",
                "--format",
                "markdown",
                "--output",
                "${fileDirname}/${fileBasenameNoExtension}_analysis.md"
            ],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared"
            }
        },
        {
            "label": "Generate UML for Current File", 
            "type": "shell",
            "command": "./target/release/prism",
            "args": [
                "analyze", 
                "--file", 
                "${file}",
                "--uml",
                "--save-artifacts",
                "${fileDirname}/${fileBasenameNoExtension}"
            ]
        }
    ]
}
```

#### Git Integration
```bash
# .gitignore additions
*_Analysis.md
*_UML.puml  
*_Logic.py
*.nfr
/docs/generated/
/analysis/temp/

# But keep important analysis results
!docs/final_analysis.md
!design/*.puml
```

---

## 🤝 Contributing and Support

### Getting Help
- **Quick Help**: `prism --help` or `prism <command> --help`
- **Configuration Debug**: `prism config --debug`  
- **Test AI Setup**: `prism config --test`
- **Community Support**: Check project repository for issues and discussions

### Reporting Issues
When reporting issues, please include:
1. **PRISM version**: `prism --version`
2. **Configuration**: `prism config --show` (redact API keys)
3. **Command that failed**: Exact command line used
4. **Error messages**: Complete error output
5. **Operating system**: Version and architecture
6. **File samples**: Minimal example that reproduces the issue (if applicable)

**Example Issue Report:**
```markdown
## Issue Description
PRISM fails to analyze PDF files with error "Failed to extract text"

## Environment
- PRISM Version: 1.0.0
- OS: Ubuntu 20.04
- Provider: OpenAI GPT-4
- File Type: PDF

## Command Used
```bash
prism analyze --file requirements.pdf --format markdown
```

## Error Output
```
❌ Failed to extract text from file: requirements.pdf
Error: PDF processing error: Invalid PDF structure
```

## Expected Behavior
Should extract text from PDF and provide analysis

## Additional Context
- PDF opens normally in viewers
- File size: 2.3MB
- Contains text (not scanned images)
```

### Feature Requests
PRISM is actively developed. Consider requesting:

#### Analysis Features
- Additional ambiguity detection patterns
- Custom rule definitions
- Industry-specific templates
- Multi-language support

#### AI Integration
- New AI providers (Cohere, Hugging Face, etc.)
- Custom model fine-tuning support
- Prompt customization
- Response caching

#### Output and Integration
- Additional output formats (Confluence, Notion)
- Database integrations
- REST API interface
- Web-based interface

#### Advanced Capabilities
- Requirement traceability
- Version control integration
- Requirement linking and dependencies
- Impact analysis

### Development Setup
If you want to contribute to PRISM development:

```bash
# Clone repository
git clone <prism-repo-url>
cd prism

# Build development version
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- analyze "test requirement"

# Format code
cargo fmt

# Run lints
cargo clippy
```

---

*This guide covers PRISM version 1.0.0. For the latest updates and features, check the project repository.*

**🔍 Ready to transform your requirements? Start with `prism config --setup` and begin your AI-powered analysis journey!**

---

## 📚 Additional Resources

### Learning Resources
- **PlantUML Documentation**: http://plantuml.com/guide
- **Requirements Engineering Best Practices**: IEEE 830 Standard
- **User Story Writing Guide**: Mike Cohn's User Stories Applied
- **Agile Requirements**: Dean Leffingwell's Agile Software Requirements

### Tool Integrations
- **VS Code PlantUML Extension**: Render UML diagrams directly in editor
- **Jira Integration**: Import analysis results as tickets
- **Confluence**: Embed generated documentation
- **GitHub Actions**: Automate requirements validation

### API References
- **OpenAI API**: https://platform.openai.com/docs
- **Google Gemini API**: https://ai.google.dev/docs
- **Anthropic Claude API**: https://docs.anthropic.com/
- **Ollama API**: https://github.com/ollama/ollama/blob/main/docs/api.md

**Happy requirement analyzing! 🚀**