use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Wrap};

use super::super::App;
use super::super::layout::{CYAN, FG, MUTED, PURPLE, help_bar, status_icon, status_label};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(4),
        Constraint::Length(2),
    ])
    .split(area);

    let title = Paragraph::new("Summary")
        .style(Style::default().fg(CYAN).bold())
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let mut lines = vec![Line::from(Span::styled(
        "  Dependencies:",
        Style::default().fg(PURPLE).bold(),
    ))];

    for dep in &app.deps {
        if dep.required || dep.selected {
            let (icon, color) = status_icon(dep.status);
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(icon, Style::default().fg(color)),
                Span::raw(" "),
                Span::styled(dep.name.as_str(), Style::default().fg(FG)),
                Span::styled(
                    format!(" ({})", status_label(dep.status)),
                    Style::default().fg(MUTED),
                ),
            ]));
        }
    }

    if !app.configs.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Configs:",
            Style::default().fg(PURPLE).bold(),
        )));

        for cfg in &app.configs {
            if cfg.selected {
                let (icon, color) = status_icon(cfg.status);
                lines.push(Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled(icon, Style::default().fg(color)),
                    Span::raw(" "),
                    Span::styled(cfg.name.as_str(), Style::default().fg(FG)),
                    Span::styled(
                        format!(" -> {}", cfg.target_display),
                        Style::default().fg(MUTED),
                    ),
                ]));
            }
        }
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), chunks[1]);
    frame.render_widget(help_bar(&["Enter/q: exit"]), chunks[2]);
}
