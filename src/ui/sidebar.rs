use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::db::TableInfo;

use super::{BORDER_COLOR, DIM_COLOR, HIGHLIGHT_COLOR, TEXT_COLOR};

pub fn render_sidebar(
    frame: &mut Frame,
    area: Rect,
    tables: &[TableInfo],
    state: &mut ListState,
    focused: bool,
) {
    let items: Vec<ListItem> = tables
        .iter()
        .map(|t| {
            let content = Line::from(vec![
                Span::styled(format!("{}.", t.schema), Style::default().fg(DIM_COLOR)),
                Span::styled(&t.name, Style::default().fg(TEXT_COLOR)),
            ]);
            ListItem::new(content)
        })
        .collect();

    let border_color = if focused { HIGHLIGHT_COLOR } else { BORDER_COLOR };

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Tables ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .highlight_style(
            Style::default()
                .fg(HIGHLIGHT_COLOR)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, state);
}
