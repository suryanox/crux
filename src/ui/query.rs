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
    let editor_area = area;

    let run_width = 10u16;
    let clear_width = 11u16;
    let copy_width = 10u16;
    let spacing = 1u16;
    let total_buttons_width = run_width + clear_width + copy_width + spacing * 2;

    let buttons_x = area.x + area.width.saturating_sub(total_buttons_width + 2);
    let buttons_y = area.y;

    let run_rect = Rect::new(buttons_x, buttons_y, run_width, 1);
    let clear_rect = Rect::new(buttons_x + run_width + spacing, buttons_y, clear_width, 1);
    let copy_rect = Rect::new(buttons_x + run_width + spacing + clear_width + spacing, buttons_y, copy_width, 1);

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

    let run_style = get_button_style(QueryButton::Run, selected_button, hovered_button, theme);
    let clear_style = get_button_style(QueryButton::Clear, selected_button, hovered_button, theme);
    let copy_style = get_button_style(QueryButton::Copy, selected_button, hovered_button, theme);

    let run_text = format!(" {} Run ", icons::PLAY);
    let clear_text = format!(" {} Clear ", icons::CLEAR);
    let copy_text = format!(" {} Copy ", icons::COPY);

    frame.render_widget(Paragraph::new(run_text).style(run_style), run_rect);
    frame.render_widget(Paragraph::new(clear_text).style(clear_style), clear_rect);
    frame.render_widget(Paragraph::new(copy_text).style(copy_style), copy_rect);

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
