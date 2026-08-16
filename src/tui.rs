use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use tokio::sync::mpsc;

use llm_tui::app::{App, Modal};
use llm_tui::domain::Role;
use llm_tui::events::{AppEvent, UserEvent};
use llm_tui::markdown::render_markdown;

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
                        KeyCode::Tab => Some(UserEvent::ToggleFocus),
                        KeyCode::Up => Some(UserEvent::NavigateUp),
                        KeyCode::Down => Some(UserEvent::NavigateDown),
                        KeyCode::Enter => Some(UserEvent::SendMessage),
                        KeyCode::Char('?') => Some(UserEvent::OpenHelp),
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
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(frame.area());

    render_status_bar(frame, app, main_chunks[0]);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(25), Constraint::Min(1)])
        .split(main_chunks[1]);

    render_sidebar(frame, app, content_chunks[0]);
    render_main(frame, app, content_chunks[1]);

    // Render modal overlay if active
    render_modal(frame, app);
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
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("| "),
        Span::styled(
            format!("{} ", app.provider_name),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(format!("| {model}{streaming_info}")),
        Span::styled(error, Style::default().fg(Color::Red)),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn render_sidebar(frame: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let title = if app.sidebar_focus {
        " Conversations [focused] "
    } else {
        " Conversations [Tab] "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(if app.sidebar_focus {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });

    if app.conversations.is_empty() {
        let empty = Paragraph::new("No conversations.\nCtrl+N to start.")
            .style(Style::default().fg(Color::DarkGray))
            .wrap(Wrap { trim: false })
            .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .conversations
        .iter()
        .enumerate()
        .map(|(i, conv)| {
            let is_active = app.active_conversation == Some(conv.id);
            let is_selected = i == app.sidebar_selection;

            let style = if is_selected && app.sidebar_focus {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else if is_active {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let title = if conv.title.len() > 20 {
                format!("{}...", &conv.title[..18])
            } else {
                conv.title.clone()
            };

            ListItem::new(Line::from(Span::styled(title, style)))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_main(frame: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    render_chat(frame, app, chunks[0]);
    render_input(frame, app, chunks[1]);
}

fn render_chat(frame: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let mut lines: Vec<Line> = Vec::new();

    if app.active_conversation.is_none() && app.messages.is_empty() {
        lines.push(Line::from(Span::styled(
            "No conversation selected.\nSelect one from the sidebar or press Ctrl+N.",
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

        // Use markdown rendering for assistant messages
        if msg.role == Role::Assistant {
            let md_lines = render_markdown(&msg.content);
            lines.extend(md_lines);
        } else {
            for text_line in msg.content.lines() {
                lines.push(Line::from(text_line.to_string()));
            }
        }
        lines.push(Line::from(""));
    }

    if let Some(buffer) = app.streaming_content()
        && !buffer.is_empty()
    {
        lines.push(Line::from(Span::styled(
            "── assistant ──",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        // Use markdown rendering for streaming content
        let md_lines = render_markdown(buffer);
        lines.extend(md_lines);
        lines.push(Line::from(Span::styled(
            "▍",
            Style::default().fg(Color::Cyan),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().title(" Chat ").borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_input(frame: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let focus_hint = if app.sidebar_focus {
        "[sidebar focused — Tab to chat]"
    } else {
        "[Tab=sidebar]"
    };

    let model_list: String = app
        .models
        .iter()
        .enumerate()
        .map(|(i, m)| format!("[{}]={}", i + 1, m.id))
        .collect::<Vec<_>>()
        .join(" ");

    let help = format!(
        " Enter=send | Ctrl+N=new | Ctrl+T=test | Ctrl+M=cycle | 1-9=pick | Alt+C=cancel | Ctrl+R=retry | Esc=quit | {} | {} ",
        focus_hint, model_list
    );

    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().title(help).borders(Borders::ALL))
        .style(if app.sidebar_focus {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        });
    frame.render_widget(input, area);
}

fn render_modal(frame: &mut ratatui::Frame, app: &App) {
    match &app.modal {
        Modal::None => {}
        Modal::Rename { buffer } => {
            let area = centered_rect(40, 3, frame.area());
            let block = Block::default()
                .title(" Rename Conversation ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let paragraph = Paragraph::new(buffer.as_str())
                .block(block)
                .style(Style::default().fg(Color::White));
            frame.render_widget(ratatui::widgets::Clear, area);
            frame.render_widget(paragraph, area);
        }
        Modal::DeleteConfirm { title, .. } => {
            let area = centered_rect(50, 5, frame.area());
            let block = Block::default()
                .title(" Delete Conversation ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red));
            let text = vec![
                Line::from(format!("Delete \"{}\"?", title)),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "Enter",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" = confirm  "),
                    Span::styled(
                        "Esc",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" = cancel"),
                ]),
            ];
            let paragraph = Paragraph::new(text)
                .block(block)
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(ratatui::widgets::Clear, area);
            frame.render_widget(paragraph, area);
        }
        Modal::Help => {
            let area = centered_rect(60, 20, frame.area());
            let block = Block::default()
                .title(" Keyboard Shortcuts ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let text = vec![
                Line::from(Span::styled(
                    "Global",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from("  Ctrl+N  New conversation"),
                Line::from("  Ctrl+T  Test connection"),
                Line::from("  Ctrl+M  Cycle model"),
                Line::from("  Ctrl+R  Retry generation"),
                Line::from("  Alt+C   Cancel generation"),
                Line::from("  ?       Show this help"),
                Line::from("  Esc     Quit / Close modal"),
                Line::from(""),
                Line::from(Span::styled(
                    "Sidebar (Tab to focus)",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from("  j/k     Navigate conversations"),
                Line::from("  ↑/↓     Navigate conversations"),
                Line::from("  Enter   Open conversation"),
                Line::from("  r       Rename conversation"),
                Line::from("  d       Delete conversation"),
                Line::from("  1-9     Select model"),
                Line::from(""),
                Line::from(Span::styled(
                    "Chat (Tab to focus)",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from("  Enter   Send message"),
                Line::from("  Esc     Quit"),
            ];
            let paragraph = Paragraph::new(text).block(block);
            frame.render_widget(ratatui::widgets::Clear, area);
            frame.render_widget(paragraph, area);
        }
        Modal::CommandPalette { query, selected: _ } => {
            let area = centered_rect(50, 10, frame.area());
            let block = Block::default()
                .title(" Command Palette ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let text = vec![
                Line::from(format!("> {}", query)),
                Line::from(""),
                Line::from("  (command palette not yet implemented)"),
                Line::from(""),
                Line::from("Press Esc to close"),
            ];
            let paragraph = Paragraph::new(text).block(block);
            frame.render_widget(ratatui::widgets::Clear, area);
            frame.render_widget(paragraph, area);
        }
    }
}

fn centered_rect(percent_x: u16, height: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);

    horizontal_layout[1]
}
