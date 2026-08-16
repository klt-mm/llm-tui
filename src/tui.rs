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

use llm_tui::app::{ActiveScreen, App, Modal};
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
                        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            Some(UserEvent::OpenPromptPicker)
                        }
                        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            Some(UserEvent::OpenPromptList)
                        }
                        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            Some(UserEvent::OpenSearch)
                        }
                        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            Some(UserEvent::OpenCommandPalette)
                        }
                        KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            Some(UserEvent::OpenGenerationSettings)
                        }
                        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            Some(UserEvent::OpenBranchHistory)
                        }
                        KeyCode::Char('/') => Some(UserEvent::InputChar('/')),
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

    match app.active_screen {
        ActiveScreen::Chat => {
            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(25), Constraint::Min(1)])
                .split(main_chunks[1]);

            render_sidebar(frame, app, content_chunks[0]);
            render_main(frame, app, content_chunks[1]);
        }
        ActiveScreen::Search => {
            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(25), Constraint::Min(1)])
                .split(main_chunks[1]);

            render_sidebar(frame, app, content_chunks[0]);
            render_search_screen(frame, app, content_chunks[1]);
        }
        ActiveScreen::Prompts => {
            render_prompts_screen(frame, app, main_chunks[1]);
        }
    }

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
    } else if let Some(ref metrics) = app.last_generation_metrics {
        format!(
            " | {:.1} tok/s | {} tok | {:.0}ms",
            metrics.tokens_per_second, metrics.total_tokens, metrics.duration_ms
        )
    } else {
        String::new()
    };

    let (ctx_used, ctx_budget) = app.context_info();
    let ctx_info = if ctx_used > 0 {
        match ctx_budget {
            Some(budget) => {
                let used_k = ctx_used as f64 / 1000.0;
                let budget_k = budget as f64 / 1000.0;
                format!(" | {:.1}k/{:.0}k tok", used_k, budget_k)
            }
            None => {
                let used_k = ctx_used as f64 / 1000.0;
                format!(" | {:.1}k tok", used_k)
            }
        }
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
        Span::raw(format!("| {model}{streaming_info}{ctx_info}")),
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

        // Show branch indicator if this message has a parent
        if let Some(parent_id) = msg.parent_id
            && let Some(parent) = app.messages.iter().find(|m| m.id == parent_id)
        {
            let parent_snippet = if parent.content.len() > 30 {
                format!("{}...", &parent.content[..27])
            } else {
                parent.content.clone()
            };
            let parent_label = match parent.role {
                Role::User => "you",
                Role::Assistant => "assistant",
                Role::System => "system",
                Role::Tool => "tool",
            };
            lines.push(Line::from(Span::styled(
                format!("  ↩ reply to {}: {}", parent_label, parent_snippet),
                Style::default().fg(Color::DarkGray),
            )));
        }

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
                Line::from("  Ctrl+K  Command palette"),
                Line::from("  Ctrl+P  Prompt picker"),
                Line::from("  Ctrl+L  Prompt list"),
                Line::from("  Ctrl+F  Search"),
                Line::from("  Ctrl+G  Generation settings"),
                Line::from("  Ctrl+B  Branch history"),
                Line::from("  /       Search (when input empty)"),
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
                Line::from("  n       Prompts screen"),
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
        Modal::CommandPalette {
            query,
            selected,
            filtered,
        } => {
            let height = (filtered.len() + 4).min(15) as u16;
            let area = centered_rect(50, height, frame.area());
            let block = Block::default()
                .title(" Command Palette (Ctrl+K) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(area);
            frame.render_widget(ratatui::widgets::Clear, area);
            frame.render_widget(block, area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(1)])
                .split(inner);

            let search =
                Paragraph::new(format!("> {}", query)).style(Style::default().fg(Color::White));
            frame.render_widget(search, chunks[0]);

            if filtered.is_empty() {
                let empty = Paragraph::new("  No matching commands")
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(empty, chunks[1]);
            } else {
                let items: Vec<ListItem> = filtered
                    .iter()
                    .enumerate()
                    .map(|(i, cmd)| {
                        let is_selected = i == *selected;
                        let style = if is_selected {
                            Style::default().fg(Color::Black).bg(Color::Cyan)
                        } else {
                            Style::default()
                        };
                        ListItem::new(Line::from(Span::styled(
                            format!("  {}", cmd.label()),
                            style,
                        )))
                    })
                    .collect();
                let list = List::new(items);
                frame.render_widget(list, chunks[1]);
            }
        }
        Modal::PromptPicker {
            query,
            selected,
            filtered,
        } => {
            let height = (filtered.len() + 4).min(15) as u16;
            let area = centered_rect(60, height, frame.area());
            let block = Block::default()
                .title(" Prompt Picker (Ctrl+P) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(area);
            frame.render_widget(ratatui::widgets::Clear, area);
            frame.render_widget(block, area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(1),
                ])
                .split(inner);

            let search =
                Paragraph::new(format!("/{}", query)).style(Style::default().fg(Color::White));
            frame.render_widget(search, chunks[0]);

            let hint = Line::from(vec![
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("=use  "),
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::raw("=nav  "),
                Span::styled("n", Style::default().fg(Color::Yellow)),
                Span::raw("=new  "),
                Span::styled("e", Style::default().fg(Color::Yellow)),
                Span::raw("=edit  "),
                Span::styled("d", Style::default().fg(Color::Red)),
                Span::raw("=del  "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw("=close"),
            ]);
            frame.render_widget(Paragraph::new(hint), chunks[1]);

            if filtered.is_empty() {
                let empty = Paragraph::new("  No prompts found. Press n to create one.")
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(empty, chunks[2]);
            } else {
                let items: Vec<ListItem> = filtered
                    .iter()
                    .enumerate()
                    .map(|(i, &real_idx)| {
                        let prompt = &app.prompts[real_idx];
                        let is_selected = i == *selected;
                        let tags = if prompt.tags.is_empty() {
                            String::new()
                        } else {
                            format!(" [{}]", prompt.tags.join(", "))
                        };
                        let style = if is_selected {
                            Style::default().fg(Color::Black).bg(Color::Cyan)
                        } else {
                            Style::default()
                        };
                        let display = if prompt.name.len() + tags.len() > 50 {
                            format!("{}{}", &prompt.name[..40], tags)
                        } else {
                            format!("{}{}", prompt.name, tags)
                        };
                        ListItem::new(Line::from(Span::styled(format!("  {}", display), style)))
                    })
                    .collect();
                let list = List::new(items);
                frame.render_widget(list, chunks[2]);
            }
        }
        Modal::PromptEditor {
            editing_id,
            name,
            description,
            content,
            system_prompt,
            tags,
            variables,
            field,
        } => {
            let height = 18;
            let area = centered_rect(70, height, frame.area());
            let title = if editing_id.is_some() {
                " Edit Prompt "
            } else {
                " New Prompt "
            };
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(area);
            frame.render_widget(ratatui::widgets::Clear, area);
            frame.render_widget(block, area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(1),
                ])
                .split(inner);

            let fields: [(&str, &String); 6] = [
                ("Name", name),
                ("Description", description),
                ("Content", content),
                ("System Prompt", system_prompt),
                ("Tags (comma-sep)", tags),
                ("Variables (comma-sep)", variables),
            ];

            for (i, (label, value)) in fields.iter().enumerate() {
                let style = if i == *field {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };
                let cursor = if i == *field { "▍" } else { "" };
                let line = Line::from(vec![
                    Span::styled(format!("  {}: ", label), style),
                    Span::styled((*value).clone(), Style::default().fg(Color::White)),
                    Span::styled(cursor, Style::default().fg(Color::Cyan)),
                ]);
                frame.render_widget(Paragraph::new(line), chunks[i]);
            }

            let hint = Line::from(vec![
                Span::styled(
                    "Tab/↑↓",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("=next field  "),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("=save  "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw("=cancel"),
            ]);
            frame.render_widget(Paragraph::new(hint), chunks[7]);
        }
        Modal::VariableInput {
            variables,
            values,
            current,
            ..
        } => {
            let height = (variables.len() + 5) as u16;
            let area = centered_rect(50, height, frame.area());
            let block = Block::default()
                .title(" Prompt Variables ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(area);
            frame.render_widget(ratatui::widgets::Clear, area);
            frame.render_widget(block, area);

            let mut lines = Vec::new();
            for (i, var) in variables.iter().enumerate() {
                let val = values.get(i).map(|s| s.as_str()).unwrap_or("");
                let style = if i == *current {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };
                let cursor = if i == *current { "▍" } else { "" };
                lines.push(Line::from(vec![
                    Span::styled(format!("  {{{{ {} }}}} = ", var), style),
                    Span::styled(val.to_string(), Style::default().fg(Color::White)),
                    Span::styled(cursor, Style::default().fg(Color::Cyan)),
                ]));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Tab/↑↓", Style::default().fg(Color::Yellow)),
                Span::raw("=next  "),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw("=confirm  "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw("=cancel"),
            ]));

            let paragraph = Paragraph::new(lines);
            frame.render_widget(paragraph, inner);
        }
        Modal::PromptDeleteConfirm { name, .. } => {
            let area = centered_rect(50, 5, frame.area());
            let block = Block::default()
                .title(" Delete Prompt ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red));
            let text = vec![
                Line::from(format!("Delete prompt \"{}\"?", name)),
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
        Modal::ModelSelector { selected } => {
            let height = (app.models.len() + 3).min(12) as u16;
            let area = centered_rect(50, height, frame.area());
            let block = Block::default()
                .title(" Select Model ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(area);
            frame.render_widget(ratatui::widgets::Clear, area);
            frame.render_widget(block, area);

            if app.models.is_empty() {
                let empty = Paragraph::new("  No models available")
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(empty, inner);
            } else {
                let items: Vec<ListItem> = app
                    .models
                    .iter()
                    .enumerate()
                    .map(|(i, model)| {
                        let is_selected = i == *selected;
                        let is_active = app.selected_model.as_deref() == Some(&model.id);
                        let style = if is_selected {
                            Style::default().fg(Color::Black).bg(Color::Cyan)
                        } else if is_active {
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };
                        let marker = if is_active { "● " } else { "  " };
                        let display = model.display_name.as_deref().unwrap_or(&model.id);
                        ListItem::new(Line::from(Span::styled(
                            format!("{}{}", marker, display),
                            style,
                        )))
                    })
                    .collect();
                let list = List::new(items);
                frame.render_widget(list, inner);
            }
        }
        Modal::ProviderSelector { selected } => {
            let height = (app.providers.len() + 3).min(10) as u16;
            let area = centered_rect(60, height, frame.area());
            let block = Block::default()
                .title(" Select Provider ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(area);
            frame.render_widget(ratatui::widgets::Clear, area);
            frame.render_widget(block, area);

            if app.providers.is_empty() {
                let empty = Paragraph::new("  No providers configured")
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(empty, inner);
            } else {
                let items: Vec<ListItem> = app
                    .providers
                    .iter()
                    .enumerate()
                    .map(|(i, provider)| {
                        let is_selected = i == *selected;
                        let is_active = app.provider_id == provider.id;
                        let style = if is_selected {
                            Style::default().fg(Color::Black).bg(Color::Cyan)
                        } else if is_active {
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };
                        let marker = if is_active { "● " } else { "  " };
                        let display =
                            format!("{}{} — {}", marker, provider.name, provider.base_url);
                        ListItem::new(Line::from(Span::styled(display, style)))
                    })
                    .collect();
                let list = List::new(items);
                frame.render_widget(list, inner);
            }
        }
        Modal::GenerationSettings {
            temperature,
            top_p,
            max_tokens,
            field,
        } => {
            let area = centered_rect(40, 8, frame.area());
            let block = Block::default()
                .title(" Generation Settings (Ctrl+G) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(area);
            frame.render_widget(ratatui::widgets::Clear, area);
            frame.render_widget(block, area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(inner);

            let fields: [(&str, &String); 3] = [
                ("Temperature", temperature),
                ("Top P", top_p),
                ("Max Tokens", max_tokens),
            ];

            for (i, (label, value)) in fields.iter().enumerate() {
                let style = if i == *field {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };
                let cursor = if i == *field { "▍" } else { "" };
                let line = Line::from(vec![
                    Span::styled(format!("  {}: ", label), style),
                    Span::styled((*value).clone(), Style::default().fg(Color::White)),
                    Span::styled(cursor, Style::default().fg(Color::Cyan)),
                ]);
                frame.render_widget(Paragraph::new(line), chunks[i]);
            }

            let hint = Line::from(vec![
                Span::styled(
                    "↑↓",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("=field  "),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("=save  "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw("=cancel  (empty = provider default)"),
            ]);
            frame.render_widget(Paragraph::new(hint), chunks[4]);
        }
        Modal::BranchHistory { selected } => {
            let height = (app.messages.len() + 4).min(20) as u16;
            let area = centered_rect(60, height, frame.area());
            let block = Block::default()
                .title(" Branch History (Ctrl+B) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(area);
            frame.render_widget(ratatui::widgets::Clear, area);
            frame.render_widget(block, area);

            if app.messages.is_empty() {
                let empty = Paragraph::new("  No messages in this conversation")
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(empty, inner);
            } else {
                let items: Vec<ListItem> = app
                    .messages
                    .iter()
                    .enumerate()
                    .map(|(i, msg)| {
                        let is_selected = i == *selected;
                        let style = if is_selected {
                            Style::default().fg(Color::Black).bg(Color::Cyan)
                        } else {
                            Style::default()
                        };
                        let role_label = match msg.role {
                            Role::User => "you",
                            Role::Assistant => "assistant",
                            Role::System => "system",
                            Role::Tool => "tool",
                        };
                        let branch_marker = if msg.parent_id.is_some() {
                            "↩ "
                        } else {
                            "  "
                        };
                        let snippet = if msg.content.len() > 40 {
                            format!("{}...", &msg.content[..37])
                        } else {
                            msg.content.clone()
                        };
                        let display = format!("{}{}: {}", branch_marker, role_label, snippet);
                        ListItem::new(Line::from(Span::styled(display, style)))
                    })
                    .collect();
                let list = List::new(items);
                frame.render_widget(list, inner);
            }
        }
    }
}

fn render_search_screen(frame: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    let search_input = Paragraph::new(format!("/ {}", app.search_query))
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(" Search (Ctrl+F, Esc=back) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
    frame.render_widget(search_input, chunks[0]);

    if app.search_results.is_empty() && !app.search_query.is_empty() {
        let no_results = Paragraph::new("  No results found.")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().title(" Results ").borders(Borders::ALL));
        frame.render_widget(no_results, chunks[1]);
        return;
    }

    if app.search_query.is_empty() {
        let hint = Paragraph::new(
            "  Type to search messages and prompts...\n  Enter to open result, ↑↓ to navigate",
        )
        .style(Style::default().fg(Color::DarkGray))
        .wrap(Wrap { trim: false })
        .block(Block::default().title(" Results ").borders(Borders::ALL));
        frame.render_widget(hint, chunks[1]);
        return;
    }

    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let is_selected = i == app.search_selection;
            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default()
            };
            match result {
                llm_tui::app::SearchResultEntry::Message {
                    message,
                    conversation_title,
                    ..
                } => {
                    let role_label = match message.role {
                        Role::User => "you",
                        Role::Assistant => "assistant",
                        Role::System => "system",
                        Role::Tool => "tool",
                    };
                    let snippet = if message.content.len() > 60 {
                        format!("{}...", &message.content[..57])
                    } else {
                        message.content.clone()
                    };
                    let line = format!(
                        "  [msg] {} in {}: {}",
                        role_label, conversation_title, snippet
                    );
                    ListItem::new(Line::from(Span::styled(line, style)))
                }
                llm_tui::app::SearchResultEntry::Prompt { prompt } => {
                    let tags = if prompt.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", prompt.tags.join(", "))
                    };
                    let line = format!("  [prompt] {}{}", prompt.name, tags);
                    ListItem::new(Line::from(Span::styled(line, style)))
                }
            }
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(" Results ({}) ", app.search_results.len()))
            .borders(Borders::ALL),
    );
    frame.render_widget(list, chunks[1]);
}

fn render_prompts_screen(frame: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    if app.prompts.is_empty() {
        let empty =
            Paragraph::new("  No prompts yet. Press n to create one.\n  q to return to chat.")
                .style(Style::default().fg(Color::DarkGray))
                .wrap(Wrap { trim: false })
                .block(
                    Block::default()
                        .title(" Prompts ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                );
        frame.render_widget(empty, chunks[0]);
    } else {
        let items: Vec<ListItem> = app
            .prompts
            .iter()
            .enumerate()
            .map(|(i, prompt)| {
                let is_selected = i == app.prompt_selection;
                let style = if is_selected {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default()
                };
                let tags = if prompt.tags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", prompt.tags.join(", "))
                };
                let desc = prompt
                    .description
                    .as_deref()
                    .unwrap_or("")
                    .chars()
                    .take(40)
                    .collect::<String>();
                let display = if desc.is_empty() {
                    format!("  {}{}", prompt.name, tags)
                } else {
                    format!("  {} — {}{}", prompt.name, desc, tags)
                };
                ListItem::new(Line::from(Span::styled(display, style)))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(" Prompts ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(list, chunks[0]);
    }

    let help = Line::from(vec![
        Span::styled("j/k", Style::default().fg(Color::Yellow)),
        Span::raw("=nav  "),
        Span::styled("n", Style::default().fg(Color::Green)),
        Span::raw("=new  "),
        Span::styled("e", Style::default().fg(Color::Cyan)),
        Span::raw("=edit  "),
        Span::styled("d", Style::default().fg(Color::Red)),
        Span::raw("=delete  "),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw("=use  "),
        Span::styled("q", Style::default().fg(Color::DarkGray)),
        Span::raw("=back"),
    ]);
    frame.render_widget(Paragraph::new(help), chunks[1]);
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
