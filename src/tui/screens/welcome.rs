use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::super::App;
use super::super::layout::{
    BLUE, CYAN, FG, GREEN, MUTED, PURPLE, SURFACE, YELLOW, draw_logo, help_bar,
};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let local_repo_path = app.config.resolved_configs_path().display().to_string();
    let outer = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);
    let content = inset_rect(outer[0], 8, 72, 26);
    let required_pending = app
        .deps
        .iter()
        .filter(|d| {
            d.required
                && matches!(
                    d.status,
                    super::super::Status::Pending
                        | super::super::Status::InProgress
                        | super::super::Status::Failed
                )
        })
        .count();

    let chunks = Layout::vertical([
        Constraint::Length(6),
        Constraint::Length(1),
        Constraint::Length(9),
        Constraint::Length(1),
        Constraint::Length(9),
        Constraint::Min(0),
    ])
    .split(content);

    draw_logo(frame, chunks[0]);

    let top_cols = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Length(2),
        Constraint::Percentage(50),
    ])
    .split(chunks[2]);

    let left = vec![
        info_line("Config", &app.config_path, CYAN),
        info_line("Repo", &app.config.configs_repo, PURPLE),
        info_line("Local", &local_repo_path, FG),
        info_line("OS", app.os.display_name(), FG),
        info_line(
            "Package",
            app.installer.map_or("not detected", |i| i.as_config_str()),
            YELLOW,
        ),
    ];

    let right = vec![
        status_line("Git", app.git_available),
        status_line("Package manager", app.package_manager_available),
        status_line("SSH key", app.ssh_key_available),
        status_line("Repo link", !app.config.configs_repo.trim().is_empty()),
        status_line("Local repo", app.local_configs_repo_ready),
    ];

    draw_panel(
        frame,
        top_cols[0],
        " current context ",
        CYAN,
        left,
        Alignment::Left,
    );
    draw_panel(
        frame,
        top_cols[2],
        " readiness ",
        PURPLE,
        right,
        Alignment::Left,
    );

    let bottom_cols = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Length(2),
        Constraint::Percentage(50),
    ])
    .split(chunks[4]);

    let required_summary = required_summary(app, required_pending == 0);
    let next_steps = vec![
        Line::from(vec![
            Span::styled("1.", Style::default().fg(CYAN).bold()),
            Span::raw(if required_pending == 0 {
                "  Required tooling already ready"
            } else {
                "  Install missing required tooling"
            }),
        ]),
        Line::from(vec![
            Span::styled("2.", Style::default().fg(CYAN).bold()),
            Span::raw("  Review optional tools"),
        ]),
        Line::from(vec![
            Span::styled("3.", Style::default().fg(CYAN).bold()),
            Span::raw("  Review dotfile targets"),
        ]),
        Line::from(vec![
            Span::styled("4.", Style::default().fg(CYAN).bold()),
            Span::raw("  Sync repo and create symlinks"),
        ]),
        Line::from(vec![
            Span::styled("Expect ", Style::default().fg(YELLOW).bold()),
            Span::raw("sudo on Linux, network for repo sync, no linking before selection."),
        ]),
    ];

    draw_panel(
        frame,
        bottom_cols[0],
        if required_pending == 0 {
            " required tooling status "
        } else {
            " what still needs setup "
        },
        BLUE,
        required_summary,
        Alignment::Left,
    );
    draw_panel(
        frame,
        bottom_cols[2],
        " next steps ",
        GREEN,
        next_steps,
        Alignment::Left,
    );

    frame.render_widget(help_bar(&["Enter: continue", "q: quit"]), outer[1]);
}

fn inset_rect(area: Rect, desired: u16, min_width: u16, min_height: u16) -> Rect {
    let max_pad_x = area.width.saturating_sub(min_width) / 2;
    let max_pad_y = area.height.saturating_sub(min_height) / 2;
    let pad = desired.min(max_pad_x).min(max_pad_y);

    Rect::new(
        area.x + pad,
        area.y + pad,
        area.width.saturating_sub(pad * 2),
        area.height.saturating_sub(pad * 2),
    )
}

