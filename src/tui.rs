use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use tokio::sync::mpsc;

use llm_tui::app::App;
use llm_tui::domain::Role;
use llm_tui::events::{AppEvent, UserEvent};

pub async fn run(app: &mut App) -> Result<()> {
    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let (event_tx, mut event_rx) = mpsc::channel::<AppEvent>(256);

    app.set_event_tx(event_tx.clone()).await;
    app.init().await;

    let input_tx = event_tx.clone();
    tokio::spawn(async move {
        loop {
            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
                    let user_event = match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            Some(UserEvent::Quit)
                        }
                        KeyCode::Esc => Some(UserEvent::Quit),
                        KeyCode::Enter => Some(UserEvent::SendMessage),
                        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            Some(UserEvent::NewConversation)
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::ALT) => {
                            Some(UserEvent::CancelGeneration)
                        }
                        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            Some(UserEvent::Retry)
                        }
                        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            Some(UserEvent::TestConnection)
                        }
                        KeyCode::Char('m') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            Some(UserEvent::SelectModel(usize::MAX))
                        }
                        KeyCode::Char(c @ '1'..='9') => {
                            let idx = (c as usize) - ('1' as usize);
                            Some(UserEvent::SelectModel(idx))
                        }
                        KeyCode::Char(c) => Some(UserEvent::InputChar(c)),
                        KeyCode::Backspace => Some(UserEvent::Backspace),
                        _ => None,
                    };

                    if let Some(ue) = user_event {
                        let _ = input_tx.send(AppEvent::User(ue)).await;
                    }
                }
            } else {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    });

    let result = loop {
        terminal.draw(|frame| render(frame, app))?;

        tokio::select! {
            Some(event) = event_rx.recv() => {
                app.handle_event(event).await;
                if app.should_quit {
                    break Ok(());
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {}
        }
    };

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn render(frame: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_status_bar(frame, app, chunks[0]);
    render_chat(frame, app, chunks[1]);
    render_input(frame, app, chunks[2]);
}

fn render_status_bar(frame: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let model = app.selected_model.as_deref().unwrap_or("—");

    let streaming_info = if let Some(stats) = app.streaming_stats() {
        let (tokens, elapsed) = stats;
        let tps = if elapsed > 0.0 {
            format!("{:.1} tok/s", tokens as f64 / elapsed)
        } else {
            "streaming...".into()
        };
        format!(" | {tps}")
    } else {
        String::new()
    };

    let error = app
        .error
        .as_ref()
        .map(|e| {
            let msg = if e.len() > 40 { &e[..40] } else { e };
            format!(" | ERR: {msg}")
        })
        .unwrap_or_default();

    let line = Line::from(vec![
        Span::styled(
            " llm-tui ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw("| "),
        Span::styled(
            format!("{} ", &app.provider_name),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(format!("| {model}{streaming_info}")),
        Span::styled(error, Style::default().fg(Color::Red)),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn render_chat(frame: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let mut lines: Vec<Line> = Vec::new();

    if app.active_conversation.is_none() && app.messages.is_empty() {
        lines.push(Line::from(Span::styled(
            "No conversation. Press Ctrl+N to start one.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    for msg in &app.messages {
        let (label, color) = match msg.role {
            Role::System => ("system", Color::Yellow),
            Role::User => ("you", Color::Green),
            Role::Assistant => ("assistant", Color::Cyan),
            Role::Tool => ("tool", Color::Magenta),
        };

        lines.push(Line::from(Span::styled(
            format!("── {label} ──"),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )));

        for text_line in msg.content.lines() {
            lines.push(Line::from(text_line.to_string()));
        }
        lines.push(Line::from(""));
    }

    if let Some(buffer) = app.streaming_content() {
        if !buffer.is_empty() {
            lines.push(Line::from(Span::styled(
                "── assistant ──",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            for text_line in buffer.lines() {
                lines.push(Line::from(text_line.to_string()));
            }
            lines.push(Line::from(Span::styled(
                "▍",
                Style::default().fg(Color::Cyan),
            )));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().title(" Chat ").borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_input(frame: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let model_list: String = app
        .models
        .iter()
        .enumerate()
        .map(|(i, m)| format!("[{}]={}", i + 1, m.id))
        .collect::<Vec<_>>()
        .join(" ");

    let help = format!(
        " Enter=send | Ctrl+N=new | Ctrl+T=test | Ctrl+M=cycle model | 1-9=pick | Alt+C=cancel | Ctrl+R=retry | Esc=quit | {} ",
        model_list
    );

    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().title(help).borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    frame.render_widget(input, area);
}
