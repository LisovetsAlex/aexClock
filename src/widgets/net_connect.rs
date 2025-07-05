use std::{
    process::Command,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};
use color_eyre::{Result};
use crossterm::{
    event::{Event, KeyEvent, KeyModifiers, KeyCode, KeyEventKind}
};
use ratatui::{
    layout::{Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, Padding, Paragraph}
};

use crate::widgets::content_menu::{EnContentMenuItem, WiMenuItem};

/// Manages WiFi connectivity UI, including:
/// - Listing available networks
/// - Handling password prompts and connection requests
/// - Displaying connected network
/// - Periodic auto-refresh of network list
#[derive(Clone)]
pub struct NetConnect {
    selected_ssid: usize,
    wifi_list: Vec<(String, u8)>,
    started_refresh: bool,
    connected_ssid: String,
    show_prompt: bool,
    prompt_ssid: String,
    prompt_pass: String,
}

impl NetConnect {
    // ====== Initialization ======

    /// Create a new `NetConnect` instance with default state.
    pub fn new() -> Self {
        Self {
            selected_ssid: 0,
            wifi_list: Vec::new(),
            started_refresh: false,
            connected_ssid: String::new(),
            show_prompt: false,
            prompt_ssid: String::new(),
            prompt_pass: String::new(),
        }
    }

    // ====== Public Interface Methods ======

    /// Handle input events (keys only).
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

        let list = self.make_wifi_widget_list(max_width);
        let (overlay, overlay_area) = if self.show_prompt {
            self.make_prompt(max_width, area)
        } else {
            self.make_empty_prompt()
        };

