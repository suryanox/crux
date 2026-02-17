mod app;
mod db;
mod event;
mod storage;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, Terminal};

use app::{App, AppState, ConnectionFocus, Focus};
use db::DatabaseConnection;
use event::poll_event;
use storage::Storage;
use ui::{render_connection_dialog, render_query_panel, render_results, render_sidebar, QueryButton, Theme};

#[tokio::main]
async fn main() -> Result<()> {
    let storage = Storage::new().await?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let theme = Theme::default();

    if let Ok(recent) = storage.get_recent_connections(10).await {
        app.set_recent_connections(recent);
    }

    let result = run_app(&mut terminal, &mut app, &storage, &theme).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err}");
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App<'_>,
    storage: &Storage,
    theme: &Theme,
) -> Result<()> {
    loop {
        terminal.draw(|frame| {
            match app.state {
                AppState::Connection => {
                    render_connection_dialog(
                        frame,
                        &app.connection_input,
                        app.connection_error.as_deref(),
                        &app.recent_connections,
                        &mut app.recent_connections_state,
                        app.connection_focus,
                        theme,
                    );
                }
                AppState::Browser => {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(22), Constraint::Percentage(78)])
                        .split(frame.area());

                    app.sidebar_area = Some(chunks[0]);

                    render_sidebar(
                        frame,
                        chunks[0],
                        &mut app.tree_state,
                        app.focus == Focus::Sidebar,
                        theme,
                    );

                    let right_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(8), Constraint::Min(10)])
                        .split(chunks[1]);

                    let button_region = render_query_panel(
                        frame,
                        right_chunks[0],
                        &app.query_input,
                        app.focus == Focus::Query || app.focus == Focus::QueryButtons,
                        app.selected_button,
                        app.hovered_button,
                        theme,
                    );
                    app.button_region = Some(button_region);

                    render_results(
                        frame,
                        right_chunks[1],
                        &app.query_result,
                        &mut app.results_state,
                        app.focus == Focus::Results,
                        theme,
                    );
                }
            }
        })?;

        if let Some(event) = poll_event(Duration::from_millis(50))? {
            match app.state {
                AppState::Connection => {
                    handle_connection_event(app, storage, event).await;
                }
                AppState::Browser => {
                    handle_browser_event(app, event).await;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

async fn handle_connection_event(app: &mut App<'_>, storage: &Storage, event: Event) {
    if let Event::Key(key) = event {
        match key.code {
            KeyCode::Esc => {
                app.should_quit = true;
            }
            KeyCode::Tab => {
                app.toggle_connection_focus();
            }
            KeyCode::Enter => {
                let conn_str = match app.connection_focus {
                    ConnectionFocus::RecentList => {
                        app.get_selected_recent_connection()
                            .map(|c| c.connection_string.clone())
                    }
                    ConnectionFocus::NewInput => {
                        let input = app.connection_input.lines().join("");
                        if input.is_empty() { None } else { Some(input) }
                    }
                };

                if let Some(conn_str) = conn_str {
                    match DatabaseConnection::connect(&conn_str).await {
                        Ok(conn) => {
                            match conn.get_tables().await {
                                Ok(tables) => {
                                    app.set_tables(tables);
                                }
                                Err(e) => {
                                    app.connection_error = Some(e.to_string());
                                    return;
                                }
                            }
                            let _ = storage.add_connection(&conn_str).await;

                            app.connection = Some(conn);
                            app.connection_error = None;
                            app.state = AppState::Browser;
                        }
                        Err(e) => {
                            app.connection_error = Some(e.to_string());
                        }
                    }
                }
            }
            KeyCode::Delete | KeyCode::Backspace if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if app.connection_focus == ConnectionFocus::RecentList {
                    if let Some(conn) = app.get_selected_recent_connection() {
                        let id = conn.id;
                        let _ = storage.delete_connection(id).await;
                        if let Ok(recent) = storage.get_recent_connections(10).await {
                            app.set_recent_connections(recent);
                        }
                    }
                }
            }
            KeyCode::Down if app.connection_focus == ConnectionFocus::RecentList => {
                app.select_next_recent();
            }
            KeyCode::Up if app.connection_focus == ConnectionFocus::RecentList => {
                app.select_prev_recent();
            }
            _ => {
                if app.connection_focus == ConnectionFocus::NewInput {
                    app.connection_input.input(event);
                }
            }
        }
    }
}

async fn handle_browser_event(app: &mut App<'_>, event: Event) {
    match event {
        Event::Mouse(mouse) => {
            match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left) => {
                    if let Some(ratio) = app.results_state.scrollbar_region.hit_test_vertical(mouse.column, mouse.row) {
                        let total_rows = app.query_result.rows.len();
                        app.results_state.scroll_to_vertical_ratio(ratio, total_rows);
                        app.focus = Focus::Results;
                        return;
                    }

                    if let Some(ratio) = app.results_state.scrollbar_region.hit_test_horizontal(mouse.column, mouse.row) {
                        app.results_state.scroll_to_horizontal_ratio(ratio);
                        app.focus = Focus::Results;
                        return;
                    }

                    if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
                        if let Some(ref region) = app.button_region {
                            let button = region.hit_test(mouse.column, mouse.row);
                            match button {
                                QueryButton::Run => {
                                    execute_query(app).await;
                                    return;
                                }
                                QueryButton::Clear => {
                                    app.clear_query();
                                    return;
                                }
                                QueryButton::Copy => {
                                    copy_query_to_clipboard(app);
                                    return;
                                }
                                QueryButton::None => {}
                            }
                        }

                        if app.handle_sidebar_click(mouse.column, mouse.row) {
                            return;
                        }
                    }
                }
                MouseEventKind::Moved => {
                    if let Some(ref region) = app.button_region {
                        app.hovered_button = region.hit_test(mouse.column, mouse.row);
                    }
                }
                MouseEventKind::ScrollUp => {
                    if app.focus == Focus::Results {
                        app.results_state.select_prev(app.query_result.rows.len());
                    } else if app.focus == Focus::Sidebar {
                        app.tree_state.select_prev();
                    }
                }
                MouseEventKind::ScrollDown => {
                    if app.focus == Focus::Results {
                        app.results_state.select_next(app.query_result.rows.len());
                    } else if app.focus == Focus::Sidebar {
                        app.tree_state.select_next();
                    }
                }
                _ => {}
            }
        }
        Event::Key(key) => {
            if key.code == KeyCode::Esc {
                app.should_quit = true;
            } else if key.code == KeyCode::Tab {
                app.cycle_focus();
            } else if app.focus == Focus::QueryButtons {
                match key.code {
                    KeyCode::Left => {
                        app.cycle_button_reverse();
                    }
                    KeyCode::Right => {
                        app.cycle_button();
                    }
                    KeyCode::Enter => {
                        match app.selected_button {
                            QueryButton::Run => execute_query(app).await,
                            QueryButton::Clear => app.clear_query(),
                            QueryButton::Copy => copy_query_to_clipboard(app),
                            QueryButton::None => {}
                        }
                    }
                    _ => {}
                }
            } else {
                match app.focus {
                    Focus::Sidebar => match key.code {
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.tree_state.select_next();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.tree_state.select_prev();
                        }
                        KeyCode::Enter | KeyCode::Right => {
                            if app.tree_state.is_selected_schema() {
                                app.tree_state.toggle_selected();
                            } else if let Some((schema, table)) = app.tree_state.get_selected_table() {
                                let query = format!(
                                    "SELECT * FROM {}.{} LIMIT 100",
                                    schema, table
                                );
                                app.query_input = tui_textarea::TextArea::from(vec![query.clone()]);
                                app.query_input.set_cursor_line_style(ratatui::style::Style::default());

                                if let Some(conn) = &app.connection {
                                    match conn.execute_query(&query).await {
                                        Ok(result) => {
                                            app.set_query_result(result);
                                        }
                                        Err(e) => {
                                            app.set_query_result(db::QueryResult {
                                                columns: vec!["Error".to_string()],
                                                rows: vec![vec![e.to_string()]],
                                                affected_rows: 0,
                                            });
                                        }
                                    }
                                }
                                app.focus = Focus::Results;
                            }
                        }
                        KeyCode::Left => {
                            if app.tree_state.is_selected_schema() {
                                app.tree_state.toggle_selected();
                            }
                        }
                        KeyCode::Char(' ') => {
                            app.tree_state.toggle_selected();
                        }
                        _ => {}
                    },
                    Focus::Query => {
                        app.query_input.input(Event::Key(key));
                    }
                    Focus::QueryButtons => {}
                    Focus::Results => match key.code {
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.results_state.select_next(app.query_result.rows.len());
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.results_state.select_prev(app.query_result.rows.len());
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            app.results_state.scroll_left();
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            let max_scroll = app.results_state.column_widths.iter().sum::<u16>() as usize;
                            app.results_state.scroll_right(max_scroll);
                        }
                        _ => {}
                    },
                }
            }
        }
        _ => {}
    }
}

async fn execute_query(app: &mut App<'_>) {
    let query = app.get_query_text();
    if query.trim().is_empty() {
        return;
    }
    if let Some(conn) = &app.connection {
        match conn.execute_query(&query).await {
            Ok(result) => {
                app.set_query_result(result);
            }
            Err(e) => {
                app.set_query_result(db::QueryResult {
                    columns: vec!["Error".to_string()],
                    rows: vec![vec![e.to_string()]],
                    affected_rows: 0,
                });
            }
        }
    }
}

fn copy_query_to_clipboard(app: &App<'_>) {
    let query = app.get_query_text();
    if !query.is_empty() {
        #[cfg(target_os = "macos")]
        {
            use std::process::{Command, Stdio};
            if let Ok(mut child) = Command::new("pbcopy")
                .stdin(Stdio::piped())
                .spawn()
            {
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    let _ = stdin.write_all(query.as_bytes());
                }
                let _ = child.wait();
            }
        }
        #[cfg(target_os = "linux")]
        {
            use std::process::{Command, Stdio};
            if let Ok(mut child) = Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(Stdio::piped())
                .spawn()
            {
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    let _ = stdin.write_all(query.as_bytes());
                }
                let _ = child.wait();
            }
        }
    }
}
