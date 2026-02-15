use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_textarea::TextArea;

use super::theme::{icons, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryButton {
    None,
    Run,
    Clear,
    Copy,
}

pub struct ButtonRegion {
    pub run: Rect,
    pub clear: Rect,
    pub copy: Rect,
}

impl ButtonRegion {
    pub fn hit_test(&self, x: u16, y: u16) -> QueryButton {
        if self.run.x <= x && x < self.run.x + self.run.width && self.run.y <= y && y < self.run.y + self.run.height {
            return QueryButton::Run;
        }
        if self.clear.x <= x && x < self.clear.x + self.clear.width && self.clear.y <= y && y < self.clear.y + self.clear.height {
            return QueryButton::Clear;
        }
        if self.copy.x <= x && x < self.copy.x + self.copy.width && self.copy.y <= y && y < self.copy.y + self.copy.height {
            return QueryButton::Copy;
        }
        QueryButton::None
    }
}

pub fn render_query_panel(
    frame: &mut Frame,
    area: Rect,
    textarea: &TextArea,
    focused: bool,
    selected_button: QueryButton,
    hovered_button: QueryButton,
    theme: &Theme,
) -> ButtonRegion {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(area);

    let button_area = chunks[0];
    let editor_area = chunks[1];

    let inner_button_area = Rect::new(
        button_area.x + 1,
        button_area.y + 1,
        button_area.width.saturating_sub(2),
        1,
    );

    let run_width = 10u16;
    let clear_width = 11u16;
    let copy_width = 10u16;
    let spacing = 2u16;

    let run_rect = Rect::new(inner_button_area.x, inner_button_area.y, run_width, 1);
    let clear_rect = Rect::new(inner_button_area.x + run_width + spacing, inner_button_area.y, clear_width, 1);
    let copy_rect = Rect::new(inner_button_area.x + run_width + spacing + clear_width + spacing, inner_button_area.y, copy_width, 1);

    let button_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.block_style(focused))
        .style(Style::default().bg(theme.bg_secondary));

    frame.render_widget(button_block, button_area);

    let run_style = get_button_style(QueryButton::Run, selected_button, hovered_button, theme);
    let clear_style = get_button_style(QueryButton::Clear, selected_button, hovered_button, theme);
    let copy_style = get_button_style(QueryButton::Copy, selected_button, hovered_button, theme);

    let run_text = format!(" {} Run ", icons::PLAY);
    let clear_text = format!(" {} Clear ", icons::CLEAR);
    let copy_text = format!(" {} Copy ", icons::COPY);

    frame.render_widget(Paragraph::new(run_text).style(run_style), run_rect);
    frame.render_widget(Paragraph::new(clear_text).style(clear_style), clear_rect);
    frame.render_widget(Paragraph::new(copy_text).style(copy_style), copy_rect);

    let mut ta = textarea.clone();
    ta.set_block(
        Block::default()
            .title(" SQL ")
            .borders(Borders::ALL)
            .border_style(theme.block_style(focused))
            .style(Style::default().bg(theme.bg_secondary)),
    );
    ta.set_style(theme.text_style());
    ta.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED).bg(theme.accent));

    frame.render_widget(&ta, editor_area);

    ButtonRegion {
        run: run_rect,
        clear: clear_rect,
        copy: copy_rect,
    }
}

fn get_button_style(button: QueryButton, selected: QueryButton, hovered: QueryButton, theme: &Theme) -> Style {
    if button == selected {
        theme.button_active_style()
    } else if button == hovered {
        theme.button_hover_style()
    } else {
        theme.button_style()
    }
}
