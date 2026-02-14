use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_textarea::TextArea;

use super::{BORDER_COLOR, HIGHLIGHT_COLOR, TEXT_COLOR};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryButton {
    None,
    Run,
    Clear,
    Copy,
}

pub fn render_query_panel(
    frame: &mut Frame,
    area: Rect,
    textarea: &TextArea,
    focused: bool,
    selected_button: QueryButton,
) {
    let border_color = if focused { HIGHLIGHT_COLOR } else { BORDER_COLOR };

    let run_style = button_style(selected_button == QueryButton::Run);
    let clear_style = button_style(selected_button == QueryButton::Clear);
    let copy_style = button_style(selected_button == QueryButton::Copy);

    let title = Line::from(vec![
        Span::styled(" Query ", Style::default().fg(TEXT_COLOR)),
        Span::raw("│ "),
        Span::styled(" ▶ Run ", run_style),
        Span::raw(" "),
        Span::styled(" ✕ Clear ", clear_style),
        Span::raw(" "),
        Span::styled(" ⎘ Copy ", copy_style),
        Span::raw(" "),
    ]);

    let mut ta = textarea.clone();
    ta.set_block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );
    ta.set_style(Style::default().fg(TEXT_COLOR));
    ta.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_widget(&ta, area);
}

fn button_style(selected: bool) -> Style {
    if selected {
        Style::default()
            .fg(ratatui::style::Color::Black)
            .bg(HIGHLIGHT_COLOR)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(HIGHLIGHT_COLOR)
    }
}

pub fn get_button_at_position(area: Rect, x: u16, y: u16) -> QueryButton {
    if y != area.y {
        return QueryButton::None;
    }

    let title_start = area.x + 1;
    let relative_x = x.saturating_sub(title_start);

    if relative_x >= 10 && relative_x < 17 {
        QueryButton::Run
    } else if relative_x >= 18 && relative_x < 27 {
        QueryButton::Clear
    } else if relative_x >= 28 && relative_x < 36 {
        QueryButton::Copy
    } else {
        QueryButton::None
    }
}
