use ratatui::widgets::ListState;
use tui_textarea::TextArea;

use crate::db::{DatabaseConnection, QueryResult, TableInfo};
use crate::storage::RecentConnection;
use crate::ui::{QueryButton, TreeState, ResultsState};
use crate::ui::query::ButtonRegion;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Connection,
    Browser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionFocus {
    RecentList,
    NewInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    Query,
    QueryButtons,
    Results,
}

pub struct App<'a> {
    pub state: AppState,
    pub focus: Focus,
    pub selected_button: QueryButton,
    pub hovered_button: QueryButton,
    pub connection_input: TextArea<'a>,
    pub connection_error: Option<String>,
    pub connection: Option<DatabaseConnection>,
    pub tables: Vec<TableInfo>,
    pub tree_state: TreeState,
    pub query_input: TextArea<'a>,
    pub query_result: QueryResult,
    pub results_state: ResultsState,
    pub should_quit: bool,
    pub button_region: Option<ButtonRegion>,
    pub recent_connections: Vec<RecentConnection>,
    pub recent_connections_state: ListState,
    pub connection_focus: ConnectionFocus,
    pub sidebar_area: Option<ratatui::layout::Rect>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let mut connection_input = TextArea::default();
        connection_input.set_cursor_line_style(ratatui::style::Style::default());

        let mut query_input = TextArea::default();
        query_input.set_cursor_line_style(ratatui::style::Style::default());

        Self {
            state: AppState::Connection,
            focus: Focus::Sidebar,
            selected_button: QueryButton::None,
            hovered_button: QueryButton::None,
            connection_input,
            connection_error: None,
            connection: None,
            tables: vec![],
            tree_state: TreeState::default(),
            query_input,
            query_result: QueryResult::empty(),
            results_state: ResultsState::new(),
            should_quit: false,
            button_region: None,
            recent_connections: vec![],
            recent_connections_state: ListState::default(),
            connection_focus: ConnectionFocus::RecentList,
            sidebar_area: None,
        }
    }

    pub fn set_tables(&mut self, tables: Vec<TableInfo>) {
        self.tree_state = TreeState::from_tables(&tables);
        self.tables = tables;
    }

    pub fn set_recent_connections(&mut self, connections: Vec<RecentConnection>) {
        self.recent_connections = connections;
        if !self.recent_connections.is_empty() {
            self.recent_connections_state.select(Some(0));
            self.connection_focus = ConnectionFocus::RecentList;
        } else {
            self.connection_focus = ConnectionFocus::NewInput;
        }
    }

    pub fn select_next_recent(&mut self) {
        if self.recent_connections.is_empty() {
            return;
        }
        let i = match self.recent_connections_state.selected() {
            Some(i) => (i + 1) % self.recent_connections.len(),
            None => 0,
        };
        self.recent_connections_state.select(Some(i));
    }

    pub fn select_prev_recent(&mut self) {
        if self.recent_connections.is_empty() {
            return;
        }
        let i = match self.recent_connections_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.recent_connections.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.recent_connections_state.select(Some(i));
    }

    pub fn get_selected_recent_connection(&self) -> Option<&RecentConnection> {
        self.recent_connections_state
            .selected()
            .and_then(|i| self.recent_connections.get(i))
    }

    pub fn toggle_connection_focus(&mut self) {
        self.connection_focus = match self.connection_focus {
            ConnectionFocus::RecentList => ConnectionFocus::NewInput,
            ConnectionFocus::NewInput => {
                if !self.recent_connections.is_empty() {
                    if self.recent_connections_state.selected().is_none() {
                        self.recent_connections_state.select(Some(0));
                    }
                    ConnectionFocus::RecentList
                } else {
                    ConnectionFocus::NewInput
                }
            }
        };
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Sidebar => Focus::Query,
            Focus::Query => Focus::QueryButtons,
            Focus::QueryButtons => Focus::Results,
            Focus::Results => Focus::Sidebar,
        };
        if self.focus == Focus::QueryButtons {
            self.selected_button = QueryButton::Run;
        } else {
            self.selected_button = QueryButton::None;
        }
    }

    pub fn cycle_button(&mut self) {
        self.selected_button = match self.selected_button {
            QueryButton::None => QueryButton::Run,
            QueryButton::Run => QueryButton::Clear,
            QueryButton::Clear => QueryButton::Copy,
            QueryButton::Copy => QueryButton::Run,
        };
    }

    pub fn cycle_button_reverse(&mut self) {
        self.selected_button = match self.selected_button {
            QueryButton::None => QueryButton::Copy,
            QueryButton::Run => QueryButton::Copy,
            QueryButton::Clear => QueryButton::Run,
            QueryButton::Copy => QueryButton::Clear,
        };
    }

    pub fn clear_query(&mut self) {
        self.query_input = TextArea::default();
        self.query_input.set_cursor_line_style(ratatui::style::Style::default());
    }

    pub fn get_query_text(&self) -> String {
        self.query_input.lines().join("\n")
    }

    pub fn set_query_result(&mut self, result: QueryResult) {
        self.results_state.reset();
        self.query_result = result;
    }

    pub fn handle_sidebar_click(&mut self, x: u16, y: u16) -> bool {
        if let Some(area) = self.sidebar_area {
            if x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height {
                let relative_y = y.saturating_sub(area.y + 1) as usize;
                let visible_idx = self.tree_state.scroll_offset + relative_y;
                self.tree_state.select_by_click(visible_idx);
                self.focus = Focus::Sidebar;
                return true;
            }
        }
        false
    }
}
