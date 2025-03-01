use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::{App, FocusPanel, InputMode};
use crate::email::EmailClient;

pub fn draw<B: Backend, T: EmailClient>(f: &mut Frame<B>, app: &App<T>) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
        .split(f.size());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(main_chunks[0]);

    draw_email_list(f, app, chunks[0]);
    draw_email_content(f, app, chunks[1]);
    draw_status_bar(f, app, main_chunks[1]);

    match app.input_mode {
        InputMode::Help => draw_help(f),
        InputMode::Search => draw_search(f, app),
        _ => {}
    }
}

fn draw_email_list<B: Backend, T: EmailClient>(f: &mut Frame<B>, app: &App<T>, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_emails
        .iter()
        .map(|&idx| &app.emails[idx])
        .map(|email| {
            let date = email.date.format("%Y-%m-%d %H:%M").to_string();
            let content = vec![
                Spans::from(vec![Span::styled(
                    &email.subject,
                    Style::default().add_modifier(Modifier::BOLD),
                )]),
                Spans::from(vec![
                    Span::styled("From: ", Style::default().fg(Color::Blue)),
                    Span::raw(&email.sender),
                ]),
                Spans::from(vec![
                    Span::styled("Date: ", Style::default().fg(Color::Blue)),
                    Span::raw(date),
                ]),
                Spans::from(""),
            ];
            ListItem::new(content)
        })
        .collect();

    let block_style = match app.focus {
        FocusPanel::EmailList => Style::default().fg(Color::Yellow),
        _ => Style::default(),
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title("Emails")
                .borders(Borders::ALL)
                .style(block_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // Create a local copy of the list state
    let mut list_state = app.list_state.clone();
    f.render_stateful_widget(list, area, &mut list_state);
}

fn draw_email_content<B: Backend, T: EmailClient>(f: &mut Frame<B>, app: &App<T>, area: Rect) {
    let block_style = match app.focus {
        FocusPanel::EmailContent => Style::default().fg(Color::Yellow),
        _ => Style::default(),
    };

    let content = if let Some(email) = app.selected_email() {
        let mut text = Text::from(vec![
            Spans::from(vec![
                Span::styled(
                    "Subject: ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &email.subject,
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]),
            Spans::from(vec![
                Span::styled(
                    "From: ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&email.sender),
            ]),
            Spans::from(vec![
                Span::styled(
                    "Date: ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(email.date.format("%Y-%m-%d %H:%M:%S").to_string()),
            ]),
            Spans::from(""),
            Spans::from(""),
        ]);

        // Split body by newlines and add each line
        for line in email.body.lines() {
            text.extend(Text::from(line));
        }

        text
    } else {
        Text::from("No email selected")
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title("Content")
                .borders(Borders::ALL)
                .style(block_style),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn draw_status_bar<B: Backend, T: EmailClient>(f: &mut Frame<B>, app: &App<T>, area: Rect) {
    let status = if app.is_loading() {
        // Create a simple spinner animation based on time
        let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let idx = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            / 100) as usize
            % spinner_chars.len();

        format!("{} Loading emails...", spinner_chars[idx])
    } else if let Some((msg, _)) = &app.status_message {
        msg.clone()
    } else {
        match app.input_mode {
            InputMode::Normal => "Normal mode | Press ? for help | q to quit".to_string(),
            InputMode::EmailView => {
                "Email view mode | Press Esc to return | ? for help".to_string()
            }
            InputMode::Help => "Help mode".to_string(),
            InputMode::Search => "Search mode".to_string(),
        }
    };

    let status_style = if app.is_loading() {
        Style::default().fg(Color::Yellow)
    } else if app.status_message.is_some() {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let status_bar = Paragraph::new(status).style(status_style);

    f.render_widget(status_bar, area);
}

fn draw_help<B: Backend>(f: &mut Frame<B>) {
    let area = centered_rect(60, 20, f.size());

    let help_text = vec![
        Spans::from(Span::styled(
            "Keyboard Shortcuts:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(""),
        Spans::from(vec![
            Span::styled("j/k", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" or "),
            Span::styled("↑/↓", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Navigate up/down through emails"),
        ]),
        Spans::from(vec![
            Span::styled("l", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" or "),
            Span::styled("→", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" or "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - View selected email details"),
        ]),
        Spans::from(vec![
            Span::styled("h", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" or "),
            Span::styled("←", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" or "),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Return to email list"),
        ]),
        Spans::from(vec![
            Span::styled("g", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Go to first email"),
        ]),
        Spans::from(vec![
            Span::styled("G", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Go to last email"),
        ]),
        Spans::from(vec![
            Span::styled("r", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Refresh emails"),
        ]),
        Spans::from(vec![
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Quit application"),
        ]),
        Spans::from(vec![
            Span::styled("?", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Show/hide this help menu"),
        ]),
        Spans::from(vec![
            Span::styled("/", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Search emails"),
        ]),
        Spans::from(""),
        Spans::from(Span::styled(
            "Press any key to close this help window",
            Style::default().fg(Color::Yellow),
        )),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    // Create a semi-transparent overlay effect
    let overlay = Block::default().style(Style::default().bg(Color::Black).fg(Color::White));
    f.render_widget(overlay, f.size());

    // Render the help dialog on top
    f.render_widget(help, area);
}

fn draw_search<B: Backend, T: EmailClient>(f: &mut Frame<B>, app: &App<T>) {
    let area = centered_rect(60, 10, f.size());

    let search_text = format!("Search: {}", app.search_input);
    let cursor_pos = search_text.len();

    let search_input = Paragraph::new(search_text)
        .block(
            Block::default()
                .title("Search Emails")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    // Create a semi-transparent overlay effect
    let overlay = Block::default().style(Style::default().bg(Color::Black).fg(Color::White));
    f.render_widget(overlay, f.size());

    // Render the search dialog on top
    f.render_widget(search_input, area);

    // Show cursor at the end of input
    f.set_cursor(
        area.x + cursor_pos as u16 + 1, // +1 for the border
        area.y + 1,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
