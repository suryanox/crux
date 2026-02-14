use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::db::QueryResult;

use super::{BORDER_COLOR, DIM_COLOR, HIGHLIGHT_COLOR, TEXT_COLOR};

pub fn render_results(
    frame: &mut Frame,
    area: Rect,
    result: &QueryResult,
    state: &mut TableState,
    focused: bool,
) {
    let border_color = if focused { HIGHLIGHT_COLOR } else { BORDER_COLOR };

    if result.columns.is_empty() {
        let block = Block::default()
            .title(" Results ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));
        frame.render_widget(block, area);
        return;
    }

    let header_cells = result
        .columns
        .iter()
        .map(|h| Cell::from(h.clone()).style(Style::default().fg(HIGHLIGHT_COLOR).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = result
        .rows
        .iter()
        .map(|row| {
            let cells = row.iter().map(|c| Cell::from(c.clone()).style(Style::default().fg(TEXT_COLOR)));
            Row::new(cells).height(1)
        })
        .collect();

    let col_count = result.columns.len();
    let col_width = if col_count > 0 {
        (area.width.saturating_sub(2) / col_count as u16).max(10)
    } else {
        10
    };
    let widths: Vec<Constraint> = (0..col_count)
        .map(|_| Constraint::Length(col_width))
        .collect();

    let title = format!(" Results ({} rows) ", result.rows.len());

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .row_highlight_style(Style::default().bg(DIM_COLOR).add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(table, area, state);
}
