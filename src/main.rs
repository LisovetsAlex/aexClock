use std::{ sync::{Arc, Mutex}, thread::sleep, time::Duration};
mod widgets;
use widgets::clock::ClockWidget;
use widgets::content_menu::ContentMenu;
use color_eyre::{eyre::Error, Result};
use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode}};
use ratatui::{
    layout::{Constraint, Direction, Layout}, DefaultTerminal, Frame
};

use crate::widgets::{content_menu::{StMenuItem}, net_connect::NetConnect};

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
        make_netconnect_menu_item()
    ];

    let mut content_menu = ContentMenu::new(items);

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

fn make_netconnect_menu_item() -> StMenuItem<'static> {
    let event_nc = Arc::new(Mutex::new(NetConnect::new()));
    let render_nc = event_nc.clone();
    let refresh_nc = event_nc.clone();

    StMenuItem {
        title: "Internet".into(),
        event: Box::new(move |event: &Event| {
            event_nc.lock().unwrap().handle_events(&event)?;
            Ok(())
        }),
        starter: Box::new(move || {
            NetConnect::start_auto_refresh(refresh_nc.clone());
            Ok(())
        }),
        render: Box::new(move |area| {
            render_nc.lock().unwrap().get_widget(area)
        }),
    }
}