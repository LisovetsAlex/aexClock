use std::{thread::sleep, time::Duration};
mod widgets;
use widgets::clock::ClockWidget;
use widgets::content_menu::ContentMenu;
use chrono::{Local, Timelike};
use color_eyre::{eyre::Error, Result};
use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode}};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect}, style::{Color, Stylize}, text::{Line, Span, Text}, widgets::{Block, Borders, List, ListItem, Paragraph}, DefaultTerminal, Frame
};

use crate::widgets::content_menu::{MenuItem, MenuItemContent};

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnableMouseCapture)?;

    let terminal = ratatui::init();
    let result = run(terminal);

    execute!(stdout, DisableMouseCapture)?;
    disable_raw_mode()?;
    ratatui::restore();

    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let items = vec![
        MenuItem {
            title: "Button 1".into(),
            content: MenuItemContent::Paragraph(
                Paragraph::new("This is content for Button 1.")
                    .block(Block::default().borders(Borders::ALL).title("Content")),
            ),
        },
        MenuItem {
            title: "Button 2".into(),
            content: make_list_content("Item List", &["Apple", "Banana", "Orange"]),
        },
    ];

    let mut content_menu: ContentMenu = ContentMenu::new(items);

    loop {
        terminal.draw(|f| {
            render(f, &content_menu);
        })?;

        if poll_and_dispatch_events(&mut content_menu)? {
            break Ok(());
        }

        sleep(Duration::from_millis(9));
    }
}


fn render(frame: &mut Frame, menu: &ContentMenu) {
    let clock_frame = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(frame.area());

    let menu_frame = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(clock_frame[1]);

    ClockWidget::render(frame, clock_frame[0]);
    menu.render(frame, menu_frame);
}

fn poll_and_dispatch_events(menu: &mut ContentMenu) -> Result<bool, Error> {
    if !event::poll(Duration::from_millis(9))? {
        return Ok(false);
    }

    let event = event::read()?;

    menu.handle_events(&event)?;

    match event {
        Event::Key(key_event) => {
            if key_event.kind != KeyEventKind::Press {
                return Ok(false);
            }

            match key_event.code {
                KeyCode::Char('q') => return Ok(true),
                _ => { 
                    return Ok(false);
                }
            }
        }
        _ => {
            return Ok(false);
        }
    }
}

fn make_list_content<'a>(title: &'a str, items: &'a [&'a str]) -> MenuItemContent<'a> {
    let list_items: Vec<ListItem<'a>> = items.iter().map(|&s| ListItem::new(s)).collect();

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title(title));

    MenuItemContent::List(list)
}