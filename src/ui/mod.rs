mod connection;
mod query;
mod results;
mod sidebar;

pub use connection::*;
pub use query::{render_query_panel, get_button_at_position, QueryButton};
pub use results::*;
pub use sidebar::*;

use ratatui::style::Color;

pub const BORDER_COLOR: Color = Color::Rgb(100, 100, 100);
pub const HIGHLIGHT_COLOR: Color = Color::Rgb(0, 150, 255);
pub const TEXT_COLOR: Color = Color::White;
pub const DIM_COLOR: Color = Color::Rgb(128, 128, 128);
