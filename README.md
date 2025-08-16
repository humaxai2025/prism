# 🔍 PRISM - AI-Powered Requirement Analyzer

**The ultimate CLI tool for software requirement validation and improvement.**

PRISM is a modern Rust application that analyzes software requirements to detect ambiguities, extract entities, generate artifacts, and **automatically improve requirements** using AI. From messy user stories to production-ready specs in seconds.

**Current Version:** 1.0.0

## ✨ Key Features

### 🎯 **Complete SDLC Workflow**
- **📋 Analysis**: Detect ambiguities, extract entities, identify issues with severity levels
- **✨ Improvement**: Generate cleaner, more specific requirements using AI
- **🎨 UML Diagrams**: Create Use Case, Sequence, and Class diagrams with PlantUML
- **🔧 Pseudocode**: Generate structured implementation foundations (Python/Java)
- **🧪 Test Cases**: Generate comprehensive test scenarios (happy path, edge cases, negative)
- **📊 Completeness Analysis**: Identify gaps and missing requirements
- **✅ User Story Validation**: Validate format and business value scoring
- **🔒 NFR Generation**: Suggest non-functional requirements by category
- **💾 Individual Artifacts**: Save each output as separate files with proper naming
- **🔄 Integration**: Export to GitHub, Jira, and other project management tools

### 🤖 **AI-Powered Intelligence**
- **5 Provider Support**: OpenAI, Google Gemini, Anthropic Claude, Azure OpenAI, Local Ollama
- **Smart Analysis**: Context-aware ambiguity detection beyond simple patterns
- **Requirement Improvement**: Automatically generates clearer, measurable requirements
- **Privacy Options**: Use local Ollama models for sensitive projects

### 🛠️ **Multiple Interfaces**
- **CLI Mode**: Perfect for automation, CI/CD, and scripting
- **Interactive TUI**: Modern terminal interface with tabbed navigation and real-time analysis
- **Rich Output**: JSON, Markdown, GitHub Issues, Jira tickets, Plain text
- **Document Support**: Process .txt, .md, .rst, .pdf, .docx, .xlsx files
- **Directory Processing**: Batch analyze multiple requirement files

## 🚀 Quick Start

### Installation
```bash
git clone <repository-url>
cd prism
cargo build --release
```

### Basic Usage with Smart Presets
```bash
# Quick requirement improvement
prism improve "As a user, I want to login quickly"

# Smart presets for different analysis levels
prism analyze "As a user, I want to login quickly" --preset basic      # Just analysis + ambiguities
prism analyze "As a user, I want to login quickly" --preset standard   # UML + tests + pseudocode  
prism analyze "As a user, I want to login quickly" --preset full       # All features enabled
prism analyze "As a user, I want to login quickly" --preset report     # Optimized for reports

# Custom generation options (mix and match)
prism analyze --file story.txt --generate uml --generate tests --format markdown

# Specialized commands for focused tasks
prism validate "As a user, I want to login" --story --completeness
prism dashboard --file requirements.txt --output dashboard.html --executive-summary
prism trace --from-commit HEAD~5 --to-commit HEAD

# Batch processing with smart defaults
prism analyze --dir ./requirements --preset report --output analysis.md

# Enhanced configuration management
prism config --setup    # Interactive setup wizard
prism config --test     # Test current configuration

# Launch interactive TUI
prism tui
```

### AI Provider Setup
```bash
# OpenAI
prism config --provider open-ai --api-key "sk-..." --model "gpt-4"

# Anthropic Claude  
prism config --provider claude --api-key "your-key" --model "claude-3-sonnet-20240229"

# Local Ollama (no API key needed - auto-detects available models)
prism config --provider ollama

# Google Gemini
prism config --provider gemini --api-key "your-key" --model "gemini-1.5-pro"

# Azure OpenAI
prism config --provider azure --api-key "your-key" --model "gpt-4"
```

## 📖 Complete Documentation

