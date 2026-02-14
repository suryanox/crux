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

use super::{BORDER_COLOR, DIM_COLOR, HIGHLIGHT_COLOR, TEXT_COLOR};

pub fn render_connection_dialog(
    frame: &mut Frame,
    textarea: &TextArea,
    error: Option<&str>,
    recent_connections: &[RecentConnection],
    recent_state: &mut ListState,
    connection_focus: ConnectionFocus,
) {
    let area = frame.area();
    
    let has_recent = !recent_connections.is_empty();
    let recent_list_height = if has_recent {
        (recent_connections.len() as u16).min(5) + 2
    } else {
        0
    };
    
    let dialog_width = 70.min(area.width.saturating_sub(4));
    let dialog_height = if has_recent {
        11 + recent_list_height
    } else {
        7
    };

    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Connect to Database ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(HIGHLIGHT_COLOR));

    frame.render_widget(block, dialog_area);

    let inner = Rect::new(
        dialog_area.x + 1,
        dialog_area.y + 1,
        dialog_area.width.saturating_sub(2),
        dialog_area.height.saturating_sub(2),
    );

    if has_recent {
        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(recent_list_height),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(inner);

        let recent_label = Paragraph::new("Recent Connections (Tab to switch)")
            .style(Style::default().fg(if connection_focus == ConnectionFocus::RecentList {
                HIGHLIGHT_COLOR
            } else {
                DIM_COLOR
            }))
            .alignment(Alignment::Left);
        frame.render_widget(recent_label, chunks[0]);

        let list_border_color = if connection_focus == ConnectionFocus::RecentList {
            HIGHLIGHT_COLOR
        } else {
            BORDER_COLOR
        };
        
        let items: Vec<ListItem> = recent_connections
            .iter()
            .map(|conn| {
                let line = Line::from(vec![
                    Span::styled(&conn.display_name, Style::default().fg(TEXT_COLOR)),
                    Span::styled(
                        format!("  ({})", format_relative_time(&conn.last_used)),
                        Style::default().fg(DIM_COLOR),
                    ),
                ]);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(list_border_color)),
            )
            .highlight_style(
                Style::default()
                    .bg(HIGHLIGHT_COLOR)
                    .fg(ratatui::style::Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, chunks[1], recent_state);

        let new_label = Paragraph::new("New Connection")
            .style(Style::default().fg(if connection_focus == ConnectionFocus::NewInput {
                HIGHLIGHT_COLOR
            } else {
                DIM_COLOR
            }))
            .alignment(Alignment::Left);
        frame.render_widget(new_label, chunks[2]);

        let hint = Paragraph::new("postgres://, mysql://, or sqlite://")
            .style(Style::default().fg(DIM_COLOR))
            .alignment(Alignment::Center);
        frame.render_widget(hint, chunks[3]);

        let input_border_color = if connection_focus == ConnectionFocus::NewInput {
            HIGHLIGHT_COLOR
        } else {
            BORDER_COLOR
        };
        
        let mut ta = textarea.clone();
        ta.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(input_border_color)),
        );
        ta.set_style(Style::default().fg(TEXT_COLOR));
        if connection_focus == ConnectionFocus::NewInput {
            ta.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        } else {
            ta.set_cursor_style(Style::default());
        }
        frame.render_widget(&ta, chunks[4]);

        let status = if let Some(err) = error {
            Paragraph::new(err)
                .style(Style::default().fg(ratatui::style::Color::Red))
                .alignment(Alignment::Center)
        } else {
            let help_text = match connection_focus {
                ConnectionFocus::RecentList => "Enter: connect, Del: remove, Tab: new connection",
                ConnectionFocus::NewInput => "Enter: connect, Tab: recent connections, Esc: quit",
            };
            Paragraph::new(help_text)
                .style(Style::default().fg(DIM_COLOR))
                .alignment(Alignment::Center)
        };
        frame.render_widget(status, chunks[5]);
    } else {
        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(inner);

        let hint = Paragraph::new("postgres://, mysql://, or sqlite://")
            .style(Style::default().fg(DIM_COLOR))
            .alignment(Alignment::Center);
        frame.render_widget(hint, chunks[0]);

        let mut ta = textarea.clone();
        ta.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(HIGHLIGHT_COLOR)),
        );
        ta.set_style(Style::default().fg(TEXT_COLOR));
        ta.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        frame.render_widget(&ta, chunks[1]);

        let status = if let Some(err) = error {
            Paragraph::new(err)
                .style(Style::default().fg(ratatui::style::Color::Red))
                .alignment(Alignment::Center)
        } else {
            Paragraph::new("Press Enter to connect, Esc to quit")
                .style(Style::default().fg(DIM_COLOR))
                .alignment(Alignment::Center)
        };
        frame.render_widget(status, chunks[2]);
    }
}

fn format_relative_time(datetime_str: &str) -> String {
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S") {
        let now = chrono::Utc::now().naive_utc();
        let diff = now.signed_duration_since(dt);
        
        if diff.num_minutes() < 1 {
            "just now".to_string()
        } else if diff.num_minutes() < 60 {
            format!("{}m ago", diff.num_minutes())
        } else if diff.num_hours() < 24 {
            format!("{}h ago", diff.num_hours())
        } else if diff.num_days() < 7 {
            format!("{}d ago", diff.num_days())
        } else {
            datetime_str.split(' ').next().unwrap_or(datetime_str).to_string()
        }
    } else {
        datetime_str.to_string()
    }
}
