use chrono::{Local, Timelike};
use ratatui::{
    layout::Rect, text::{Line, Span, Text}, widgets::{Block, Borders, Paragraph}, Frame
};

pub struct ClockWidget;

impl ClockWidget {
    pub fn render(frame: &mut Frame, area: Rect) {
        let area = area;

        let hour = Local::now().hour();
        let minute = Local::now().minute();

        let hour_text = 
            Self::get_number_text(&[
                hour / 10, 
                hour % 10, 
                10, 
                minute / 10, 
                minute % 10]
            );

        let width = (hour_text
            .lines
            .iter()
            .map(|line| line.width())
            .max()
            .unwrap_or(0) - 1) as u16;

        let height = 6 as u16;

        let block = 
            Block::default()
                .borders(Borders::BOTTOM);

        let content_area = Rect {
            x: area.x + (area.width.saturating_sub(width)) / 2,
            y: 0,
            width,
            height,
        };

        let paragraph = Paragraph::new(hour_text).block(block).centered();

        frame.render_widget(paragraph, content_area);
    }

    fn get_number_text(digits: &[u32]) -> Text<'static> {
        let mut text = Text::default();

        for line_number in 0..5 {
            let mut line = Line::default();
            for &digit in digits {
                if digit > 11 {
                    continue;
                }
                line.push_span(Span::from(ASCII_DIGITS[digit as usize][line_number]));
                line.push_span(Span::from(" "));
            }
            text.lines.push(line);
        }

        text
    }
}


const ASCII_DIGITS: [[&str; 5]; 11] = [
    [ // 0
        "██████",
        "██  ██",
        "██  ██",
        "██  ██",
        "██████",
    ],
    [ // 1
        "  ██  ",
        "████  ",
        "  ██  ",
        "  ██  ",
        "██████",
    ],
    [ // 2
        "██████",
        "    ██",
        "██████",
        "██    ",
        "██████",
    ],
    [ // 3
        "██████",
        "    ██",
        "██████",
        "    ██",
        "██████",
    ],
    [ // 4
        "██  ██",
        "██  ██",
        "██████",
        "    ██",
        "    ██",
    ],
    [ // 5
        "██████",
        "██    ",
        "██████",
        "    ██",
        "██████",
    ],
    [ // 6
        "██████",
        "██    ",
        "██████",
        "██  ██",
        "██████",
    ],
    [ // 7
        "██████",
        "    ██",
        "   ██ ",
        "  ██  ",
        " ██   ",
    ],
    [ // 8
        "██████",
        "██  ██",
        "██████",
        "██  ██",
        "██████",
    ],
    [ // 9
        "██████",
        "██  ██",
        "██████",
        "    ██",
        "██████",
    ],
    [ // :
        "    ",
        " ██ ",
        "    ",
        " ██ ",
        "    ",
    ]
];
