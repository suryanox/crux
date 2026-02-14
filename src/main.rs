mod app;
mod db;
mod event;
mod storage;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{Event, KeyCode, KeyModifiers, MouseEventKind, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, Terminal};

use app::{App, AppState, ConnectionFocus, Focus};
use db::DatabaseConnection;
use event::{is_ctrl_enter, poll_event};
use storage::Storage;
use ui::{render_connection_dialog, render_query_panel, render_results, render_sidebar, get_button_at_position, QueryButton};

#[tokio::main]
async fn main() -> Result<()> {
    let storage = Storage::new().await?;
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    
    if let Ok(recent) = storage.get_recent_connections(10).await {
        app.set_recent_connections(recent);
    }
    
    let result = run_app(&mut terminal, &mut app, &storage).await;

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
                    );
                }
                AppState::Browser => {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
                        .split(frame.area());

                    render_sidebar(
                        frame,
                        chunks[0],
                        &app.tables,
                        &mut app.table_state,
                        app.focus == Focus::Sidebar,
                    );

                    let right_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                        .split(chunks[1]);

                    app.query_area = Some(right_chunks[0]);

                    render_query_panel(
                        frame,
                        right_chunks[0],
                        &app.query_input,
                        app.focus == Focus::Query || app.focus == Focus::QueryButtons,
                        app.selected_button,
                    );

                    render_results(
                        frame,
                        right_chunks[1],
                        &app.query_result,
                        &mut app.result_state,
                        app.focus == Focus::Results,
                    );
                }
            }
        })?;

        if let Some(event) = poll_event(Duration::from_millis(100))? {
            match app.state {
                AppState::Connection => {
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
                                                    app.tables = tables;
                                                    if !app.tables.is_empty() {
                                                        app.table_state.select(Some(0));
                                                    }
                                                }
                                                Err(e) => {
                                                    app.connection_error = Some(e.to_string());
                                                    continue;
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
                            KeyCode::Down | KeyCode::Char('j') if app.connection_focus == ConnectionFocus::RecentList => {
                                app.select_next_recent();
                            }
                            KeyCode::Up | KeyCode::Char('k') if app.connection_focus == ConnectionFocus::RecentList => {
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
                AppState::Browser => {
                    match event {
                        Event::Mouse(mouse) => {
                            if let MouseEventKind::Down(_) = mouse.kind {
                                if let Some(query_area) = app.query_area {
                                    let button = get_button_at_position(query_area, mouse.column, mouse.row);
                                    match button {
                                        QueryButton::Run => {
                                            execute_query(app).await;
                                        }
                                        QueryButton::Clear => {
                                            app.clear_query();
                                        }
                                        QueryButton::Copy => {
                                            copy_query_to_clipboard(app);
                                        }
                                        QueryButton::None => {}
                                    }
                                }
                            }
                        }
                        Event::Key(key) => {
                            if key.code == KeyCode::Esc {
                                app.should_quit = true;
                            } else if key.code == KeyCode::Tab {
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    app.cycle_focus_reverse();
                                } else {
                                    app.cycle_focus();
                                }
                            } else if is_ctrl_enter(&key) {
                                execute_query(app).await;
                            } else if app.focus == Focus::QueryButtons {
                                match key.code {
                                    KeyCode::Left | KeyCode::Char('h') => {
                                        app.cycle_button_reverse();
                                    }
                                    KeyCode::Right | KeyCode::Char('l') => {
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
                                            app.select_next_table();
                                        }
                                        KeyCode::Up | KeyCode::Char('k') => {
                                            app.select_prev_table();
                                        }
                                        KeyCode::Enter => {
                                            if let Some(table) = app.get_selected_table() {
                                                let query = format!(
                                                    "SELECT * FROM {}.{} LIMIT 100",
                                                    table.schema, table.name
                                                );
                                                app.query_input = tui_textarea::TextArea::from(vec![query.clone()]);
                                                app.query_input.set_cursor_line_style(ratatui::style::Style::default());
                                                
                                                if let Some(conn) = &app.connection {
                                                    match conn.execute_query(&query).await {
                                                        Ok(result) => {
                                                            app.query_result = result;
                                                            if !app.query_result.rows.is_empty() {
                                                                app.result_state.select(Some(0));
                                                            }
                                                        }
                                                        Err(e) => {
                                                            app.query_result = db::QueryResult {
                                                                columns: vec!["Error".to_string()],
                                                                rows: vec![vec![e.to_string()]],
                                                                affected_rows: 0,
                                                            };
                                                        }
                                                    }
                                                }
                                                app.focus = Focus::Results;
                                            }
                                        }
                                        _ => {}
                                    },
                                    Focus::Query => {
                                        app.query_input.input(Event::Key(key));
                                    }
                                    Focus::QueryButtons => {}
                                    Focus::Results => match key.code {
                                        KeyCode::Down | KeyCode::Char('j') => {
                                            app.select_next_row();
                                        }
                                        KeyCode::Up | KeyCode::Char('k') => {
                                            app.select_prev_row();
                                        }
                                        _ => {}
                                    },
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

async fn execute_query(app: &mut App<'_>) {
    let query = app.get_query_text();
    if query.trim().is_empty() {
        return;
    }
    if let Some(conn) = &app.connection {
        match conn.execute_query(&query).await {
            Ok(result) => {
                app.query_result = result;
                if !app.query_result.rows.is_empty() {
                    app.result_state.select(Some(0));
                }
            }
            Err(e) => {
                app.query_result = db::QueryResult {
                    columns: vec!["Error".to_string()],
                    rows: vec![vec![e.to_string()]],
                    affected_rows: 0,
                };
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
