//! Content Menu UI Component
//!
//! This module defines a `ContentMenu` UI widget built with `ratatui`, allowing navigation between
//! multiple menu items.

use std::rc::Rc;

use color_eyre::{Result, eyre::Error};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, List, Paragraph},
};

use crate::config::CONFIG;

/// Type alias for the render function of a `MenuItem`.
pub type FnRenderMenuItem<'a> = Box<dyn Fn(Rect) -> WiMenuItem<'a> + 'a>;

/// Enum representing possible UI content types within a `MenuItem`.
pub enum EnContentMenuItem<'a> {
    Paragraph(Paragraph<'a>),
    List(List<'a>),
}

/// A widget rendered by a `MenuItem`, containing primary and optional overlay content.
pub struct WiMenuItem<'a> {
    pub content: EnContentMenuItem<'a>,
    pub overlay: EnContentMenuItem<'a>,
    pub overlay_area: Rect,
    pub show_overlay: bool,
}

/// Represents a single menu item in the content menu.
pub struct StMenuItem<'a> {
    pub title: String,
    pub event: Box<dyn Fn(&Event) -> Result<(), Error>>,
    pub starter: Box<dyn Fn() -> Result<(), Error>>,
    pub render: FnRenderMenuItem<'a>,
}

/// Main structure for managing and rendering a list of interactive menu items.
pub struct ContentMenu<'a> {
    selected_button: usize,
    items: Vec<StMenuItem<'a>>,
}

impl<'a> ContentMenu<'a> {
    /// Creates a new content menu and triggers each item's `starter` function.
    pub fn new(items: Vec<StMenuItem<'a>>) -> Self {
        for menu_item in &items {
            let _ = (menu_item.starter)();
        }

        Self {
            selected_button: 0,
            items,
        }
    }

    /// Dispatches an input event to the currently selected menu item and handles key navigation.
    pub fn handle_events(&mut self, event: &Event) -> Result<()> {
        if let Some(menu_item) = self.items.get(self.selected_button) {
            (menu_item.event)(&event)?;
        }

        match event {
            Event::Key(key_event) => {
                self.handle_key_event(&key_event);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Renders the currently selected menu item and the navigation list on screen.
    pub fn render(&self, frame: &mut Frame, area: Rc<[Rect]>) {
        if let Some(menu_item) = self.items.get(self.selected_button) {
            let widget = (menu_item.render)(area[0]);

            match widget.content {
                EnContentMenuItem::Paragraph(p) => frame.render_widget(p, area[0]),
                EnContentMenuItem::List(l) => frame.render_widget(l, area[0]),
            }

            if widget.show_overlay {
                match widget.overlay {
                    EnContentMenuItem::Paragraph(p) => frame.render_widget(p, widget.overlay_area),
                    EnContentMenuItem::List(l) => frame.render_widget(l, widget.overlay_area),
                }
            }
        }

        let mut button_lines = Vec::new();
        for (i, item) in self.items.iter().enumerate() {
            let style = if i == self.selected_button {
                ratatui::style::Style::default()
                    .fg(CONFIG().themes.nav_selected_fg_color)
                    .bg(CONFIG().themes.nav_selected_bg_color)
            } else {
                ratatui::style::Style::default()
            };
            button_lines.push(Line::styled(item.title.clone(), style));
        }

        let borders = if CONFIG().themes.borders_on {
            Borders::LEFT | Borders::TOP | Borders::BOTTOM
        } else {
            Borders::NONE
        };

        let paragraph = Paragraph::new(button_lines).block(
            Block::default()
                .borders(borders)
                .border_type(CONFIG().themes.border_type)
                .border_style(Style::default().fg(CONFIG().themes.border_color)),
        );

        frame.render_widget(paragraph, area[1]);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) {
        if key_event.kind != KeyEventKind::Press {
            return;
        }

        let c = CONFIG();

        if c.key_matches(key_event, &c.keybinds.nav_up) {
            self.move_selected_up();
        } else if c.key_matches(key_event, &c.keybinds.nav_down) {
            self.move_selected_down();
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
