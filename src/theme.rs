#![allow(dead_code)]
use std::time::Duration;

// TODO: load in runtime
#[derive(Debug)]
pub struct Theme {
    font: &'static str,
    bg_tabs: (u8, u8, u8),
    bg_widgets: (u8, u8, u8),
    fg_widgets: (u8, u8, u8),
    selection_style: SelectionStyle,
    padding: u8,
    idle_timeout: Duration,
}

#[derive(Debug)]
pub enum SelectionStyle {
    Outline(u8, u8, u8),
    BackgroundDiff,
    TextHighlight(u8, u8, u8),
}

impl Theme {
    pub fn font() -> &'static str {
        "/usr/share/fonts/liberation/LiberationSans-Regular.ttf"
    }
    pub fn bg_tabs() -> (u8, u8, u8) {
        (50, 50, 50)
    }
    pub fn bg_widgets() -> (u8, u8, u8) {
        (30, 30, 30)
    }
    pub fn fg_widgets() -> (u8, u8, u8) {
        (250, 250, 250)
    }
    pub fn selection_style() -> SelectionStyle {
        SelectionStyle::Outline(200, 200, 200)
    }
    pub fn padding() -> u8 {
        20
    }
    pub fn idle_timeout() -> Duration {
        Duration::from_secs(2)
    }
}
