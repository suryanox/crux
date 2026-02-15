use ratatui::{
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, Borders, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::db::QueryResult;
use super::theme::Theme;

#[derive(Debug, Default)]
pub struct ResultsState {
    pub selected_row: usize,
    pub scroll_offset: usize,
    pub horizontal_scroll: usize,
    pub column_widths: Vec<u16>,
}

impl ResultsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.selected_row = 0;
        self.scroll_offset = 0;
        self.horizontal_scroll = 0;
        self.column_widths.clear();
    }

    pub fn select_next(&mut self, total_rows: usize) {
        if total_rows == 0 {
            return;
        }
        self.selected_row = (self.selected_row + 1) % total_rows;
    }

    pub fn select_prev(&mut self, total_rows: usize) {
        if total_rows == 0 {
            return;
        }
        if self.selected_row == 0 {
            self.selected_row = total_rows - 1;
        } else {
            self.selected_row -= 1;
        }
    }

    pub fn scroll_left(&mut self) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
    }

    pub fn scroll_right(&mut self, max_scroll: usize) {
        if self.horizontal_scroll < max_scroll {
            self.horizontal_scroll += 1;
        }
    }

    pub fn calculate_column_widths(&mut self, result: &QueryResult, _max_width: u16) {
        if result.columns.is_empty() {
            self.column_widths.clear();
            return;
        }

        let mut widths: Vec<u16> = result
            .columns
            .iter()
            .map(|h| (h.width() as u16).max(8).min(40))
            .collect();

        for row in &result.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    let cell_width = (cell.width() as u16).max(8).min(50);
                    widths[i] = widths[i].max(cell_width);
                }
            }
        }

        let min_col_width = 12u16;
        let max_col_width = 50u16;
        
        for w in &mut widths {
            *w = (*w + 2).clamp(min_col_width, max_col_width);
        }

        self.column_widths = widths;
    }
}

pub fn render_results(
    frame: &mut Frame,
    area: Rect,
    result: &QueryResult,
    state: &mut ResultsState,
    focused: bool,
    theme: &Theme,
) {
    if result.columns.is_empty() {
        let block = Block::default()
            .title(" Results ")
            .borders(Borders::ALL)
            .border_style(theme.block_style(focused))
            .style(Style::default().bg(theme.bg_secondary));
        frame.render_widget(block, area);
        return;
    }

    if state.column_widths.is_empty() || state.column_widths.len() != result.columns.len() {
        state.calculate_column_widths(result, area.width);
    }

    let visible_height = area.height.saturating_sub(4) as usize;

    if state.selected_row < state.scroll_offset {
        state.scroll_offset = state.selected_row;
    } else if state.selected_row >= state.scroll_offset + visible_height {
        state.scroll_offset = state.selected_row.saturating_sub(visible_height - 1);
    }

    let header_cells: Vec<Cell> = result
        .columns
        .iter()
        .map(|h| Cell::from(h.clone()).style(theme.header_style()))
        .collect();
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = result
        .rows
        .iter()
        .enumerate()
        .skip(state.scroll_offset)
        .take(visible_height)
        .map(|(idx, row)| {
            let is_selected = idx == state.selected_row;
            let row_style = if is_selected {
                theme.selected_style()
            } else if idx % 2 == 0 {
                Style::default().bg(theme.bg_secondary)
            } else {
                Style::default().bg(theme.bg)
            };

            let cells: Vec<Cell> = row
                .iter()
                .map(|c| {
                    let display = if c.len() > 47 {
                        format!("{}...", &c[..47])
                    } else {
                        c.clone()
                    };
                    Cell::from(display).style(theme.text_style())
                })
                .collect();
            Row::new(cells).height(1).style(row_style)
        })
        .collect();

    let widths: Vec<Constraint> = state
        .column_widths
        .iter()
        .map(|&w| Constraint::Length(w))
        .collect();

    let title = format!(" Results ({} rows) ", result.rows.len());

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(theme.block_style(focused))
                .style(Style::default().bg(theme.bg_secondary)),
        )
        .row_highlight_style(theme.selected_style())
        .column_spacing(1);

    frame.render_widget(table, area);

    if result.rows.len() > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        let mut scrollbar_state = ScrollbarState::new(result.rows.len())
            .position(state.scroll_offset);

        let scrollbar_area = Rect::new(
            area.x + area.width - 1,
            area.y + 2,
            1,
            area.height.saturating_sub(3),
        );

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    let total_width: u16 = state.column_widths.iter().sum::<u16>() + state.column_widths.len() as u16;
    let content_width = area.width.saturating_sub(3);
    if total_width > content_width {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
            .begin_symbol(Some("◀"))
            .end_symbol(Some("▶"))
            .track_symbol(Some("─"))
            .thumb_symbol("█");

        let max_h_scroll = (total_width.saturating_sub(content_width)) as usize;
        let mut scrollbar_state = ScrollbarState::new(max_h_scroll)
            .position(state.horizontal_scroll);

        let scrollbar_area = Rect::new(
            area.x + 1,
            area.y + area.height - 1,
            area.width.saturating_sub(2),
            1,
        );

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}
