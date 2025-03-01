use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, widgets::ListState, Terminal};

use super::view;
use crate::email::{Email, EmailClient};

pub enum InputMode {
    Normal,
    Help,
    EmailView,
    Search,
}

pub enum FocusPanel {
    EmailList,
    EmailContent,
}

pub struct App<T: EmailClient> {
    pub email_client: T,
    pub emails: Vec<Email>,
    pub filtered_emails: Vec<usize>, // Indices into emails for search results
    pub selected_index: usize,
    pub input_mode: InputMode,
    pub focus: FocusPanel,
    pub list_state: ListState,
    pub status_message: Option<(String, Instant)>,
    pub search_input: String,
    should_quit: bool,
    loading: bool,
}

impl<T: EmailClient> App<T> {
    pub fn new(email_client: T) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            email_client,
            emails: Vec::new(),
            filtered_emails: Vec::new(),
            selected_index: 0,
            input_mode: InputMode::Normal,
            focus: FocusPanel::EmailList,
            list_state,
            status_message: None,
            search_input: String::new(),
            should_quit: false,
            loading: false,
        }
    }

    pub fn search(&mut self, query: String) {
        self.filtered_emails.clear();

        if query.is_empty() {
            // If search is empty, include all emails
            self.filtered_emails = (0..self.emails.len()).collect();
        } else {
            let query_lower = query.to_lowercase();

            // Filter emails that match the search query
            for (idx, email) in self.emails.iter().enumerate() {
                if email.subject.to_lowercase().contains(&query_lower)
                    || email.sender.to_lowercase().contains(&query_lower)
                    || email.body.to_lowercase().contains(&query_lower)
                {
                    self.filtered_emails.push(idx);
                }
            }
        }

        // Reset selection
        if !self.filtered_emails.is_empty() {
            self.selected_index = 0;
            self.list_state.select(Some(0));
        }

        // Update status message
        if self.filtered_emails.is_empty() && !query.is_empty() {
            self.set_status_message(format!("No emails found matching '{}'", query));
        } else if !query.is_empty() {
            self.set_status_message(format!(
                "Found {} emails matching '{}'",
                self.filtered_emails.len(),
                query
            ));
        }
    }

    pub async fn refresh_emails(&mut self) -> Result<()> {
        self.loading = true;
        match self.email_client.fetch_current_quarter_emails().await {
            Ok(emails) => {
                self.emails = emails;

                // Reset filtered emails to show all emails
                self.filtered_emails = (0..self.emails.len()).collect();

                if !self.emails.is_empty() {
                    self.selected_index = self.selected_index.min(self.emails.len() - 1);
                    self.list_state.select(Some(self.selected_index));
                }
                self.set_status_message("Emails refreshed successfully".to_string());
                Ok(())
            }
            Err(e) => {
                self.set_status_message(format!("Failed to fetch emails: {}", e));
                Err(e)
            }
        }
    }

    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some((message, Instant::now()));
        self.loading = false;
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Fetch emails
        self.set_status_message("Loading emails...".to_string());
        let _ = self.refresh_emails().await;

        // Main loop
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();
        let status_timeout = Duration::from_secs(5);

        loop {
            terminal.draw(|f| view::draw(f, self))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match self.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('q') => self.should_quit = true,
                            KeyCode::Char('?') => self.input_mode = InputMode::Help,
                            KeyCode::Char('r') => {
                                // Just set the status message, we can't refresh asynchronously
                                // inside the event loop without changing the architecture
                                self.set_status_message("Can't refresh emails during UI running. Restart app to refresh.".to_string());
                            }
                            KeyCode::Char('/') => {
                                self.input_mode = InputMode::Search;
                                self.search_input.clear();
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                if !self.emails.is_empty() {
                                    self.selected_index =
                                        (self.selected_index + 1) % self.emails.len();
                                    self.list_state.select(Some(self.selected_index));
                                }
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                if !self.emails.is_empty() {
                                    self.selected_index = if self.selected_index > 0 {
                                        self.selected_index - 1
                                    } else {
                                        self.emails.len() - 1
                                    };
                                    self.list_state.select(Some(self.selected_index));
                                }
                            }
                            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                                if !self.emails.is_empty() {
                                    self.input_mode = InputMode::EmailView;
                                    self.focus = FocusPanel::EmailContent;
                                }
                            }
                            KeyCode::Char('h') | KeyCode::Left => {
                                self.focus = FocusPanel::EmailList;
                            }
                            KeyCode::Char('g') => {
                                if !self.emails.is_empty() {
                                    self.selected_index = 0;
                                    self.list_state.select(Some(self.selected_index));
                                }
                            }
                            KeyCode::Char('G') => {
                                if !self.emails.is_empty() {
                                    self.selected_index = self.emails.len() - 1;
                                    self.list_state.select(Some(self.selected_index));
                                }
                            }
                            _ => {}
                        },
                        InputMode::EmailView => match key.code {
                            KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
                                self.input_mode = InputMode::Normal;
                                self.focus = FocusPanel::EmailList;
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                if !self.emails.is_empty() {
                                    self.selected_index =
                                        (self.selected_index + 1) % self.emails.len();
                                    self.list_state.select(Some(self.selected_index));
                                }
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                if !self.emails.is_empty() {
                                    self.selected_index = if self.selected_index > 0 {
                                        self.selected_index - 1
                                    } else {
                                        self.emails.len() - 1
                                    };
                                    self.list_state.select(Some(self.selected_index));
                                }
                            }
                            KeyCode::Char('q') => self.should_quit = true,
                            KeyCode::Char('?') => self.input_mode = InputMode::Help,
                            _ => {}
                        },
                        InputMode::Help => {
                            // Any key returns from help mode
                            self.input_mode = InputMode::Normal;
                        }
                        InputMode::Search => match key.code {
                            KeyCode::Esc => {
                                // First change the mode to release the borrow
                                self.input_mode = InputMode::Normal;
                                // Clear search input
                                self.search_input.clear();
                                // Then show all emails
                                self.search(String::new());
                            }
                            KeyCode::Enter => {
                                // Clone the search input before using it
                                let query = self.search_input.clone();
                                // Set input mode first to release the borrow
                                self.input_mode = InputMode::Normal;
                                // Then perform the search
                                self.search(query);
                            }
                            KeyCode::Char(c) => {
                                self.search_input.push(c);
                            }
                            KeyCode::Backspace => {
                                self.search_input.pop();
                            }
                            _ => {}
                        },
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();

                // Clear status message after timeout
                if let Some((_, instant)) = self.status_message {
                    if instant.elapsed() >= status_timeout {
                        self.status_message = None;
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    pub fn selected_email(&self) -> Option<&Email> {
        if self.emails.is_empty() || self.filtered_emails.is_empty() {
            None
        } else if self.selected_index < self.filtered_emails.len() {
            let real_index = self.filtered_emails[self.selected_index];
            Some(&self.emails[real_index])
        } else {
            None
        }
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }
}