**For comprehensive usage instructions, examples, and advanced features, please refer to:**

### **[📘 PRISM_USER_GUIDE.md](./PRISM_USER_GUIDE.md)**

The user guide covers:
- **Complete Command Reference** - All CLI commands and options
- **Output Formats** - JSON, Markdown, GitHub, Jira, Plain text examples
- **AI Provider Setup** - Detailed setup for all 5 AI providers
- **Advanced Features** - Multi-type UML generation, structured pseudocode, test cases
- **Artifact Management** - Individual file saving with proper naming conventions
- **Integration Examples** - CI/CD, pre-commit hooks, automation
- **Troubleshooting Guide** - Common issues and solutions
- **Configuration Details** - Environment variables, config files

## 🏢 Enterprise Features (NEW!)

### **Smart Presets for Every Use Case**
```bash
# Quick analysis for development
prism analyze --file requirements.txt --preset basic

# Standard workflow with essential artifacts  
prism analyze --dir ./stories --preset standard --format markdown

# Complete enterprise analysis
prism analyze --file requirements.txt --preset full --output complete-analysis.md

# Report-optimized for documentation
prism analyze --dir ./requirements --preset report --output project-report.md
```

### **Specialized Commands for Different Teams**
```bash
# Quality Assurance - Validation focused
prism validate --dir ./user-stories --all --output validation-report.md

# Architecture Team - Requirements traceability 
prism trace --file requirements.txt --source-dir ./src --test-dir ./tests

# Management - Executive dashboards
prism dashboard --dir ./requirements --output executive-dashboard.html --executive-summary
```

### **Advanced Configuration Management**
```bash
# Interactive setup wizard (recommended)
prism config --setup

# Test current AI configuration
prism config --test

# Validate all configuration settings
prism config --validate-all

# Test all AI providers simultaneously  
prism config --test-providers
```

### **Batch Processing with Smart Defaults**
```bash
# Robust batch processing (auto-enables progress, error handling)
prism analyze --dir ./large-project --preset report --parallel 4

# Custom artifact generation  
prism analyze --file story.txt --generate all --save-artifacts "project"
```

## 💡 Quick Examples

### **Requirement Improvement & Analysis**
```bash
# Transform vague requirements into precise specifications
prism improve "As a user, I want to login quickly"
```

**Sample Output:**
```markdown
# 🔍 PRISM Requirements Improvement Report

## 📝 Improved Requirements

The system shall authenticate users within 2 seconds under normal 
network conditions. The 95th percentile login time shall not exceed 
3 seconds. Authentication shall utilize multi-factor authentication 
(MFA) with a minimum password complexity of 12 characters...

## 📊 Issues Fixed
**Total Issues Addressed:** 3
- "quickly" → Defined specific performance metrics (High severity)
- Missing auth details → Added security specifications (Medium severity)
- Vague success criteria → Measurable requirements (High severity)
```

### **Complete Analysis with Smart Presets**
```bash
# Generate all artifacts as separate files using preset
prism analyze --file user_stories.txt --preset full --save-artifacts "project"

# Creates up to 5 files:
# - project_Analysis.md (comprehensive analysis report)
# - project_Req.md (improved requirements only)
# - project_UML.puml (PlantUML diagrams - Use Case, Sequence, Class)
# - project_Logic.py (structured pseudocode with business logic)
# - project_NFR.md (non-functional requirements by category)
```

### **Specialized Commands for Different Needs**
```bash
# User story validation and business value scoring
prism validate --file user_story.txt --story --completeness --format markdown

# Generate non-functional requirements with custom generation
prism analyze "As a user, I want to upload files" --generate nfr --format markdown

# Custom mix of features
prism analyze --file requirements.txt --generate improve --generate tests --format github
```

### **Local AI Privacy**
```bash
# Use local Ollama for sensitive projects  
prism config --provider ollama
prism improve --file confidential_requirements.txt
```

