use crate::events::{AppEvent, UserEvent};
use crate::persistence::Database;

pub struct App {
    pub db: Database,
    pub should_quit: bool,
    pub input: String,
}

impl App {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            should_quit: false,
            input: String::new(),
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        if let AppEvent::User(UserEvent::Quit) = event {
            self.should_quit = true;
        }
    }
}