fn draw_panel<'a>(
    frame: &mut Frame,
    area: Rect,
    title: &'a str,
    title_color: Color,
    lines: Vec<Line<'a>>,
    alignment: Alignment,
) {
    let block = panel_block(title, title_color);
    let inner = block.inner(area);

    frame.render_widget(block, area);

    let body = body_rect(inner, 2, 1);
    let paragraph = Paragraph::new(lines)
        .alignment(alignment)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, body);
}

fn body_rect(area: Rect, horizontal_pad: u16, top_pad: u16) -> Rect {
    let width = area.width.saturating_sub(horizontal_pad * 2).max(1);
    let height = area.height.saturating_sub(top_pad).max(1);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + top_pad.min(area.height.saturating_sub(1));

    Rect::new(x, y, width, height)
}

fn panel_block<'a>(title: &'a str, title_color: Color) -> Block<'a> {
    Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .title_style(Style::default().fg(title_color).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BLUE))
        .style(Style::default().bg(SURFACE))
}

fn info_line<'a>(label: &'a str, value: &'a str, color: Color) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{label:<11}"), Style::default().fg(MUTED)),
        Span::styled(value, Style::default().fg(color).bold()),
    ])
}

fn status_line<'a>(label: &'a str, ok: bool) -> Line<'a> {
    let (icon, color, text) = if ok {
        ("✓", GREEN, "ready")
    } else {
        ("!", YELLOW, "check")
    };

    Line::from(vec![
        Span::styled(icon, Style::default().fg(color).bold()),
        Span::raw("  "),
        Span::styled(format!("{label:<18}"), Style::default().fg(FG)),
        Span::raw("  "),
        Span::styled(text, Style::default().fg(color).bold()),
    ])
}

fn required_summary(app: &App, all_ready: bool) -> Vec<Line<'_>> {
    let mut by_category: std::collections::BTreeMap<&str, Vec<&crate::tui::DepItem>> =
        std::collections::BTreeMap::new();

    for dep in app.deps.iter().filter(|d| {
        d.required
            && if all_ready {
                true
            } else {
                matches!(
                    d.status,
                    super::super::Status::Pending
                        | super::super::Status::InProgress
                        | super::super::Status::Failed
                )
            }
    }) {
        by_category
            .entry(dep.category.as_str())
            .or_default()
            .push(dep);
    }

    let mut lines = Vec::new();
    for (category, deps) in by_category {
        let examples_vec = deps
            .iter()
            .take(2)
            .map(|d| d.name.as_str())
            .collect::<Vec<_>>();
        let examples = examples_vec.join(", ");
        let more = deps.len().saturating_sub(examples_vec.len());

        lines.push(Line::from(vec![
            Span::styled("• ", Style::default().fg(CYAN).bold()),
            Span::styled(category.to_uppercase(), Style::default().fg(PURPLE).bold()),
            Span::raw("  "),
            Span::styled(
                format!("({})", deps.len()),
                Style::default().fg(YELLOW).bold(),
            ),
        ]));

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(examples, Style::default().fg(FG)),
            if more > 0 {
                Span::styled(format!("  +{more} more"), Style::default().fg(MUTED))
            } else {
                Span::raw("")
            },
        ]));

        let why = if category == "core" {
            if all_ready {
                "already available; optional tools and config deployment can proceed"
            } else {
                "required before optional tools and config deployment"
            }
        } else {
            deps.first()
                .and_then(|dep| (!dep.description.is_empty()).then_some(dep.description.as_str()))
                .unwrap_or("required by current setup plan")
        };

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(why, Style::default().fg(MUTED)),
        ]));

        lines.push(Line::from(""));
    }

    if lines.pop().is_none() {
        lines.push(Line::from(Span::styled(
            if all_ready {
                "All required tooling is already ready on this machine."
            } else {
                "No required tooling remains to be installed."
            },
            Style::default().fg(GREEN),
        )));
    }

    lines
}
