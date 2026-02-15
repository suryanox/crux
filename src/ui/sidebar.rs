use std::collections::BTreeMap;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::db::TableInfo;
use super::theme::{icons, Theme};

#[derive(Debug, Clone)]
pub enum TreeNode {
    Schema { name: String, expanded: bool },
    Table { schema: String, name: String },
}

#[derive(Debug, Default)]
pub struct TreeState {
    pub nodes: Vec<TreeNode>,
    pub selected: usize,
    pub scroll_offset: usize,
}

impl TreeState {
    pub fn from_tables(tables: &[TableInfo]) -> Self {
        let mut grouped: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for t in tables {
            grouped.entry(&t.schema).or_default().push(&t.name);
        }

        let mut nodes = Vec::new();
        for (schema, table_names) in grouped {
            nodes.push(TreeNode::Schema {
                name: schema.to_string(),
                expanded: true,
            });
            for name in table_names {
                nodes.push(TreeNode::Table {
                    schema: schema.to_string(),
                    name: name.to_string(),
                });
            }
        }

        Self {
            nodes,
            selected: 0,
            scroll_offset: 0,
        }
    }

    fn visible_indices(&self) -> Vec<usize> {
        let mut visible = Vec::new();
        let mut current_schema_expanded = true;

        for (idx, node) in self.nodes.iter().enumerate() {
            match node {
                TreeNode::Schema { expanded, .. } => {
                    visible.push(idx);
                    current_schema_expanded = *expanded;
                }
                TreeNode::Table { .. } => {
                    if current_schema_expanded {
                        visible.push(idx);
                    }
                }
            }
        }
        visible
    }

    pub fn visible_nodes(&self) -> Vec<(usize, &TreeNode)> {
        self.visible_indices()
            .into_iter()
            .filter_map(|idx| self.nodes.get(idx).map(|n| (idx, n)))
            .collect()
    }

    pub fn select_next(&mut self) {
        let visible = self.visible_indices();
        if visible.is_empty() {
            return;
        }

        let current_visible_idx = visible
            .iter()
            .position(|&idx| idx == self.selected)
            .unwrap_or(0);

        let next_visible_idx = (current_visible_idx + 1) % visible.len();
        self.selected = visible[next_visible_idx];
    }

    pub fn select_prev(&mut self) {
        let visible = self.visible_indices();
        if visible.is_empty() {
            return;
        }

        let current_visible_idx = visible
            .iter()
            .position(|&idx| idx == self.selected)
            .unwrap_or(0);

        let prev_visible_idx = if current_visible_idx == 0 {
            visible.len() - 1
        } else {
            current_visible_idx - 1
        };
        self.selected = visible[prev_visible_idx];
    }

    pub fn toggle_selected(&mut self) {
        if let Some(TreeNode::Schema { expanded, .. }) = self.nodes.get_mut(self.selected) {
            *expanded = !*expanded;
        }
    }

    pub fn get_selected_table(&self) -> Option<(&str, &str)> {
        match self.nodes.get(self.selected) {
            Some(TreeNode::Table { schema, name }) => Some((schema.as_str(), name.as_str())),
            _ => None,
        }
    }

    pub fn is_selected_schema(&self) -> bool {
        matches!(self.nodes.get(self.selected), Some(TreeNode::Schema { .. }))
    }

    pub fn select_by_click(&mut self, visible_index: usize) {
        let visible = self.visible_indices();
        if let Some(&real_idx) = visible.get(visible_index) {
            self.selected = real_idx;
        }
    }

    pub fn update_scroll(&mut self, visible_height: usize) {
        let visible = self.visible_indices();
        let selected_visible_idx = visible
            .iter()
            .position(|&idx| idx == self.selected)
            .unwrap_or(0);

        if selected_visible_idx < self.scroll_offset {
            self.scroll_offset = selected_visible_idx;
        } else if selected_visible_idx >= self.scroll_offset + visible_height {
            self.scroll_offset = selected_visible_idx.saturating_sub(visible_height - 1);
        }
    }
}

pub fn render_sidebar(
    frame: &mut Frame,
    area: Rect,
    tree_state: &mut TreeState,
    focused: bool,
    theme: &Theme,
) -> Rect {
    let visible_height = area.height.saturating_sub(2) as usize;

    tree_state.update_scroll(visible_height);

    let visible = tree_state.visible_nodes();
    let total_visible = visible.len();
    let scroll_offset = tree_state.scroll_offset;
    let selected = tree_state.selected;

    let items: Vec<ListItem> = visible
        .iter()
        .skip(scroll_offset)
        .take(visible_height)
        .map(|(idx, node)| {
            let is_selected = *idx == selected;
            match node {
                TreeNode::Schema { name, expanded } => {
                    let icon = if *expanded { icons::COLLAPSE } else { icons::EXPAND };
                    let style = if is_selected {
                        theme.selected_style()
                    } else {
                        theme.schema_style()
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(format!(" {} ", icon), theme.dim_style()),
                        Span::styled(name.as_str(), style),
                    ]))
                }
                TreeNode::Table { name, .. } => {
                    let style = if is_selected {
                        theme.selected_style()
                    } else {
                        theme.table_style()
                    };
                    ListItem::new(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(icons::TABLE, theme.dim_style()),
                        Span::raw(" "),
                        Span::styled(name.as_str(), style),
                    ]))
                }
            }
        })
        .collect();

    let title = format!(" {} Database ", icons::DATABASE);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(theme.block_style(focused))
        .style(Style::default().bg(theme.bg_secondary));

    let list = List::new(items).block(block);

    frame.render_widget(list, area);

    if total_visible > visible_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        let mut scrollbar_state = ScrollbarState::new(total_visible)
            .position(scroll_offset);

        let scrollbar_area = Rect::new(
            area.x + area.width - 1,
            area.y + 1,
            1,
            area.height.saturating_sub(2),
        );

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    area
}
