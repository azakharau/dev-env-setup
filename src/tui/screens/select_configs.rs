use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use super::super::App;
use super::super::layout::{BLUE, CYAN, FG, GREEN, MUTED, help_bar};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(4),
        Constraint::Length(2),
    ])
    .split(area);

    let title = Paragraph::new("Select configs to deploy")
        .style(Style::default().fg(CYAN).bold())
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let items: Vec<_> = app
        .configs
        .iter()
        .enumerate()
        .map(|(i, cfg)| {
            let is_cursor = i == app.cursor;
            let cursor_str = if is_cursor { " > " } else { "   " };
            let checkbox = if cfg.selected { "[x]" } else { "[ ]" };

            let (name_style, path_color) = if is_cursor {
                (Style::default().fg(CYAN).bold(), CYAN)
            } else {
                (Style::default().fg(FG), MUTED)
            };

            let check_color = if cfg.selected {
                GREEN
            } else {
                name_style.fg.unwrap_or(FG)
            };

            ListItem::new(Line::from(vec![
                Span::styled(cursor_str, name_style),
                Span::styled(checkbox, Style::default().fg(check_color)),
                Span::raw(" "),
                Span::styled(cfg.name.as_str(), name_style),
                Span::styled(
                    format!(" - {} -> {}", cfg.description, cfg.target_display),
                    Style::default().fg(path_color),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(BLUE)),
    );

    frame.render_widget(list, chunks[1]);
    frame.render_widget(
        help_bar(&[
            "j/k: move",
            "space: toggle",
            "a: toggle all",
            "Enter: deploy",
            "Esc: back",
            "q: quit",
        ]),
        chunks[2],
    );
}
