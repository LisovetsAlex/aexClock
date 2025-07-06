use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, Padding, Paragraph},
};
use std::{
    process::Command,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
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
    show_info: bool,
    connection_info: Vec<String>,
    scroll_offset: usize,
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
            show_info: false,
            connection_info: Vec::new(),
            scroll_offset: 0,
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
        } else if self.show_info {
            self.make_info_overlay(max_width, area)
        } else {
            self.make_empty_prompt()
        };

        WiMenuItem {
            content: EnContentMenuItem::List(list),
            overlay,
            overlay_area,
            show_overlay: self.show_prompt || self.show_info,
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

        std::thread::spawn(move || {
            loop {
                let new_list = NetConnect::make_wifi_list();

                if let Ok(mut nc) = this.lock() {
                    nc.connected_ssid = nc.get_connected_ssid();
                    nc.connection_info = nc.get_connection_info();
                    nc.wifi_list = new_list;
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

        if self.show_info {
            match key_event.code {
                KeyCode::Up => {
                    if !key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        return;
                    }
                    if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                    }
                }
                KeyCode::Down => {
                    if !key_event.modifiers.contains(KeyModifiers::SHIFT) {
                        return;
                    }
                    let max_offset = self.connection_info.len().saturating_sub(15);
                    if self.scroll_offset < max_offset {
                        self.scroll_offset += 1;
                    }
                }
                KeyCode::Tab | KeyCode::Enter | KeyCode::Esc => {
                    self.show_info = false;
                    self.scroll_offset = 0;
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
                if self.show_info {
                    return;
                }
                self.show_prompt = true;
                self.prompt_ssid = self
                    .wifi_list
                    .get(self.selected_ssid)
                    .map(|(ssid, _)| ssid.clone())
                    .unwrap_or_default();
                self.prompt_pass.clear();
            }
            KeyCode::Tab => {
                if self.show_prompt {
                    return;
                }
                self.show_info = true;
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
        let y = area.y + 3;
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

    fn make_info_overlay(
        &self,
        max_width: usize,
        area: Rect,
    ) -> (EnContentMenuItem<'static>, Rect) {
        let height = 15;
        let total_lines = self.connection_info.len();
        let content_height = height - 3;

        let mut lines: Vec<Line> = self
            .connection_info
            .iter()
            .skip(self.scroll_offset)
            .take(content_height)
            .map(|s| {
                let mut padded = s.clone();
                let len = unicode_width::UnicodeWidthStr::width(padded.as_str());
                if len < max_width {
                    padded.push_str(&" ".repeat(max_width - len + 8));
                }

                let style = if padded.contains(" :  ") {
                    Style::default()
                        .bg(Color::Black)
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().bg(Color::Black).fg(Color::White)
                };

                Line::from(Span::styled(padded, style))
            })
            .collect();

        let scroll_line_text = if total_lines > content_height {
            format!(
                "Scroll: {}/{}",
                (self.scroll_offset + content_height).min(total_lines),
                total_lines - 3
            )
        } else {
            "Scroll: 1/1".to_string()
        };

        let padded_scroll_line = {
            let content_width = max_width;
            let len = unicode_width::UnicodeWidthStr::width(scroll_line_text.as_str());
            if len < content_width {
                let total_padding = content_width - len;
                let left_padding = total_padding / 2;
                let right_padding = total_padding - left_padding;
                format!(
                    "{}{}{}",
                    " ".repeat(left_padding),
                    scroll_line_text,
                    " ".repeat(right_padding)
                )
            } else {
                scroll_line_text
            }
        };

        lines.push(Line::from(Span::styled(
            padded_scroll_line,
            Style::default()
                .bg(Color::Black)
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        )));

        let paragraph = Paragraph::new(lines)
            .style(Style::default().bg(Color::Black).fg(Color::White))
            .block(
                Block::default()
                    .padding(Padding {
                        left: 1,
                        right: 1,
                        top: 0,
                        bottom: 0,
                    })
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );

        let w = (max_width + 4) as u16;
        let h = height as u16;

        let x = area.x + (area.width - w) / 2;
        let y = area.y;

        (
            EnContentMenuItem::Paragraph(paragraph),
            Rect::new(x, y, w, h),
        )
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

    fn get_connection_info(&self) -> Vec<String> {
        let output = Command::new("nmcli").args(&["device", "show"]).output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);

                let field_map = [
                    ("GENERAL.DEVICE", "Device"),
                    ("GENERAL.TYPE", "Type"),
                    ("GENERAL.CONNECTION", "Name"),
                    ("IP4.ADDRESS", "IPv4"),
                    ("IP4.GATEWAY", "IPv4 Gateway"),
                    ("IP4.DNS", "DNS"),
                ];

                let mut devices = Vec::new();
                let mut current_block = Vec::new();

                for line in stdout.lines() {
                    if line.starts_with("GENERAL.DEVICE") && !current_block.is_empty() {
                        devices.push(current_block.clone());
                        current_block.clear();
                    }
                    current_block.push(line);
                }
                if !current_block.is_empty() {
                    devices.push(current_block);
                }

                let mut results = Vec::new();

                for device in devices {
                    let mut device_type = "";
                    let mut device_name = "";

                    for line in &device {
                        if line.starts_with("GENERAL.TYPE:") {
                            device_type = line.splitn(2, ':').nth(1).unwrap_or("").trim();
                        }
                        if line.starts_with("GENERAL.DEVICE:") {
                            device_name = line.splitn(2, ':').nth(1).unwrap_or("").trim();
                        }
                    }

                    if device_type != "wifi" && device_type != "ethernet" {
                        continue;
                    }

                    results.push(format!("===> {} ({})", device_name, device_type));
                    results.push("⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽".to_string());

                    for (nmcli_field, label) in &field_map {
                        for line in &device {
                            if line.starts_with(nmcli_field) {
                                let parts: Vec<&str> = line.splitn(2, ':').collect();
                                if parts.len() == 2 {
                                    let value = parts[1].trim();
                                    results.push(format!("{} :  ", label));
                                    results.push(value.to_string());
                                    results.push("⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽⎽".to_string());
                                }
                            }
                        }
                    }

                    results.push(String::new());
                }

                results.push(String::new());
                results.push(String::new());

                results
            }
            _ => vec!["Failed to get connection info.".to_string()],
        }
    }
}
