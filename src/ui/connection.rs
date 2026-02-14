use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use tui_textarea::TextArea;

use super::{BORDER_COLOR, HIGHLIGHT_COLOR, TEXT_COLOR};

pub fn render_connection_dialog(frame: &mut Frame, textarea: &TextArea, error: Option<&str>) {
    let area = frame.area();
    let dialog_width = 60.min(area.width.saturating_sub(4));
    let dialog_height = 7;

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

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .split(inner);

    let hint = Paragraph::new("postgres://, mysql://, or sqlite://")
        .style(Style::default().fg(super::DIM_COLOR))
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[0]);

    let mut ta = textarea.clone();
    ta.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR)),
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
            .style(Style::default().fg(super::DIM_COLOR))
            .alignment(Alignment::Center)
    };
    frame.render_widget(status, chunks[2]);
}
