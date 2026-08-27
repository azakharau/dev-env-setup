use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use super::{App, Screen, Status, screens};

pub(crate) const BG: Color = Color::Rgb(26, 27, 38);
pub(crate) const SURFACE: Color = Color::Rgb(36, 40, 59);
pub(crate) const SURFACE_HI: Color = Color::Rgb(41, 46, 66);
pub(crate) const FG: Color = Color::Rgb(192, 202, 245);
pub(crate) const MUTED: Color = Color::Rgb(169, 177, 214);
pub(crate) const BLUE: Color = Color::Rgb(122, 162, 247);
pub(crate) const CYAN: Color = Color::Rgb(125, 207, 255);
pub(crate) const PURPLE: Color = Color::Rgb(187, 154, 247);
pub(crate) const GREEN: Color = Color::Rgb(158, 206, 106);
pub(crate) const YELLOW: Color = Color::Rgb(224, 175, 104);
pub(crate) const RED: Color = Color::Rgb(247, 118, 142);

pub(crate) const LOGO: [&str; 6] = [
    "  ____             _____                    ",
    " |  _ \\  _____   _|  ___|__  _ __ __ _  ___ ",
    " | | | |/ _ \\ \\ / / |_ / _ \\| '__/ _` |/ _ \\",
    " | |_| |  __/\\ V /|  _| (_) | | | (_| |  __/",
    " |____/ \\___| \\_/ |_|  \\___/|_|  \\__, |\\___|",
    "                                  |___/      ",
];

pub(crate) fn draw(frame: &mut Frame, app: &App) {
    frame.render_widget(Clear, frame.area());
    frame.render_widget(
        Block::default().style(Style::default().bg(BG)),
        frame.area(),
    );

    let outer = Rect::new(
        frame.area().x,
        frame.area().y,
        frame.area().width,
        frame.area().height,
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BLUE));
    let inner = block.inner(outer);

    frame.render_widget(block, outer);

    match app.screen {
        Screen::Welcome => screens::welcome::draw(frame, inner, app),
        Screen::InstallingRequired => screens::progress::draw(frame, inner, app),
        Screen::SelectDeps => screens::select_deps::draw(frame, inner, app),
        Screen::InstallingSelected => screens::progress::draw(frame, inner, app),
        Screen::SelectConfigs => screens::select_configs::draw(frame, inner, app),
        Screen::DeployingConfigs => screens::progress::draw(frame, inner, app),
        Screen::Summary => screens::summary::draw(frame, inner, app),
    }
}

pub(crate) fn centered_rect(max_w: u16, max_h: u16, area: Rect) -> Rect {
    let width = max_w.min(area.width);
    let height = max_h.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

pub(crate) fn help_bar<'a>(items: &[&'a str]) -> Paragraph<'a> {
    let spans: Vec<Span<'a>> = items
        .iter()
        .enumerate()
        .flat_map(|(index, item)| {
            let mut spans = vec![Span::styled(*item, Style::default().fg(FG).bold())];
            if index + 1 < items.len() {
                spans.push(Span::styled("  |  ", Style::default().fg(BLUE)));
            }
            spans
        })
        .collect();

    Paragraph::new(Line::from(spans)).alignment(Alignment::Center)
}

pub(crate) fn draw_logo(frame: &mut Frame, area: Rect) {
    let logo_area = centered_rect(46, 6, area);
    let lines = LOGO
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(CYAN).bold())))
        .collect::<Vec<_>>();

    frame.render_widget(Paragraph::new(lines), logo_area);
}

pub(crate) fn status_icon(status: Status) -> (&'static str, Color) {
    match status {
        Status::Done | Status::AlreadyDone => ("✓", GREEN),
        Status::Failed => ("✗", RED),
        Status::InProgress => ("…", YELLOW),
        Status::Pending => ("·", BLUE),
    }
}

pub(crate) fn status_label(status: Status) -> &'static str {
    match status {
        Status::Done => "installed",
        Status::AlreadyDone => "already installed",
        Status::Failed => "failed",
        Status::InProgress => "in progress",
        Status::Pending => "pending",
    }
}
