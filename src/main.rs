use std::{thread::sleep, time::Duration};
mod widgets;
use widgets::clock::ClockWidget;
use chrono::{Local, Timelike};
use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect}, style::{Color, Stylize}, text::{Line, Span, Text}, widgets::{Block, Borders, Paragraph}, DefaultTerminal, Frame
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(|f|{
            ClockWidget::render(f);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(_) = event::read()? {
                break Ok(());
            }
        }
        
        sleep(Duration::from_secs(1));
    }
}