        WiMenuItem {
            content: EnContentMenuItem::List(list),
            overlay,
            overlay_area,
            show_overlay: self.show_prompt,
        }
    }

    /// Start background thread to refresh network list and connection state.
    pub fn start_auto_refresh(this: Arc<Mutex<Self>>) {
        {
            let mut guard = this.lock().unwrap();
            if guard.started_refresh {
                return;
            }
            guard.started_refresh = true;
        };

        std::thread::spawn(move || loop {
            let new_list = NetConnect::make_wifi_list();

            if let Ok(mut nc) = this.lock() {
                nc.connected_ssid = nc.get_connected_ssid();
                nc.wifi_list = new_list;
            }
            sleep(Duration::from_secs(3));
        });
    }

    // ====== Input Handling ======

    fn handle_key_event(&mut self, key_event: &KeyEvent) {
        if key_event.kind != KeyEventKind::Press {
            return;
        }

        if self.show_prompt {
            match key_event.code {
                KeyCode::Enter => {
                    self.connect_to_wifi(&self.prompt_ssid, &self.prompt_pass);
                    self.show_prompt = false;
                    self.prompt_ssid.clear();
                    self.prompt_pass.clear();
                }
                KeyCode::Backspace => {
                    self.prompt_pass.pop();
                }
                KeyCode::Char(c) => {
                    self.prompt_pass.push(c);
                }
                _ => {}
            }

            return;
        }

        match key_event.code {
            KeyCode::Up => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.move_selected_up();
                }
            }
            KeyCode::Down => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    self.move_selected_down();
                }
            }
            KeyCode::Enter => {
                self.show_prompt = true;
                self.prompt_ssid = self
                    .wifi_list
                    .get(self.selected_ssid)
                    .map(|(ssid, _)| ssid.clone())
                    .unwrap_or_default();
                self.prompt_pass.clear();
            }
            _ => {}
        }
    }

    fn move_selected_down(&mut self) {
        self.selected_ssid += 1;
        if self.selected_ssid >= self.wifi_list.len() {
            self.selected_ssid = 0;
        }
    }

    fn move_selected_up(&mut self) {
        let number = self.selected_ssid as i32 - 1;
        self.selected_ssid = self.selected_ssid.saturating_sub(1);

        if number < 0 {
            self.selected_ssid = self.wifi_list.len() - 1;
        }
    }

    // ====== Rendering UI Components ======

    fn make_wifi_widget_list(&self, max_width: usize) -> List<'static> {
        let mut items: Vec<Line> = Vec::new();

        let connected_line = format!("Connected to: {}", self.connected_ssid);
        items.push(Line::from(Span::styled(
            format!("{:<width$}", connected_line, width = max_width),
            Style::default().fg(Color::White),
        )));

        items.push(Line::from(" ".repeat(max_width)));

        let mut wifi_lines: Vec<Line> = self
            .wifi_list
            .iter()
            .enumerate()
            .map(|(i, (ssid, signal))| {
                let line = self.make_wifi_line(ssid, *signal, max_width);

                if i == self.selected_ssid && !self.show_prompt {
                    line.style(Color::Yellow)
                } else {
                    line.style(Color::White)
                }
            })
            .collect();

        items.append(&mut wifi_lines);

        List::new(items).block(
            Block::default()
                .borders(Borders::all())
                .border_type(BorderType::Rounded)
                .padding(Padding {
                    left: 1,
                    right: 1,
                    top: 0,
                    bottom: 0,
                }),
        )
    }

    fn make_prompt(&self, max_width: usize, area: Rect) -> (EnContentMenuItem<'static>, Rect) {
        let prompt_lines = self.make_prompt_lines(max_width + 1);

        let paragraph = Paragraph::new(prompt_lines)
            .style(Style::default().bg(Color::Black))
            .block(
                Block::default()
                    .title("Password")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(Color::Black).fg(Color::White)),
            );

        let prompt_width = (max_width - 2) as u16;
        let prompt_height = 3;
        let w = prompt_width.min(area.width);
        let h = prompt_height.min(area.height);

        let x = area.x + (area.width - w) / 2;
        let y = area.y + 2;
        let rect = Rect::new(x, y, w, h);

        (EnContentMenuItem::Paragraph(paragraph), rect)
    }

    fn make_empty_prompt(&self) -> (EnContentMenuItem<'static>, Rect) {
        (
            EnContentMenuItem::Paragraph(
                Paragraph::new("").style(Style::default().bg(Color::Black)),
            ),
            Rect::default(),
        )
    }

    fn make_prompt_lines(&self, max_width: usize) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        let masked_pass = "*".repeat(self.prompt_pass.len());
        let pass_line = format!("{:<width$}", masked_pass, width = max_width);
        lines.push(Line::from(Span::styled(
            pass_line,
            Style::default().bg(Color::Black),
        )));

        let prompt_height = 3;
        while lines.len() < prompt_height as usize {
            lines.push(Line::from(Span::styled(
                " ".repeat(38),
                Style::default().bg(Color::Black),
            )));
        }

        lines
    }

    fn make_wifi_line(&self, ssid: &String, signal: u8, max_width: usize) -> Line<'static> {
        let formatted_ssid = self.format_ssid_string(ssid, max_width.saturating_sub(7));
        let formatted_signal = self.format_signal(signal);

        let ssid_width = formatted_ssid.chars().count();
        let signal_width = formatted_signal.chars().count();

        let space_width = if max_width > ssid_width + signal_width {
            max_width - ssid_width - signal_width
        } else {
            1
        };

        let display = format!(
            "{}{}{}",
            formatted_ssid,
            " ".repeat(space_width),
            formatted_signal
        );

        Line::from(vec![Span::raw(display)])
    }

    fn format_ssid_string(&self, ssid: &String, max_ssid_len: usize) -> String {
        let is_long = ssid.chars().count() > max_ssid_len;
        let is_connected = self.connected_ssid.trim() == ssid.trim();

        if !is_long {
            ssid.clone()
        } else {
            let short: String = ssid.chars().take(max_ssid_len).collect();
            if is_connected {
                format!("* {short}...")
            } else {
                format!("{short}...")
            }
        }
    }

    fn format_signal(&self, signal: u8) -> String {
        match signal {
            0..=33 => "▁▁▁".to_string(),
            34..=67 => "▁▂▃".to_string(),
            _ => "▃▅▇".to_string(),
        }
    }

    // ====== nmcli-related Commands ======

    fn connect_to_wifi(&self, ssid: &str, password: &str) -> String {
        if password == "" {
            return String::new();
        }

        let result = Command::new("nmcli")
            .args(["device", "wifi", "connect", ssid, "password", password])
            .output();

        match result {
            Ok(output) if output.status.success() => format!("Connected to {}", ssid),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                format!("Failed to connect to {}: {}", ssid, stderr.trim())
            }
            Err(e) => format!("Failed to execute nmcli: {}", e),
        }
    }

    fn make_wifi_list() -> Vec<(String, u8)> {
        let output = Command::new("nmcli")
            .args(&["-t", "-f", "SSID,SIGNAL", "dev", "wifi"])
            .output()
            .expect("failed to execute nmcli");

        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut networks: Vec<(String, u8)> = stdout
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, ':');
                let ssid = parts.next()?.trim();
                let signal_str = parts.next()?.trim();
                let signal = signal_str.parse::<u8>().ok()?;
                if ssid.is_empty() {
                    None
                } else {
                    Some((ssid.to_string(), signal))
                }
            })
            .collect();

        networks.sort_by(|a, b| b.1.cmp(&a.1));
        networks.into_iter().take(10).collect()
    }

    fn get_connected_ssid(&self) -> String {
        let output = Command::new("nmcli")
            .args(&["-t", "-f", "active,ssid", "dev", "wifi"])
            .output();

        let output = match output {
            Ok(o) => o,
            Err(_) => return String::new(),
        };

        if !output.status.success() {
            return String::new();
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            let mut parts = line.splitn(2, ':');
            if let (Some(active), Some(ssid)) = (parts.next(), parts.next()) {
                if active == "yes" {
                    return ssid.to_string();
                }
            }
        }

        String::new()
    }
}
