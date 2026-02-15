use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use tui_textarea::TextArea;

use crate::app::ConnectionFocus;
use crate::storage::RecentConnection;
use super::theme::{icons, Theme};

pub fn render_connection_dialog(
    frame: &mut Frame,
    textarea: &TextArea,
    error: Option<&str>,
    recent_connections: &[RecentConnection],
    recent_state: &mut ListState,
    connection_focus: ConnectionFocus,
    theme: &Theme,
) {
    let area = frame.area();

    frame.render_widget(
        Block::default().style(Style::default().bg(theme.bg)),
        area,
    );

    let has_recent = !recent_connections.is_empty();
    let recent_list_height = if has_recent {
        (recent_connections.len() as u16).min(6) + 2
    } else {
        0
    };

    let dialog_width = 80.min(area.width.saturating_sub(4));
    let dialog_height = if has_recent {
        14 + recent_list_height
    } else {
        10
    };

    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(format!(" {} Connect to Database ", icons::DATABASE))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(theme.border_focused_style())
        .style(Style::default().bg(theme.bg_secondary));

    frame.render_widget(block, dialog_area);

    let inner = Rect::new(
        dialog_area.x + 2,
        dialog_area.y + 1,
        dialog_area.width.saturating_sub(4),
        dialog_area.height.saturating_sub(2),
    );

    if has_recent {
        let chunks = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(recent_list_height),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(inner);

        let recent_label_style = if connection_focus == ConnectionFocus::RecentList {
            theme.accent_style().add_modifier(Modifier::BOLD)
        } else {
            theme.dim_style()
        };
        let recent_label = Paragraph::new(Line::from(vec![
            Span::styled(format!("{} ", icons::FOLDER_OPEN), recent_label_style),
            Span::styled("Recent Connections", recent_label_style),
            Span::styled("  (Tab to switch)", theme.muted_style()),
        ]))
        .alignment(Alignment::Left);
        frame.render_widget(recent_label, chunks[0]);

        let list_border_style = if connection_focus == ConnectionFocus::RecentList {
            theme.border_focused_style()
        } else {
            theme.border_style()
        };

        let items: Vec<ListItem> = recent_connections
            .iter()
            .map(|conn| {
                let line = Line::from(vec![
                    Span::styled(format!("{} ", icons::CONNECTION), theme.accent_style()),
                    Span::styled(&conn.display_name, theme.text_style()),
                ]);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(list_border_style)
                    .style(Style::default().bg(theme.bg)),
            )
            .highlight_style(theme.selected_style())
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, chunks[1], recent_state);

        let new_label_style = if connection_focus == ConnectionFocus::NewInput {
            theme.accent_style().add_modifier(Modifier::BOLD)
        } else {
            theme.dim_style()
        };
        let new_label = Paragraph::new(Line::from(vec![
            Span::styled(format!("{} ", icons::DATABASE), new_label_style),
            Span::styled("New Connection", new_label_style),
        ]))
        .alignment(Alignment::Left);
        frame.render_widget(new_label, chunks[2]);

        let hint = Paragraph::new("postgres://  mysql://  sqlite://")
            .style(theme.muted_style())
            .alignment(Alignment::Left);
        frame.render_widget(hint, chunks[3]);

        let input_border_style = if connection_focus == ConnectionFocus::NewInput {
            theme.border_focused_style()
        } else {
            theme.border_style()
        };

        let mut ta = textarea.clone();
        ta.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(input_border_style)
                .style(Style::default().bg(theme.bg)),
        );
        ta.set_style(theme.text_style());
        if connection_focus == ConnectionFocus::NewInput {
            ta.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED).bg(theme.accent));
        } else {
            ta.set_cursor_style(Style::default());
        }
        frame.render_widget(&ta, chunks[4]);

        let status = if let Some(err) = error {
            Paragraph::new(Line::from(vec![
                Span::styled(format!("{} ", icons::CLEAR), theme.error_style()),
                Span::styled(err, theme.error_style()),
            ]))
            .alignment(Alignment::Center)
        } else {
            let help_text = match connection_focus {
                ConnectionFocus::RecentList => "Enter: connect  |  Ctrl+Del: remove  |  Tab: new connection  |  Esc: quit",
                ConnectionFocus::NewInput => "Enter: connect  |  Tab: recent connections  |  Esc: quit",
            };
            Paragraph::new(help_text)
                .style(theme.muted_style())
                .alignment(Alignment::Center)
        };
        frame.render_widget(status, chunks[5]);
    } else {
        let chunks = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(inner);

        let title = Paragraph::new("Enter connection string")
            .style(theme.accent_style().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        let hint = Paragraph::new("postgres://user:pass@host/db  |  mysql://...  |  sqlite://path.db")
            .style(theme.muted_style())
            .alignment(Alignment::Center);
        frame.render_widget(hint, chunks[1]);

        let mut ta = textarea.clone();
        ta.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_focused_style())
                .style(Style::default().bg(theme.bg)),
        );
        ta.set_style(theme.text_style());
        ta.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED).bg(theme.accent));
        frame.render_widget(&ta, chunks[2]);

        let status = if let Some(err) = error {
            Paragraph::new(Line::from(vec![
                Span::styled(format!("{} ", icons::CLEAR), theme.error_style()),
                Span::styled(err, theme.error_style()),
            ]))
            .alignment(Alignment::Center)
        } else {
            Paragraph::new("Press Enter to connect  |  Esc to quit")
                .style(theme.muted_style())
                .alignment(Alignment::Center)
        };
        frame.render_widget(status, chunks[3]);
    }
}
