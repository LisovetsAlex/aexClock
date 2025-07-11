use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols::bar,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, Padding, Paragraph},
};
use std::{
    process::Command,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use crate::{
    config::CONFIG,
    widgets::content_menu::{EnContentMenuItem, WiMenuItem},
};

#[derive(Clone)]
pub struct AudioMixer {
    selected_audio: usize,
    selected_id: String,
    selected_volume: u8,
    audio_list: Vec<(String, u8, String)>,
    started_refresh: bool,
}

impl AudioMixer {
    pub fn new() -> Self {
        Self {
            selected_audio: 0,
            audio_list: Vec::new(),
            started_refresh: false,
            selected_volume: 0,
            selected_id: String::new(),
        }
    }

    pub fn handle_events(&mut self, event: &Event) -> Result<()> {
        match event {
            Event::Key(key_event) => {
                self.handle_key_event(&key_event);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Build the main and overlay widgets for rendering.
    pub fn get_widget(&self, area: Rect) -> WiMenuItem<'static> {
        let max_width = (area.width - 4) as usize;

        let list = self.make_audio_widget_list(max_width);

        let (overlay, overlay_area) = self.make_empty_prompt();

        WiMenuItem {
            content: EnContentMenuItem::List(list),
            overlay,
            overlay_area,
            show_overlay: false,
        }
    }

    /// Start background thread to refresh audio list.
    pub fn start_auto_refresh(this: Arc<Mutex<Self>>) {
        {
            let mut guard = this.lock().unwrap();
            if guard.started_refresh {
                return;
            }
            guard.started_refresh = true;
        };

        std::thread::spawn(move || {
            loop {
                let new_list = AudioMixer::make_audio_list();

                if let Ok(mut am) = this.lock() {
                    am.audio_list = new_list;
                    if am.audio_list.len() != 0 {
                        am.selected_volume = am.audio_list[am.selected_audio].1;
                        am.selected_id = am.audio_list[am.selected_audio].2.clone();
                    }
                }

                sleep(Duration::from_secs(3));
            }
        });
    }

    // ====== Input Handling ======

    fn handle_key_event(&mut self, key_event: &KeyEvent) {
        if key_event.kind != KeyEventKind::Press {
            return;
        }

        let c = CONFIG();

        if c.key_matches(key_event, &c.keybinds.content_up) {
            self.move_selected_up();
        } else if c.key_matches(key_event, &c.keybinds.content_down) {
            self.move_selected_down();
        } else if c.key_matches(key_event, &c.keybinds.content_left) {
            let id = self.selected_id.clone();
            self.add_volume(&id, 5, false);
        } else if c.key_matches(key_event, &c.keybinds.content_right) {
            let id = self.selected_id.clone();
            self.add_volume(&id, 5, true);
        }
    }

    fn move_selected_down(&mut self) {
        self.selected_audio += 1;
        if self.selected_audio >= self.audio_list.len() {
            self.selected_audio = 0;
        }

        let (_, volume, id) = &self.audio_list[self.selected_audio];
        self.selected_volume = *volume;
        self.selected_id = id.clone();
    }

    fn move_selected_up(&mut self) {
        let number = self.selected_audio as i32 - 1;
        self.selected_audio = self.selected_audio.saturating_sub(1);

        if number < 0 {
            self.selected_audio = self.audio_list.len() - 1;
        }

        let (_, volume, id) = &self.audio_list[self.selected_audio];
        self.selected_volume = *volume;
        self.selected_id = id.clone();
    }

    // ====== Rendering UI Components ======

    fn make_audio_widget_list(&self, max_width: usize) -> List<'static> {
        let mut items: Vec<Line> = Vec::new();
        let mut audio_lines: Vec<Line> = Vec::new();

        for (i, (audio, volume, id)) in self.audio_list.iter().enumerate() {
            let color = if i == self.selected_audio {
                CONFIG().themes.content_selected_color
            } else {
                CONFIG().themes.fg_color
            };

            let mut name_line = self.make_audio_name_line(audio);
            name_line = name_line.style(color);

            let mut volume_line = self.make_audio_volume_line(id, volume, max_width);
            volume_line = volume_line.style(color);

            audio_lines.push(name_line);
            audio_lines.push(volume_line);
        }

        items.append(&mut audio_lines);

        let theme = &CONFIG().themes;

        let block = Block::default()
            .borders(if theme.borders_on {
                Borders::ALL
            } else {
                Borders::NONE
            })
            .border_type(theme.border_type)
            .border_style(Style::default().fg(theme.border_color))
            .padding(Padding {
                left: 1,
                right: 1,
                top: 0,
                bottom: 0,
            });

        List::new(items).block(block)
    }

    fn make_audio_name_line(&self, name: &str) -> Line<'static> {
        let audio_name = format!("♪ {}", name.to_string());
        Line::from(Span::raw(audio_name))
    }

    fn make_audio_volume_line(&self, id: &String, volume: &u8, max_width: usize) -> Line<'static> {
        let bar_length = max_width.saturating_sub(2) as u8;
        let clamped_volume = *volume;
        let filled_len =
            ((clamped_volume as u32 * bar_length as u32) / 100).clamp(0, bar_length as u32) as u8;
        let empty_len = bar_length.saturating_sub(filled_len);

        let is_selected = *id == self.selected_id;
        let theme = &CONFIG().themes;

        let bar_side_color = if !is_selected {
            theme.bar_selected_side_color
        } else {
            theme.bar_side_color
        };
        let filled_color = if !is_selected {
            theme.bar_selected_filled_color
        } else {
            theme.bar_filled_color
        };
        let empty_color = if !is_selected {
            theme.bar_selected_empty_color
        } else {
            theme.bar_empty_color
        };

        let open_bracket = Span::styled("[", Style::default().fg(bar_side_color));
        let filled = Span::styled(
            "=".repeat(filled_len as usize),
            Style::default().fg(filled_color),
        );
        let empty = Span::styled(
            "-".repeat(empty_len as usize),
            Style::default().fg(empty_color),
        );
        let close_bracket = Span::styled("]", Style::default().fg(bar_side_color));

        Line::from(vec![open_bracket, filled, empty, close_bracket])
    }

    fn make_empty_prompt(&self) -> (EnContentMenuItem<'static>, Rect) {
        (
            EnContentMenuItem::Paragraph(
                Paragraph::new("").style(Style::default().bg(CONFIG().themes.bg_color)),
            ),
            Rect::default(),
        )
    }

    // ====== audio-related Commands ======

    pub fn add_volume(&mut self, id: &String, amount: u8, increase: bool) {
        let mut am = amount;
        let vol: i32 = self.selected_volume as i32;
        if vol + (amount as i32) > 100 && increase {
            am = 0;
        }
        if vol - (amount as i32) < 0 && !increase {
            am = 0;
        }

        let volume_change = format!("{}{}%", if increase { "+" } else { "-" }, am);

        let status = Command::new("pactl")
            .args(["set-sink-input-volume", id, &volume_change])
            .status();

        match status {
            Ok(s) if s.success() => {
                self.audio_list = AudioMixer::make_audio_list();
            }
            Ok(s) => {}
            Err(e) => {}
        }
    }

    fn make_audio_list() -> Vec<(String, u8, String)> {
        let output = Command::new("pactl")
            .arg("list")
            .arg("sink-inputs")
            .output()
            .expect("Failed to run pactl");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut result = Vec::new();

        let mut current_id = String::new();
        let mut current_name = String::new();
        let mut current_volume = 0;

        for line in stdout.lines() {
            if line.trim_start().starts_with("Sink Input") {
                if let Some(id) = line.split('#').nth(1) {
                    current_id = id.to_string();
                }
            }

            if line.trim_start().starts_with("Volume:") {
                if let Some(percent) = line.split('/').nth(1) {
                    let volume = percent
                        .trim()
                        .trim_end_matches('%')
                        .parse::<u8>()
                        .unwrap_or(0);
                    current_volume = volume;
                }
            }

            if line.trim_start().starts_with("application.name =") {
                if let Some(name) = line.split('=').nth(1) {
                    current_name = name.trim().trim_matches('"').to_string();
                    result.push((current_name.clone(), current_volume, current_id.clone()));
                }
            }
        }

        result
    }
}
