use std::{rc::Rc, thread::sleep, time::Duration};
use chrono::{Local, Timelike};
use color_eyre::Result;
use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEvent, MouseEvent, MouseEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode}};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect}, style::{Color, Stylize}, text::{Line, Span, Text}, widgets::{Block, Borders, List, Paragraph, Widget}, DefaultTerminal, Frame
};
use crossterm::event::{KeyCode, KeyEventKind};

pub enum MenuItemContent<'a> {
    Paragraph(Paragraph<'a>),
    List(List<'a>),
}

pub struct MenuItem<'a> {
    pub title: String,
    pub content: MenuItemContent<'a>,
}

pub struct ContentMenu<'a> {
    selected_button: usize,
    items: Vec<MenuItem<'a>>
}

impl<'a> ContentMenu<'a> {
    pub fn new(items: Vec<MenuItem<'a>>) -> Self {
        Self {
            selected_button: 0,
            items,
        }
    }

    pub fn handle_events(&mut self, event: &Event) -> Result<()>  {
        match event {
            Event::Key(key_event) => {
                self.handle_key_event(&key_event); 
                Ok(())
            }
            _ => {
                Ok(())
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rc<[Rect]>) {
        let area = area;

        if let Some(menu_item) = self.items.get(self.selected_button) {
            match &menu_item.content {
                MenuItemContent::Paragraph(p) => frame.render_widget(p, area[0]),
                MenuItemContent::List(l) => frame.render_widget(l, area[0]),
            }
        }

        let mut button_lines = Vec::new();
        for (i, item) in self.items.iter().enumerate() {
            let style = if i == self.selected_button {
                ratatui::style::Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                ratatui::style::Style::default()
            };
            button_lines.push(Line::styled(item.title.clone(), style));
        }

        let paragraph = Paragraph::new(button_lines)
            .block(Block::default().borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM));

        frame.render_widget(paragraph, area[1]);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) {
        if key_event.kind != KeyEventKind::Press {
            return;
        }

        match key_event.code {
            KeyCode::Up => {
                self.move_selected_up();
            }
            KeyCode::Down => {
                self.move_selected_down();
            }
            _ => { }
        }
    }

    fn move_selected_down(&mut self) {
        self.selected_button += 1;

        if self.selected_button >= self.items.len() {
            self.selected_button = 0;
        }
    }

    fn move_selected_up(&mut self) {
        let number = self.selected_button as i32 - 1;
        self.selected_button = self.selected_button.saturating_sub(1); 

        if number < 0 {
            self.selected_button = self.items.len() - 1;
        }
    }

}
