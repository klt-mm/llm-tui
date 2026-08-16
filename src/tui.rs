use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io::{self, stdout};
use std::time::Duration;

use crate::app::App;
use crate::events::{AppEvent, UserEvent};

pub async fn run(app: &mut App) -> Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = loop {
        terminal.draw(|frame| render(frame, app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                let event = match key.code {
                    KeyCode::Esc => Some(AppEvent::User(UserEvent::Quit)),
                    KeyCode::Enter => Some(AppEvent::User(UserEvent::SendMessage)),
                    KeyCode::Char(c) => {
                        app.input.push(c);
                        Some(AppEvent::User(UserEvent::InputChanged(app.input.clone())))
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                        Some(AppEvent::User(UserEvent::InputChanged(app.input.clone())))
                    }
                    _ => None,
                };

                if let Some(event) = event {
                    app.handle_event(event);
                }
            }
        }

        if app.should_quit {
            break Ok(());
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

    frame.render_widget(
        Paragraph::new("llm-tui  |  provider: not configured  |  model: —"),
        chunks[0],
    );

    frame.render_widget(
        Paragraph::new("No conversation selected.")
            .block(Block::default().title(" Chat ").borders(Borders::ALL)),
        chunks[1],
    );

    frame.render_widget(
        Paragraph::new(app.input.as_str())
            .block(Block::default().title(" Input ").borders(Borders::ALL)),
        chunks[2],
    );
}
