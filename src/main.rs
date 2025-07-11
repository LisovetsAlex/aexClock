mod config;
mod widgets;
use crate::{
    config::{CONFIG, init_config},
    widgets::{audio_mixer::AudioMixer, content_menu::StMenuItem, net_connect::NetConnect},
};
use color_eyre::{Result, eyre::Error};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use widgets::clock::ClockWidget;
use widgets::content_menu::ContentMenu;

fn main() -> Result<()> {
    init_config()?;

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
    let items = vec![make_netconnect_menu_item(), make_audiomixer_menu_item()];
    let mut content_menu = ContentMenu::new(items);

    let tick_rate = Duration::from_secs(1);
    let mut last_tick = Instant::now();

    loop {
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if event::poll(timeout)? {
            let quit = dispatch_events(&mut content_menu)?;
            if quit {
                break;
            }

            terminal.draw(|f| {
                render(f, &content_menu);
            })?;
        }

        if last_tick.elapsed() >= tick_rate {
            terminal.draw(|f| {
                render(f, &content_menu);
            })?;
            last_tick = Instant::now();
        }
    }

    Ok(())
}

fn render(frame: &mut Frame, menu: &ContentMenu) {
    let clock_frame = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(frame.area());

    let menu_frame = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(clock_frame[1]);

    ClockWidget::render(frame, clock_frame[0]);
    menu.render(frame, menu_frame);
}

fn dispatch_events(menu: &mut ContentMenu) -> Result<bool, Error> {
    let event = event::read()?;

    menu.handle_events(&event)?;

    match event {
        Event::Key(key_event) => {
            if key_event.kind != KeyEventKind::Press {
                return Ok(false);
            }

            if CONFIG().key_matches(&key_event, &CONFIG().keybinds.quit) {
                return Ok(true);
            } else {
                return Ok(false);
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
        render: Box::new(move |area| render_nc.lock().unwrap().get_widget(area)),
    }
}

fn make_audiomixer_menu_item() -> StMenuItem<'static> {
    let event_am = Arc::new(Mutex::new(AudioMixer::new()));
    let render_am = event_am.clone();
    let refresh_am = event_am.clone();

    StMenuItem {
        title: "Audio".into(),
        event: Box::new(move |event: &Event| {
            event_am.lock().unwrap().handle_events(&event)?;
            Ok(())
        }),
        starter: Box::new(move || {
            AudioMixer::start_auto_refresh(refresh_am.clone());
            Ok(())
        }),
        render: Box::new(move |area| render_am.lock().unwrap().get_widget(area)),
    }
}