### **CI/CD Integration**
```bash
# Automated requirement validation in pipelines
prism validate --dir ./requirements --all --format plain --output validation.log
```

## 🏗️ Development Workflow Integration

PRISM integrates seamlessly into your development workflow:

```yaml
# GitHub Actions - Requirement Quality Gate
- name: Validate Requirements
  run: |
    prism validate --dir ./requirements --all --format github --output PR_analysis.md
    prism improve --file user_story.md --output improved_story.md
```

```bash
# Pre-commit Hook - Prevent Bad Requirements
prism validate --dir ./docs/requirements --all --format plain
```

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines and feel free to:

1. 🐛 Report bugs and issues
2. 💡 Suggest new features  
3. 🔧 Submit pull requests
4. 📖 Improve documentation
5. 🧪 Add test cases

## 📄 License

MIT License - see the LICENSE file for details.

---

## 🆘 Need Help?

- **Quick Help**: `prism --help` or `prism <command> --help`
- **Complete Guide**: [PRISM_USER_GUIDE.md](./PRISM_USER_GUIDE.md)
- **Issues**: Report bugs on GitHub Issues
- **Configuration**: `prism config --debug` for troubleshooting

## 🎯 What Makes PRISM Special?

PRISM is the **complete SDLC requirements tool** that doesn't just analyze—it **transforms your entire requirements workflow**:

### 🚀 **End-to-End Requirement Management**
- **Analysis**: Detects ambiguities with severity levels (Critical/High/Medium/Low)
- **Improvement**: AI-powered requirement enhancement with specific suggestions
- **Validation**: User story format validation and business value scoring
- **Completeness**: Gap analysis and missing requirement identification
- **Design**: Multi-type UML diagram generation with PlantUML
- **Implementation**: Structured pseudocode with business logic and error handling
- **Testing**: Comprehensive test case generation (happy path, edge cases, negative)
- **Quality**: Non-functional requirement suggestions across 8 categories
- **Documentation**: Individual artifact files with proper naming conventions

### 🎨 **Professional UML Generation**
- **Use Case Diagrams**: Smart actor-action relationships with system boundaries
- **Sequence Diagrams**: Complete interaction flows with error handling and alternative paths
- **Class Diagrams**: Entity classes, service classes, enums, and relationships
- **PlantUML Output**: Ready-to-render diagrams with professional styling and themes

### 🔧 **Real Implementation Foundations**
- **Python/Java Pseudocode**: Complete class structures with business logic
- **Authentication & Validation**: Built-in security patterns and input validation
- **Error Handling**: Comprehensive exception management and logging
- **Business Logic**: Service layers with permission checks and audit trails
- **Data Models**: Entity classes with validation and serialization methods

### 📊 **Quality Assurance Features**
- **Completeness Analysis**: Identifies missing actors, success criteria, and NFRs
- **Business Value Scoring**: Quantifies user story value and quality
- **NFR Generation**: Performance, Security, Usability, Reliability, and more
- **Test Case Coverage**: Happy path, negative scenarios, and edge cases
- **Gap Identification**: Structured recommendations with priority levels

**Transform this:** *"As a user, I want to login quickly"*  
**Into this complete workflow:**
- ✅ **Improved Requirements**: Specific 2-second login requirement with MFA
- 📊 **Completeness Analysis**: 85% score with identified gaps
- ✅ **User Story Validation**: Format validation and business value scoring
- 🎨 **3 UML Diagrams**: Use case, sequence, and class diagrams
- 🔧 **Implementation Code**: Complete authentication classes and business logic
- 🧪 **Test Cases**: 12 scenarios covering happy path, edge cases, and errors
- 🔒 **NFR Suggestions**: Security, performance, and usability requirements
- 📄 **5 Separate Files**: Analysis, Requirements, UML, Logic, and NFR files

**Ready to revolutionize your SDLC?** 🚀

---

*Built with ❤️ in Rust | Powered by AI | Open Source*
