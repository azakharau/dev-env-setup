use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem};

use super::super::layout::{BLUE, CYAN, FG, GREEN, RED, SURFACE_HI, help_bar};
use super::super::{App, LogLevel, Screen};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(4),
        Constraint::Length(2),
    ])
    .split(area);

    let title = match app.screen {
        Screen::InstallingRequired => "Installing Required Dependencies",
        Screen::InstallingSelected => "Installing Selected Dependencies",
        Screen::DeployingConfigs => "Deploying Configs",
        _ => "Working",
    };

    let ratio = if app.progress_total == 0 {
        0.0
    } else {
        app.progress_current as f64 / app.progress_total as f64
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(title)
                .title_style(Style::default().fg(CYAN).bold())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BLUE)),
        )
        .gauge_style(Style::default().fg(CYAN).bg(SURFACE_HI))
        .ratio(ratio.min(1.0))
        .label(format!("{}/{}", app.progress_current, app.progress_total));
    frame.render_widget(gauge, chunks[0]);

    let items = app
        .log
        .iter()
        .rev()
        .take(chunks[1].height as usize)
        .rev()
        .map(|entry| {
            let color = match entry.level {
                LogLevel::Info => FG,
                LogLevel::Success => GREEN,
                LogLevel::Error => RED,
            };

            ListItem::new(Line::from(Span::styled(
                &entry.message,
                Style::default().fg(color),
            )))
        })
        .collect::<Vec<_>>();

    let logs = List::new(items).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(BLUE)),
    );

    frame.render_widget(logs, chunks[1]);
    frame.render_widget(help_bar(&["q: quit"]), chunks[2]);
}
