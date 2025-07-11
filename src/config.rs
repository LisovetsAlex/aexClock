use color_eyre::eyre::{Result, eyre};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use once_cell::sync::OnceCell;
use ratatui::style::Color;
use ratatui::widgets::BorderType;
use serde::Deserialize;
use std::convert::TryFrom;
use std::fs;

#[derive(Debug)]
pub struct Config {
    pub themes: Theme,
    pub keybinds: Keybinds,
}

#[derive(Debug)]
pub struct Theme {
    pub borders_on: bool,
    pub border_color: Color,
    pub border_type: BorderType,
    pub nav_selected_fg_color: Color,
    pub nav_selected_bg_color: Color,
    pub content_selected_color: Color,
    pub bg_color: Color,
    pub fg_color: Color,
    pub scroll_color: Color,
    pub bar_side_color: Color,
    pub bar_filled_color: Color,
    pub bar_empty_color: Color,
    pub bar_selected_side_color: Color,
    pub bar_selected_filled_color: Color,
    pub bar_selected_empty_color: Color,
}

#[derive(Debug, Deserialize)]
pub struct RawThemes {
    pub borders_on: bool,
    pub border_color: String,
    pub border_style: String,
    pub nav_selected_fg_color: String,
    pub nav_selected_bg_color: String,
    pub content_selected_color: String,
    pub bg_color: String,
    pub fg_color: String,
    pub scroll_color: String,
    pub bar_side_color: String,
    pub bar_filled_color: String,
    pub bar_empty_color: String,
    pub bar_selected_side_color: String,
    pub bar_selected_filled_color: String,
    pub bar_selected_empty_color: String,
}

impl TryFrom<RawThemes> for Theme {
    type Error = color_eyre::eyre::Report;

    fn try_from(raw: RawThemes) -> Result<Self> {
        Ok(Self {
            border_color: parse_color(&raw.border_color)?,
            border_type: parse_border(&raw.border_style)?,
            nav_selected_fg_color: parse_color(&raw.nav_selected_fg_color)?,
            nav_selected_bg_color: parse_color(&raw.nav_selected_bg_color)?,
            content_selected_color: parse_color(&raw.content_selected_color)?,
            bg_color: parse_color(&raw.bg_color)?,
            fg_color: parse_color(&raw.fg_color)?,
            scroll_color: parse_color(&raw.scroll_color)?,
            borders_on: raw.borders_on,
            bar_side_color: parse_color(&raw.bar_side_color)?,
            bar_filled_color: parse_color(&raw.bar_filled_color)?,
            bar_empty_color: parse_color(&raw.bar_empty_color)?,
            bar_selected_side_color: parse_color(&raw.bar_selected_side_color)?,
            bar_selected_filled_color: parse_color(&raw.bar_selected_filled_color)?,
            bar_selected_empty_color: parse_color(&raw.bar_selected_empty_color)?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Keybinds {
    pub nav_up: String,
    pub nav_down: String,
    pub content_up: String,
    pub content_down: String,
    pub content_right: String,
    pub content_left: String,
    pub accept: String,
    pub info: String,
    pub cancel: String,
    pub quit: String,
}

// RawConfig mirrors the toml, to parse before converting themes to strong types
#[derive(Debug, Deserialize)]
pub struct RawConfig {
    pub themes: RawThemes,
    pub keybinds: Keybinds,
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut config_path =
            dirs::config_dir().ok_or_else(|| eyre!("Could not find config directory"))?;
        config_path.push("aex");
        config_path.push("clock.toml");

        let config_str = fs::read_to_string(&config_path)?;
        let raw: RawConfig = toml::from_str(&config_str)?;
        Ok(Config {
            themes: raw.themes.try_into()?,
            keybinds: raw.keybinds,
        })
    }

    /// Checks if a KeyEvent matches the keybind string (e.g. "shift+w", "enter")
    pub fn key_matches(&self, key_event: &KeyEvent, keybind_str: &str) -> bool {
        let parts = keybind_str.split('+');
        let mut required_modifiers = KeyModifiers::empty();
        let mut keycode: Option<KeyCode> = None;

        for part in parts {
            let part_lower = part.trim().to_lowercase();
            match part_lower.as_str() {
                "shift" => required_modifiers |= KeyModifiers::SHIFT,
                "ctrl" | "control" => required_modifiers |= KeyModifiers::CONTROL,
                "alt" => required_modifiers |= KeyModifiers::ALT,
                "up" => keycode = Some(KeyCode::Up),
                "down" => keycode = Some(KeyCode::Down),
                "left" => keycode = Some(KeyCode::Left),
                "right" => keycode = Some(KeyCode::Right),
                "enter" => keycode = Some(KeyCode::Enter),
                "esc" | "escape" => keycode = Some(KeyCode::Esc),
                "tab" => keycode = Some(KeyCode::Tab),
                "backspace" => keycode = Some(KeyCode::Backspace),
                "space" => keycode = Some(KeyCode::Char(' ')),
                s if s.len() == 1 => {
                    let ch = s.chars().next().unwrap();
                    let ch = if required_modifiers.contains(KeyModifiers::SHIFT) {
                        ch.to_ascii_uppercase()
                    } else {
                        ch.to_ascii_lowercase()
                    };
                    keycode = Some(KeyCode::Char(ch));
                }
                _ => {}
            }
        }

        match keycode {
            Some(code) => key_event.code == code && key_event.modifiers == required_modifiers,
            None => false,
        }
    }
}

static CONFIG_CELL: OnceCell<Config> = OnceCell::new();

pub fn init_config() -> Result<()> {
    let config = Config::load()?;
    CONFIG_CELL
        .set(config)
        .map_err(|_| eyre!("Config already initialized"))?;
    Ok(())
}

pub fn CONFIG() -> &'static Config {
    CONFIG_CELL.get().expect("Config not initialized")
}

// --- Helper parsers ---

fn parse_color(s: &str) -> Result<Color> {
    match s.to_lowercase().as_str() {
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "magenta" => Ok(Color::Magenta),
        "cyan" => Ok(Color::Cyan),
        "gray" => Ok(Color::Gray),
        "darkgray" => Ok(Color::DarkGray),
        "white" => Ok(Color::White),
        s if s.starts_with('#') && s.len() == 7 => {
            let r = u8::from_str_radix(&s[1..3], 16)?;
            let g = u8::from_str_radix(&s[3..5], 16)?;
            let b = u8::from_str_radix(&s[5..7], 16)?;
            Ok(Color::Rgb(r, g, b))
        }
        _ => Err(eyre!("Invalid color: {}", s)),
    }
}

fn parse_border(s: &str) -> Result<BorderType> {
    match s.to_lowercase().as_str() {
        "plain" => Ok(BorderType::Plain),
        "rounded" => Ok(BorderType::Rounded),
        "double" => Ok(BorderType::Double),
        "thick" => Ok(BorderType::Thick),
        _ => Err(eyre!("Invalid border type: {}", s)),
    }
}
