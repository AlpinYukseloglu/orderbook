use std::cell::RefCell;
use std::error;
use std::rc::Rc;
use crate::book::orderbook::Orderbook;
use crate::bank::account::{Account, AccountType};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub counter: u8,
    pub updates: Vec<String>,
    pub positions: Vec<u64>,
    pub command_line: String,

    // session orderbook
    pub session_book: Orderbook,

    // user account
    pub user_account: Rc<RefCell<Account>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            counter: 0,
            updates: vec![String::new()],
            positions: vec![0],
            command_line: String::new(),
            session_book: Orderbook::new(0),
            user_account: Rc::new(RefCell::new(Account::new(0, AccountType::Individual))),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        if let Some(res) = self.counter.checked_add(1) {
            self.counter = res;
        }
    }

    pub fn decrement_counter(&mut self) {
        if let Some(res) = self.counter.checked_sub(1) {
            self.counter = res;
        }
    }
}
