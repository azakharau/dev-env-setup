use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use super::super::layout::{BLUE, CYAN, FG, GREEN, MUTED, PURPLE, help_bar};
use super::super::{App, Status};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(4),
        Constraint::Length(2),
    ])
    .split(area);

    let title = Paragraph::new("Select optional dependencies to install")
        .style(Style::default().fg(CYAN).bold())
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let opt_indices = app.optional_indices();
    let mut items = Vec::with_capacity(opt_indices.len() * 2);
    let mut last_category: Option<&str> = None;

    for (list_pos, &dep_idx) in opt_indices.iter().enumerate() {
        let dep = &app.deps[dep_idx];

        if last_category != Some(dep.category.as_str()) {
            if last_category.is_some() {
                items.push(ListItem::new(Line::from("")));
            }
            items.push(ListItem::new(Line::from(Span::styled(
                format!("  {} ", dep.category.to_uppercase()),
                Style::default().fg(PURPLE).bold(),
            ))));
            last_category = Some(dep.category.as_str());
        }

        let is_cursor = list_pos == app.cursor;
        let checkbox = match (dep.status, dep.selected) {
            (Status::AlreadyDone, _) => "[~]",
            (_, true) => "[x]",
            _ => "[ ]",
        };

        let status_hint = if dep.status == Status::AlreadyDone {
            " (installed)"
        } else {
            ""
        };

        let (name_style, desc_color) = if is_cursor {
            (Style::default().fg(CYAN).bold(), CYAN)
        } else if dep.status == Status::AlreadyDone {
            (Style::default().fg(GREEN), MUTED)
        } else {
            (Style::default().fg(FG), MUTED)
        };

        let cursor_str = if is_cursor { " > " } else { "   " };
        let check_color = if dep.selected && dep.status != Status::AlreadyDone {
            GREEN
        } else {
            name_style.fg.unwrap_or(FG)
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(cursor_str, name_style),
            Span::styled(checkbox, Style::default().fg(check_color)),
            Span::raw(" "),
            Span::styled(dep.name.as_str(), name_style),
            Span::styled(
                format!(" - {}{}", dep.description, status_hint),
                Style::default().fg(desc_color),
            ),
        ])));
    }

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
            "Enter: install",
            "Esc: back",
            "q: quit",
        ]),
        chunks[2],
    );
}
