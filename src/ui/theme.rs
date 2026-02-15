use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Color,
    pub bg_secondary: Color,
    pub bg_highlight: Color,
    pub bg_selected: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub error: Color,
    pub border: Color,
    pub border_focused: Color,
    pub schema: Color,
    pub table: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            bg: Color::Rgb(22, 22, 30),
            bg_secondary: Color::Rgb(30, 30, 40),
            bg_highlight: Color::Rgb(40, 42, 54),
            bg_selected: Color::Rgb(68, 71, 90),
            text: Color::Rgb(248, 248, 242),
            text_dim: Color::Rgb(149, 152, 170),
            text_muted: Color::Rgb(98, 100, 118),
            accent: Color::Rgb(139, 233, 253),
            error: Color::Rgb(255, 85, 85),
            border: Color::Rgb(68, 71, 90),
            border_focused: Color::Rgb(139, 233, 253),
            schema: Color::Rgb(255, 184, 108),
            table: Color::Rgb(80, 250, 123),
        }
    }

    pub fn text_style(&self) -> Style {
        Style::default().fg(self.text)
    }

    pub fn dim_style(&self) -> Style {
        Style::default().fg(self.text_dim)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.text_muted)
    }

    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn border_focused_style(&self) -> Style {
        Style::default().fg(self.border_focused)
    }

    pub fn selected_style(&self) -> Style {
        Style::default()
            .bg(self.bg_selected)
            .fg(self.text)
            .add_modifier(Modifier::BOLD)
    }

    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    pub fn button_style(&self) -> Style {
        Style::default()
            .fg(self.text_dim)
            .bg(self.bg_secondary)
    }

    pub fn button_hover_style(&self) -> Style {
        Style::default()
            .fg(self.text)
            .bg(self.bg_highlight)
    }

    pub fn button_active_style(&self) -> Style {
        Style::default()
            .fg(self.bg)
            .bg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn schema_style(&self) -> Style {
        Style::default()
            .fg(self.schema)
            .add_modifier(Modifier::BOLD)
    }

    pub fn table_style(&self) -> Style {
        Style::default().fg(self.table)
    }

    pub fn header_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn block_style(&self, focused: bool) -> Style {
        if focused {
            self.border_focused_style()
        } else {
            self.border_style()
        }
    }
}

pub mod icons {
    pub const FOLDER_OPEN: &str = "";
    pub const TABLE: &str = "";
    pub const DATABASE: &str = "";
    pub const PLAY: &str = "";
    pub const COPY: &str = "";
    pub const CLEAR: &str = "";
    pub const EXPAND: &str = "▶";
    pub const COLLAPSE: &str = "▼";
    pub const CONNECTION: &str = "◆";
}
