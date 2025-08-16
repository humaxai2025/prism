use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, time::Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Tabs, Wrap,
    },
    Frame, Terminal,
};

use crate::analyzer::{Analyzer, AnalysisResult, AmbiguitySeverity};
use crate::config::Config;

#[derive(Clone)]
pub struct TuiApp {
    analyzer: Analyzer,
    config: Config,
    state: AppState,
}

#[derive(Clone)]
struct AppState {
    input_text: String,
    current_tab: usize,
    analysis_result: Option<AnalysisResult>,
    is_analyzing: bool,
    selected_ambiguity: usize,
    show_help: bool,
    cursor_position: usize,
    input_mode: InputMode,
    clarification_questions: Vec<ClarificationQuestion>,
    current_question: usize,
}

#[derive(Clone)]
enum InputMode {
    Normal,
    Editing,
    Clarification,
}

#[derive(Clone)]
struct ClarificationQuestion {
    question: String,
    context: String,
    answer: Option<String>,
}

impl TuiApp {
    pub fn new(analyzer: Analyzer, config: Config) -> Result<Self> {
        Ok(Self {
            analyzer,
            config,
            state: AppState {
                input_text: String::new(),
                current_tab: 0,
                analysis_result: None,
                is_analyzing: false,
                selected_ambiguity: 0,
                show_help: false,
                cursor_position: 0,
                input_mode: InputMode::Normal,
                clarification_questions: Vec::new(),
                current_question: 0,
            },
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal).await;

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match self.state.input_mode {
                        InputMode::Normal => {
                            if self.handle_normal_input(key).await? {
                                break;
                            }
                        }
                        InputMode::Editing => {
                            if self.handle_editing_input(key).await? {
                                break;
                            }
                        }
                        InputMode::Clarification => {
                            if self.handle_clarification_input(key).await? {
                                break;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_normal_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('h') => self.state.show_help = !self.state.show_help,
            KeyCode::Char('i') => self.state.input_mode = InputMode::Editing,
            KeyCode::Char('a') => {
                if !self.state.input_text.is_empty() && !self.state.is_analyzing {
                    self.analyze_input().await?;
                }
            }
            KeyCode::Char('c') => {
                if self.state.analysis_result.is_some() && !self.state.clarification_questions.is_empty() {
                    self.state.input_mode = InputMode::Clarification;
                }
            }
            KeyCode::Tab => {
                self.state.current_tab = (self.state.current_tab + 1) % 4;
            }
            KeyCode::Up => {
                if self.state.selected_ambiguity > 0 {
                    self.state.selected_ambiguity -= 1;
                }
            }
            KeyCode::Down => {
                if let Some(result) = &self.state.analysis_result {
                    if self.state.selected_ambiguity < result.ambiguities.len().saturating_sub(1) {
                        self.state.selected_ambiguity += 1;
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_editing_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => self.state.input_mode = InputMode::Normal,
            KeyCode::Char(c) => {
                self.state.input_text.insert(self.state.cursor_position, c);
                self.state.cursor_position += 1;
            }
            KeyCode::Backspace => {
                if self.state.cursor_position > 0 {
                    self.state.cursor_position -= 1;
                    self.state.input_text.remove(self.state.cursor_position);
                }
            }
            KeyCode::Left => {
                if self.state.cursor_position > 0 {
                    self.state.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.state.cursor_position < self.state.input_text.len() {
                    self.state.cursor_position += 1;
                }
            }
            KeyCode::Enter => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.state.input_mode = InputMode::Normal;
                    if !self.state.input_text.is_empty() {
                        self.analyze_input().await?;
                    }
                } else {
                    self.state.input_text.insert(self.state.cursor_position, '\n');
                    self.state.cursor_position += 1;
                }
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_clarification_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => self.state.input_mode = InputMode::Normal,
            KeyCode::Enter => {
                self.state.current_question = (self.state.current_question + 1) % self.state.clarification_questions.len();
            }
            _ => {}
        }
        Ok(false)
    }

    async fn analyze_input(&mut self) -> Result<()> {
        self.state.is_analyzing = true;
        
        match self.analyzer.analyze(&self.state.input_text).await {
            Ok(mut result) => {
                self.generate_clarification_questions(&result);
                
                let use_case = self.analyzer.generate_uml_use_case(&result.entities);
                result.uml_diagrams = Some(crate::analyzer::UmlDiagrams {
                    use_case: Some(use_case),
                    sequence: None,
                    class_diagram: None,
                });
                
                let pseudocode = self.analyzer.generate_pseudocode(&result.entities, None);
                result.pseudocode = Some(pseudocode);
                
                let test_cases = self.analyzer.generate_test_cases(&result.entities);
                result.test_cases = Some(test_cases);
                
                self.state.analysis_result = Some(result);
            }
            Err(e) => {
                eprintln!("Analysis failed: {}", e);
            }
        }
        
        self.state.is_analyzing = false;
        Ok(())
    }

    fn generate_clarification_questions(&mut self, result: &AnalysisResult) {
        self.state.clarification_questions.clear();
        
        for ambiguity in &result.ambiguities {
            let question = match ambiguity.text.as_str() {
                text if text.contains("fast") || text.contains("quick") => {
                    ClarificationQuestion {
                        question: format!("You mentioned '{}'. Please specify the exact performance requirement (e.g., response time in milliseconds).", text),
                        context: ambiguity.reason.clone(),
                        answer: None,
                    }
                }
                text if text.contains("user-friendly") || text.contains("easy") => {
                    ClarificationQuestion {
                        question: format!("You mentioned '{}'. What specific usability criteria define this? (e.g., number of clicks, learning time)", text),
                        context: ambiguity.reason.clone(),
                        answer: None,
                    }
                }
                _ => {
                    ClarificationQuestion {
                        question: format!("Please clarify: {}", ambiguity.text),
                        context: ambiguity.reason.clone(),
                        answer: None,
                    }
                }
            };
            self.state.clarification_questions.push(question);
        }
    }

    fn ui<B: Backend>(&self, f: &mut Frame<B>) {
        if self.state.show_help {
            self.render_help_popup(f);
            return;
        }

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(f.size());

        self.render_header(f, main_layout[0]);
        self.render_main_content(f, main_layout[1]);
        self.render_footer(f, main_layout[2]);
    }

    fn render_header<B: Backend>(&self, f: &mut Frame<B>, area: tui::layout::Rect) {
        let title = "🔍 PRISM - AI-Powered Requirement Analyzer";
        let header = Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, area);
    }

    fn render_main_content<B: Backend>(&self, f: &mut Frame<B>, area: tui::layout::Rect) {
        let tabs = ["📝 Input", "⚠️  Ambiguities", "🎯 Entities", "📊 Output"]
            .iter()
            .cloned()
            .map(Spans::from)
            .collect();

        let tabs_widget = Tabs::new(tabs)
            .block(Block::default().borders(Borders::ALL).title("Analysis Tabs"))
            .select(self.state.current_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        f.render_widget(tabs_widget, content_layout[0]);

        match self.state.current_tab {
            0 => self.render_input_tab(f, content_layout[1]),
            1 => self.render_ambiguities_tab(f, content_layout[1]),
            2 => self.render_entities_tab(f, content_layout[1]),
            3 => self.render_output_tab(f, content_layout[1]),
            _ => {}
        }
    }

    fn render_input_tab<B: Backend>(&self, f: &mut Frame<B>, area: tui::layout::Rect) {
        let input_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(area);

        let input_style = match self.state.input_mode {
            InputMode::Editing => Style::default().fg(Color::Green),
            _ => Style::default().fg(Color::White),
        };

        let input_widget = Paragraph::new(self.state.input_text.as_ref())
            .style(input_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Requirement Text (Press 'i' to edit, Ctrl+Enter to analyze)")
            )
            .wrap(Wrap { trim: true });

        f.render_widget(input_widget, input_layout[0]);

        if self.state.is_analyzing {
            let progress = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("Status"))
                .gauge_style(Style::default().fg(Color::Yellow))
                .label("Analyzing...")
                .ratio(0.5);
            f.render_widget(progress, input_layout[1]);
        } else {
            let status_text = if self.state.analysis_result.is_some() {
                "✅ Analysis Complete"
            } else {
                "⏳ Ready to Analyze"
            };

            let status_widget = Paragraph::new(status_text)
                .style(Style::default().fg(Color::Green))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(status_widget, input_layout[1]);
        }
    }

    fn render_ambiguities_tab<B: Backend>(&self, f: &mut Frame<B>, area: tui::layout::Rect) {
        if let Some(result) = &self.state.analysis_result {
            if result.ambiguities.is_empty() {
                let no_ambiguities = Paragraph::new("✅ No ambiguities detected! Your requirements are clear.")
                    .style(Style::default().fg(Color::Green))
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL).title("Ambiguities"));
                f.render_widget(no_ambiguities, area);
                return;
            }

            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
                .split(area);

            let items: Vec<ListItem> = result
                .ambiguities
                .iter()
                .enumerate()
                .map(|(_i, ambiguity)| {
                    let severity_icon = match ambiguity.severity {
                        AmbiguitySeverity::Critical => "🔴",
                        AmbiguitySeverity::High => "🟠",
                        AmbiguitySeverity::Medium => "🟡",
                        AmbiguitySeverity::Low => "🟢",
                    };
                    
                    let content = vec![Spans::from(vec![
                        Span::raw(severity_icon),
                        Span::raw(" "),
                        Span::styled(
                            &ambiguity.text,
                            Style::default().add_modifier(Modifier::BOLD)
                        ),
                    ])];
                    ListItem::new(content)
                })
                .collect();

            let mut list_state = ListState::default();
            list_state.select(Some(self.state.selected_ambiguity));

            let ambiguities_list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Detected Issues"))
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol("▶ ");

            f.render_stateful_widget(ambiguities_list, layout[0], &mut list_state);

            if let Some(selected_ambiguity) = result.ambiguities.get(self.state.selected_ambiguity) {
                let detail_text = vec![
                    Spans::from(vec![Span::styled(
                        "Reason:",
                        Style::default().add_modifier(Modifier::BOLD)
                    )]),
                    Spans::from(vec![Span::raw(&selected_ambiguity.reason)]),
                    Spans::from(vec![Span::raw("")]),
                    Spans::from(vec![Span::styled(
                        "Suggestions:",
                        Style::default().add_modifier(Modifier::BOLD)
                    )]),
                ];

                let mut full_text = detail_text;
                for suggestion in &selected_ambiguity.suggestions {
                    full_text.push(Spans::from(vec![
                        Span::raw("• "),
                        Span::raw(suggestion)
                    ]));
                }

                let details = Paragraph::new(full_text)
                    .block(Block::default().borders(Borders::ALL).title("Details"))
                    .wrap(Wrap { trim: true });

                f.render_widget(details, layout[1]);
            }
        } else {
            let no_analysis = Paragraph::new("No analysis performed yet. Go to Input tab and analyze some requirements!")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Ambiguities"));
            f.render_widget(no_analysis, area);
        }
    }

    fn render_entities_tab<B: Backend>(&self, f: &mut Frame<B>, area: tui::layout::Rect) {
        if let Some(result) = &self.state.analysis_result {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(34)].as_ref())
                .split(area);

            let actors_text = if result.entities.actors.is_empty() {
                "No actors identified".to_string()
            } else {
                result.entities.actors.join(", ")
            };

            let actions_text = if result.entities.actions.is_empty() {
                "No actions identified".to_string()
            } else {
                result.entities.actions.join(", ")
            };

            let objects_text = if result.entities.objects.is_empty() {
                "No objects identified".to_string()
            } else {
                result.entities.objects.join(", ")
            };

            let actors_widget = Paragraph::new(actors_text)
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL).title("👥 Actors"))
                .wrap(Wrap { trim: true });

            let actions_widget = Paragraph::new(actions_text)
                .style(Style::default().fg(Color::Green))
                .block(Block::default().borders(Borders::ALL).title("⚡ Actions"))
                .wrap(Wrap { trim: true });

            let objects_widget = Paragraph::new(objects_text)
                .style(Style::default().fg(Color::Magenta))
                .block(Block::default().borders(Borders::ALL).title("📦 Objects"))
                .wrap(Wrap { trim: true });

            f.render_widget(actors_widget, layout[0]);
            f.render_widget(actions_widget, layout[1]);
            f.render_widget(objects_widget, layout[2]);
        } else {
            let no_analysis = Paragraph::new("No analysis performed yet. Go to Input tab and analyze some requirements!")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Entities"));
            f.render_widget(no_analysis, area);
        }
    }

    fn render_output_tab<B: Backend>(&self, f: &mut Frame<B>, area: tui::layout::Rect) {
        if let Some(result) = &self.state.analysis_result {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(area);

            let uml_text = if let Some(uml) = &result.uml_diagrams {
                if let Some(use_case) = &uml.use_case {
                    use_case.clone()
                } else {
                    "No UML diagram generated".to_string()
                }
            } else {
                "No UML diagram generated".to_string()
            };

            let pseudocode_text = result.pseudocode.clone()
                .unwrap_or_else(|| "No pseudocode generated".to_string());

            let uml_widget = Paragraph::new(uml_text)
                .style(Style::default().fg(Color::Blue))
                .block(Block::default().borders(Borders::ALL).title("🔄 UML Use Case Diagram"))
                .wrap(Wrap { trim: true });

            let code_widget = Paragraph::new(pseudocode_text)
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("💻 Generated Pseudocode"))
                .wrap(Wrap { trim: true });

            f.render_widget(uml_widget, layout[0]);
            f.render_widget(code_widget, layout[1]);
        } else {
            let no_analysis = Paragraph::new("No analysis performed yet. Go to Input tab and analyze some requirements!")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Output"));
            f.render_widget(no_analysis, area);
        }
    }

    fn render_footer<B: Backend>(&self, f: &mut Frame<B>, area: tui::layout::Rect) {
        let help_text = match self.state.input_mode {
            InputMode::Normal => "q: Quit | h: Help | i: Edit | a: Analyze | Tab: Switch tabs | ↑/↓: Navigate",
            InputMode::Editing => "Esc: Normal mode | Ctrl+Enter: Analyze | Type to edit text",
            InputMode::Clarification => "Esc: Normal mode | Enter: Next question",
        };

        let footer = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, area);
    }

    fn render_help_popup<B: Backend>(&self, f: &mut Frame<B>) {
        let popup_area = self.centered_rect(80, 60, f.size());

        f.render_widget(Clear, popup_area);

        let help_text = vec![
            Spans::from(vec![Span::styled(
                "PRISM - AI-Powered Requirement Analyzer",
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
            )]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::styled(
                "Navigation:",
                Style::default().add_modifier(Modifier::BOLD)
            )]),
            Spans::from(vec![Span::raw("q - Quit application")]),
            Spans::from(vec![Span::raw("h - Toggle this help")]),
            Spans::from(vec![Span::raw("Tab - Switch between tabs")]),
            Spans::from(vec![Span::raw("↑/↓ - Navigate lists")]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::styled(
                "Input Mode:",
                Style::default().add_modifier(Modifier::BOLD)
            )]),
            Spans::from(vec![Span::raw("i - Enter edit mode")]),
            Spans::from(vec![Span::raw("Esc - Exit edit mode")]),
            Spans::from(vec![Span::raw("Ctrl+Enter - Analyze requirements")]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::styled(
                "Analysis:",
                Style::default().add_modifier(Modifier::BOLD)
            )]),
            Spans::from(vec![Span::raw("a - Analyze current input")]),
            Spans::from(vec![Span::raw("c - Clarification mode (if available)")]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::styled(
                "Tabs:",
                Style::default().add_modifier(Modifier::BOLD)
            )]),
            Spans::from(vec![Span::raw("📝 Input - Enter and edit requirements")]),
            Spans::from(vec![Span::raw("⚠️  Ambiguities - Review detected issues")]),
            Spans::from(vec![Span::raw("🎯 Entities - View extracted components")]),
            Spans::from(vec![Span::raw("📊 Output - See UML and pseudocode")]),
        ];

        let help_widget = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help (Press 'h' to close)")
            )
            .wrap(Wrap { trim: true });

        f.render_widget(help_widget, popup_area);
    }

    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: tui::layout::Rect) -> tui::layout::Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}