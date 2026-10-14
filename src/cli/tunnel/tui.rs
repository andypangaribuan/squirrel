/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use super::config::{ActiveRecord, TunnelItem, TunnelStatus, current_timestamp, format_duration, get_item_status, load_active, load_items};
use super::manager;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, Table, TableState},
};
use std::{
    collections::BTreeMap,
    io::{self, Stdout},
    time::Duration,
};

#[derive(PartialEq, Eq)]
enum ModalAction {
    Execute,
    Cancel,
}

struct ModalState {
    item_id: String,
    status: TunnelStatus,
    selected: ModalAction,
}

pub struct App {
    items: Vec<TunnelItem>,
    active_records: BTreeMap<String, ActiveRecord>,
    table_state: TableState,
    modal: Option<ModalState>,
    status_message: Option<(String, Color)>,
    error_message: Option<String>,
    search_query: String,
    search_active: bool,
    pub is_min: bool,
}

impl App {
    pub fn new(is_min: bool) -> Self {
        let mut app = Self {
            items: Vec::new(),
            active_records: BTreeMap::new(),
            table_state: TableState::default(),
            modal: None,
            status_message: None,
            error_message: None,
            search_query: String::new(),
            search_active: false,
            is_min,
        };
        app.reload_items();
        app.refresh_active();
        if !app.items.is_empty() {
            app.table_state.select(Some(0));
        }
        app
    }

    pub fn filtered_indices(&self) -> Vec<usize> {
        if self.search_query.trim().is_empty() {
            (0..self.items.len()).collect()
        } else {
            self.items.iter().enumerate().filter(|(_, it)| it.matches_query(self.search_query.trim())).map(|(idx, _)| idx).collect()
        }
    }

    pub fn clamp_selection(&mut self) {
        let count = self.filtered_indices().len();
        if count == 0 {
            self.table_state.select(None);
        } else if let Some(selected) = self.table_state.selected() {
            if selected >= count {
                self.table_state.select(Some(count - 1));
            }
        } else {
            self.table_state.select(Some(0));
        }
    }

    pub fn reload_items(&mut self) {
        match load_items() {
            Ok(items) => {
                self.items = items;
                self.error_message = None;
                self.clamp_selection();
            }
            Err(e) => {
                self.error_message = Some(e);
            }
        }
    }

    pub fn refresh_active(&mut self) {
        manager::sync_active_records();
        if let Ok(records) = load_active() {
            self.active_records = records;
        }
    }

    pub fn next_item(&mut self) {
        let count = self.filtered_indices().len();
        if count == 0 {
            self.table_state.select(None);
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i + 1 >= count {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous_item(&mut self) {
        let count = self.filtered_indices().len();
        if count == 0 {
            self.table_state.select(None);
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn open_modal(&mut self) {
        let indices = self.filtered_indices();
        if let Some(selected) = self.table_state.selected()
            && let Some(&item_idx) = indices.get(selected)
            && let Some(item) = self.items.get(item_idx)
        {
            let (status, _) = get_item_status(item.item_id(), &self.active_records);
            self.modal = Some(ModalState { item_id: item.item_id().to_string(), status, selected: ModalAction::Execute });
        }
    }

    pub fn close_modal(&mut self) {
        self.modal = None;
    }

    pub fn confirm_modal_action(&mut self) {
        if let Some(modal) = self.modal.take()
            && modal.selected == ModalAction::Execute
            && let Some(item) = self.items.iter().find(|i| i.item_id() == modal.item_id)
        {
            if modal.status == TunnelStatus::Inactive {
                // Start tunnel
                match manager::start_tunnel(item) {
                    Ok(_) => {
                        self.status_message = Some((format!("Starting tunnel '{}'...", item.name), Color::Green));
                    }
                    Err(e) => {
                        self.status_message = Some((format!("Failed to start: {}", e), Color::Red));
                    }
                }
            } else {
                // Stop tunnel
                match manager::stop_tunnel(item.item_id()) {
                    Ok(_) => {
                        self.status_message = Some((format!("Stopped tunnel '{}'", item.name), Color::Yellow));
                    }
                    Err(e) => {
                        self.status_message = Some((format!("Failed to stop: {}", e), Color::Red));
                    }
                }
            }
            self.refresh_active();
        }
    }
}

pub fn run(is_min: bool) -> Result<(), String> {
    enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| format!("Failed to enter alternate screen: {}", e))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| format!("Failed to initialize terminal: {}", e))?;

    let res = run_app(&mut terminal, is_min);

    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    res
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, is_min: bool) -> Result<(), String> {
    let mut app = App::new(is_min);

    loop {
        terminal.draw(|f| ui(f, &mut app)).map_err(|e| format!("Failed to draw UI: {}", e))?;

        if event::poll(Duration::from_millis(500)).map_err(|e| format!("Event poll error: {}", e))?
            && let Event::Key(key) = event::read().map_err(|e| format!("Event read error: {}", e))?
            && key.kind == KeyEventKind::Press
        {
            if app.modal.is_some() {
                // Modal navigation
                match key.code {
                    KeyCode::Esc => {
                        app.close_modal();
                    }
                    KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                        if let Some(ref mut m) = app.modal {
                            m.selected = if m.selected == ModalAction::Execute { ModalAction::Cancel } else { ModalAction::Execute };
                        }
                    }
                    KeyCode::Enter => {
                        app.confirm_modal_action();
                    }
                    _ => {}
                }
            } else if app.search_active {
                // Active search input mode
                match key.code {
                    KeyCode::Esc => {
                        app.search_active = false;
                        app.search_query.clear();
                        app.clamp_selection();
                    }
                    KeyCode::Enter => {
                        // Confirm search and transfer focus to the table
                        app.search_active = false;
                    }
                    KeyCode::Down => {
                        // Move focus to table and select next item
                        app.search_active = false;
                        app.next_item();
                    }
                    KeyCode::Up => {
                        // Move focus to table and select prev item
                        app.search_active = false;
                        app.previous_item();
                    }
                    KeyCode::Backspace => {
                        app.search_query.pop();
                        app.clamp_selection();
                    }
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Char('u') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        app.search_query.clear();
                        app.clamp_selection();
                    }
                    KeyCode::Char(c) => {
                        app.search_query.push(c);
                        app.clamp_selection();
                    }
                    _ => {}
                }
            } else {
                // Main view navigation
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Esc => {
                        if !app.search_query.is_empty() {
                            app.search_query.clear();
                            app.clamp_selection();
                        } else {
                            return Ok(());
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Char('f') | KeyCode::Char('/') => {
                        app.search_active = true;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.previous_item();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.next_item();
                    }
                    KeyCode::Home | KeyCode::Char('g') => {
                        let count = app.filtered_indices().len();
                        if count > 0 {
                            app.table_state.select(Some(0));
                        }
                    }
                    KeyCode::End | KeyCode::Char('G') => {
                        let count = app.filtered_indices().len();
                        if count > 0 {
                            app.table_state.select(Some(count - 1));
                        }
                    }
                    KeyCode::Char('o') => {
                        match manager::stop_all_tunnels() {
                            Ok(0) => {
                                app.status_message = Some(("No active tunnels to stop".to_string(), Color::DarkGray));
                            }
                            Ok(count) => {
                                app.status_message = Some((format!("Stopped {} active tunnel(s)", count), Color::Yellow));
                            }
                            Err(e) => {
                                app.status_message = Some((format!("Failed to stop all tunnels: {}", e), Color::Red));
                            }
                        }
                        app.refresh_active();
                    }
                    KeyCode::Char('p') => {
                        match manager::restart_all_tunnels() {
                            Ok(0) => {
                                app.status_message = Some(("No active tunnels to restart".to_string(), Color::DarkGray));
                            }
                            Ok(count) => {
                                app.status_message = Some((format!("Restarting {} tunnel(s)...", count), Color::Green));
                            }
                            Err(e) => {
                                app.status_message = Some((format!("Failed to restart tunnels: {}", e), Color::Red));
                            }
                        }
                        app.refresh_active();
                    }
                    KeyCode::Char('r') => {
                        app.reload_items();
                        app.refresh_active();
                        app.status_message = Some(("Reloaded items and active status".to_string(), Color::Cyan));
                    }
                    KeyCode::Enter => {
                        app.open_modal();
                    }
                    _ => {}
                }
            }
        } else {
            // Periodic tick to refresh active status
            app.refresh_active();
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let size = f.area();
    let show_search_bar = app.search_active || !app.search_query.is_empty();

    let chunks = if show_search_bar {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search bar
                Constraint::Min(6),    // Table list
                Constraint::Length(6), // Detail footer
                Constraint::Length(3), // Bottom shortcuts bar
            ])
            .split(size)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(6),    // Table list
                Constraint::Length(6), // Detail footer
                Constraint::Length(3), // Bottom shortcuts bar
            ])
            .split(size)
    };

    let (search_chunk_opt, table_chunk, footer_chunk, bottom_chunk) =
        if show_search_bar { (Some(chunks[0]), chunks[1], chunks[2], chunks[3]) } else { (None, chunks[0], chunks[1], chunks[2]) };

    let filtered_indices = app.filtered_indices();
    let filtered_count = filtered_indices.len();
    let total_count = app.items.len();

    // 1. Search Bar (when active or filter present)
    if let Some(search_chunk) = search_chunk_opt {
        let (border_color, title_text) = if app.search_active {
            (Color::Yellow, " Search [Type to filter | Enter: Focus Table | Esc: Clear] ")
        } else {
            (Color::Cyan, " Search [f: Edit filter | Esc: Clear filter] ")
        };

        let search_block = Block::default()
            .title(title_text)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color));

        let cursor = if app.search_active { "█" } else { "" };

        let search_line = Line::from(vec![
            Span::styled(" Filter: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(&app.search_query, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(cursor, Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled(format!("({} of {} tunnels)", filtered_count, total_count), Style::default().fg(Color::DarkGray)),
        ]);

        let search_para = Paragraph::new(vec![search_line]).block(search_block);
        f.render_widget(search_para, search_chunk);
    }

    // 3. Main List / Table
    let table_title = if !app.search_query.is_empty() {
        format!(" Tunnels [Filter: \"{}\" - {} matches] ", app.search_query, filtered_count)
    } else {
        " Tunnels ".to_string()
    };

    let table_block = Block::default()
        .title(table_title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    if let Some(ref err) = app.error_message {
        let err_para = Paragraph::new(vec![
            Line::from(Span::styled("Error reading items.json:", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled(err, Style::default().fg(Color::Red))),
            Line::from(Span::styled("Please fix items.json and press [r] to reload.", Style::default().fg(Color::DarkGray))),
        ])
        .block(table_block);
        f.render_widget(err_para, table_chunk);
    } else if app.items.is_empty() {
        let empty_para = Paragraph::new(vec![
            Line::from(Span::styled("No tunnels found in ~/.squirrel/tunnel/items.json", Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled("Add a tunnel configuration and press [r] to reload.", Style::default().fg(Color::DarkGray))),
        ])
        .block(table_block)
        .alignment(Alignment::Center);
        f.render_widget(empty_para, table_chunk);
    } else if filtered_count == 0 {
        let empty_para = Paragraph::new(vec![
            Line::from(Span::styled(
                format!("No tunnels match \"{}\"", app.search_query),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled("Press [Esc] to clear search filter.", Style::default().fg(Color::DarkGray))),
        ])
        .block(table_block)
        .alignment(Alignment::Center);
        f.render_widget(empty_para, table_chunk);
    } else {
        let selected_style = Style::default().bg(Color::Rgb(35, 45, 65)).add_modifier(Modifier::BOLD);

        let header_row = if app.is_min {
            Row::new(vec![
                Span::styled(" ", Style::default()),
                Span::styled("Name", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("Forwarding", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("Duration", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ])
            .height(1)
            .bottom_margin(1)
        } else {
            Row::new(vec![
                Span::styled(" ", Style::default()),
                Span::styled("Name", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("Type", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("Forwarding", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("Target", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("Duration", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("Description", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ])
            .height(1)
            .bottom_margin(1)
        };

        let max_name_len = filtered_indices.iter().map(|&i| app.items[i].name.len()).max().unwrap_or(4).max(4);
        let max_type_len = filtered_indices.iter().map(|&i| app.items[i].tunnel_type.len()).max().unwrap_or(4).max(4);
        let max_local_len = filtered_indices
            .iter()
            .map(|&i| {
                let item = &app.items[i];
                format!("{}:{}", item.local_bind, item.local_port).len()
            })
            .max()
            .unwrap_or(0);
        let max_forward_len = if app.is_min {
            max_local_len.max(10)
        } else {
            filtered_indices
                .iter()
                .map(|&i| {
                    let item = &app.items[i];
                    let local_str = format!("{}:{}", item.local_bind, item.local_port);
                    let remote_target = if item.is_cloud_sql_proxy() {
                        format!(":{}", item.remote_port())
                    } else {
                        format!("{}:{}", item.remote_bind, item.remote_port())
                    };
                    format!("{:<max_local_len$} -> {}", local_str, remote_target).len()
                })
                .max()
                .unwrap_or(10)
                .max(10)
        };
        let max_target_len = filtered_indices
            .iter()
            .map(|&i| {
                let item = &app.items[i];
                item.host.as_deref().or(item.instance.as_deref()).unwrap_or("-").len()
            })
            .max()
            .unwrap_or(6)
            .max(6);

        let rows: Vec<Row> = filtered_indices
            .iter()
            .map(|&i| {
                let item = &app.items[i];
                let (status, maybe_record) = get_item_status(item.item_id(), &app.active_records);

                // Bullet color based on status:
                // inactive = gray, connecting = yellow, disconnected = red, connected = green
                let (bullet_char, bullet_color) = match status {
                    TunnelStatus::Inactive => ("●", Color::Rgb(128, 128, 128)),
                    TunnelStatus::Connecting => ("●", Color::Yellow),
                    TunnelStatus::Connected => ("●", Color::Green),
                    TunnelStatus::Disconnected => ("●", Color::Red),
                };

                let duration_str = if status == TunnelStatus::Connected {
                    format_duration(maybe_record.as_ref().and_then(|r| r.connected_at), current_timestamp())
                } else {
                    "-".to_string()
                };
                let duration_style =
                    if duration_str == "-" { Style::default().fg(Color::DarkGray) } else { Style::default().fg(Color::White) };

                if app.is_min {
                    let forward_str = format!("{}:{}", item.local_bind, item.local_port);
                    Row::new(vec![
                        Span::styled(bullet_char, Style::default().fg(bullet_color)),
                        Span::styled(item.name.clone(), Style::default()),
                        Span::styled(forward_str, Style::default().fg(Color::White)),
                        Span::styled(duration_str, duration_style),
                    ])
                } else {
                    let type_span = if item.is_cloud_sql_proxy() {
                        Span::styled(item.tunnel_type.clone(), Style::default().fg(Color::Magenta))
                    } else if item.is_gcloud() {
                        Span::styled(item.tunnel_type.clone(), Style::default().fg(Color::LightGreen))
                    } else {
                        Span::styled(item.tunnel_type.clone(), Style::default().fg(Color::Cyan))
                    };

                    let local_str = format!("{}:{}", item.local_bind, item.local_port);
                    let remote_target = if item.is_cloud_sql_proxy() {
                        format!(":{}", item.remote_port())
                    } else {
                        format!("{}:{}", item.remote_bind, item.remote_port())
                    };
                    let forward_str = format!("{:<max_local_len$} -> {}", local_str, remote_target);

                    let target_str = if let Some(ref h) = item.host {
                        h.clone()
                    } else if let Some(ref inst) = item.instance {
                        inst.clone()
                    } else {
                        "-".to_string()
                    };

                    Row::new(vec![
                        Span::styled(bullet_char, Style::default().fg(bullet_color)),
                        Span::styled(item.name.clone(), Style::default()),
                        type_span,
                        Span::styled(forward_str, Style::default().fg(Color::White)),
                        Span::styled(target_str, Style::default().fg(Color::DarkGray)),
                        Span::styled(duration_str, duration_style),
                        Span::styled(item.description.clone(), Style::default().fg(Color::Gray)),
                    ])
                }
            })
            .collect();

        let table = if app.is_min {
            Table::new(
                rows,
                [
                    Constraint::Length(2),                       // Bullet
                    Constraint::Length(max_name_len as u16 + 1), // Name
                    Constraint::Min(max_forward_len as u16 + 1), // Forwarding
                    Constraint::Length(10),                      // Duration
                    Constraint::Fill(1),                         // Extra space filler
                ],
            )
        } else {
            Table::new(
                rows,
                [
                    Constraint::Length(2),                         // Bullet
                    Constraint::Length(max_name_len as u16 + 1),   // Name
                    Constraint::Length(max_type_len as u16 + 1),   // Type
                    Constraint::Min(max_forward_len as u16 + 1),   // Forwarding
                    Constraint::Length(max_target_len as u16 + 1), // Target
                    Constraint::Length(10),                        // Duration
                    Constraint::Fill(1),                           // Description
                ],
            )
        }
        .header(header_row)
        .block(table_block)
        .row_highlight_style(selected_style)
        .highlight_symbol(" ");

        f.render_stateful_widget(table, table_chunk, &mut app.table_state);
    }

    // 4. Detail Footer
    let selected_index = app.table_state.selected();
    let selected_item = selected_index.and_then(|sel| filtered_indices.get(sel)).and_then(|&idx| app.items.get(idx));

    let detail_lines = if let Some(item) = selected_item {
        let (status, record) = get_item_status(item.item_id(), &app.active_records);

        let (status_text, status_color) = match status {
            TunnelStatus::Inactive => ("Inactive", Color::Rgb(128, 128, 128)),
            TunnelStatus::Connecting => ("Connecting...", Color::Yellow),
            TunnelStatus::Connected => ("Connected", Color::Green),
            TunnelStatus::Disconnected => ("Disconnected (auto-retrying)", Color::Red),
        };

        let mut lines = Vec::new();

        let mut line1 = vec![
            Span::styled("Selected: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&item.name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" [{}]", item.item_id()), Style::default().fg(Color::DarkGray)),
            Span::raw(" | "),
            Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
            Span::styled("● ", Style::default().fg(status_color)),
            Span::styled(status_text, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        ];

        let connected_at = record.as_ref().and_then(|r| r.connected_at);

        if let Some(rec) = record {
            line1.push(Span::raw(" | "));
            line1.push(Span::styled(format!("Worker PID: {}", rec.worker_pid), Style::default().fg(Color::DarkGray)));
            if let Some(t_pid) = rec.tunnel_pid {
                line1.push(Span::styled(format!(" (Tunnel PID: {})", t_pid), Style::default().fg(Color::DarkGray)));
            }
            if rec.retry_count > 0 {
                line1.push(Span::styled(format!(" [retries: {}]", rec.retry_count), Style::default().fg(Color::Yellow)));
            }
            if let Some(err) = rec.last_error {
                lines.push(Line::from(line1));
                lines.push(Line::from(vec![
                    Span::styled("Error: ", Style::default().fg(Color::Red)),
                    Span::styled(err, Style::default().fg(Color::Red)),
                ]));
            } else {
                lines.push(Line::from(line1));
            }
        } else {
            lines.push(Line::from(line1));
        }

        let binding_str = if item.is_cloud_sql_proxy() {
            format!("{}:{} -> :{}", item.local_bind, item.local_port, item.remote_port())
        } else {
            format!("{}:{} -> {}:{}", item.local_bind, item.local_port, item.remote_bind, item.remote_port())
        };

        let mut line2 = vec![
            Span::styled("Binding: ", Style::default().fg(Color::DarkGray)),
            Span::styled(binding_str, Style::default().fg(Color::White)),
            Span::raw(" | "),
            Span::styled("Target: ", Style::default().fg(Color::DarkGray)),
            Span::styled(item.host.as_deref().or(item.instance.as_deref()).unwrap_or("-"), Style::default().fg(Color::White)),
        ];
        if status == TunnelStatus::Connected {
            let dur = format_duration(connected_at, current_timestamp());
            line2.push(Span::raw(" | "));
            line2.push(Span::styled("Duration: ", Style::default().fg(Color::DarkGray)));
            line2.push(Span::styled(dur, Style::default().fg(Color::Green)));
        }
        lines.push(Line::from(line2));

        if let Some((ref msg, color)) = app.status_message {
            lines.push(Line::from(Span::styled(msg, Style::default().fg(color))));
        }

        lines
    } else {
        vec![Line::from("No item selected")]
    };

    let footer_block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue));
    let footer_para = Paragraph::new(detail_lines).block(footer_block);
    f.render_widget(footer_para, footer_chunk);

    // 4. Bottom Shortcuts Bar
    let bottom_spans: Vec<Span> = if app.search_active {
        vec![
            Span::raw(" "),
            Span::styled("[type]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" filter query  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[enter/↓]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" focus table  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[esc]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" clear & exit search", Style::default().fg(Color::DarkGray)),
        ]
    } else if !app.search_query.is_empty() {
        vec![
            Span::raw(" "),
            Span::styled("[f]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" search  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[esc]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" clear filter  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[r]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" reload  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[p]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" restart all  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[o]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" stop all  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[q]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" quit", Style::default().fg(Color::DarkGray)),
        ]
    } else {
        vec![
            Span::raw(" "),
            Span::styled("[f]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" search  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[r]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" reload  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[p]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" restart all  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[o]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" stop all  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[q/esc]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" quit", Style::default().fg(Color::DarkGray)),
        ]
    };
    let bottom_block =
        Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(Color::Blue));
    let bottom_para = Paragraph::new(vec![Line::from(bottom_spans)]).block(bottom_block).alignment(Alignment::Left);
    f.render_widget(bottom_para, bottom_chunk);

    // 5. Action Modal
    if let Some(ref modal) = app.modal
        && let Some(item) = app.items.iter().find(|i| i.item_id() == modal.item_id)
    {
        let modal_area = centered_rect(54, 9, size);
        f.render_widget(Clear, modal_area);

        let is_inactive = modal.status == TunnelStatus::Inactive;
        let action_label = if is_inactive { "Start Tunnel" } else { "Stop Tunnel" };
        let action_color = if is_inactive { Color::Green } else { Color::Red };

        let modal_block = Block::default()
            .title(format!(" Action: {} ", item.name))
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(action_color));

        let status_badge = match modal.status {
            TunnelStatus::Inactive => Span::styled("Inactive", Style::default().fg(Color::Rgb(128, 128, 128))),
            TunnelStatus::Connecting => Span::styled("Connecting", Style::default().fg(Color::Yellow)),
            TunnelStatus::Connected => Span::styled("Connected", Style::default().fg(Color::Green)),
            TunnelStatus::Disconnected => Span::styled("Disconnected", Style::default().fg(Color::Red)),
        };

        let btn_action_style = if modal.selected == ModalAction::Execute {
            Style::default().bg(action_color).fg(Color::Black).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(action_color)
        };

        let btn_cancel_style = if modal.selected == ModalAction::Cancel {
            Style::default().bg(Color::White).fg(Color::Black).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let modal_content = vec![
            Line::from(vec![Span::raw("Current Status: "), status_badge]),
            Line::from(""),
            Line::from(vec![
                Span::raw("    "),
                Span::styled(format!(" [ {} ] ", action_label), btn_action_style),
                Span::raw("      "),
                Span::styled(" [ Cancel ] ", btn_cancel_style),
            ]),
            Line::from(""),
            Line::from(Span::styled("[Enter] Confirm   [←/→/Tab] Select   [Esc] Cancel", Style::default().fg(Color::DarkGray))),
        ];

        let modal_para = Paragraph::new(modal_content).block(modal_block).alignment(Alignment::Center);

        f.render_widget(modal_para, modal_area);
    }
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(r.height.saturating_sub(height) / 2), Constraint::Length(height), Constraint::Min(0)])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(r.width.saturating_sub(width) / 2), Constraint::Length(width), Constraint::Min(0)])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_ui_renders_correctly() {
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(false);

        terminal.draw(|f| ui(f, &mut app)).unwrap();
        let buffer = terminal.backend().buffer();
        let rendered = format!("{:?}", buffer);

        assert!(rendered.contains("Tunnels"));
        assert!(rendered.contains("Details"));
        assert!(rendered.contains("[f]"));
        assert!(rendered.contains("search"));
        assert!(rendered.contains("[r]"));
        assert!(rendered.contains("reload"));
        assert!(rendered.contains("[p]"));
        assert!(rendered.contains("restart all"));
        assert!(rendered.contains("[o]"));
        assert!(rendered.contains("stop all"));
        assert!(rendered.contains("[q/esc]"));
        assert!(rendered.contains("quit"));
        if !app.items.is_empty() {
            assert!(rendered.contains("Duration"));
        }
    }

    #[test]
    fn test_modal_rendering() {
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(false);

        app.open_modal();
        assert!(app.modal.is_some());

        terminal.draw(|f| ui(f, &mut app)).unwrap();
        let buffer = terminal.backend().buffer();
        let rendered = format!("{:?}", buffer);

        assert!(rendered.contains("Action:"));
        assert!(rendered.contains("Cancel"));
    }

    #[test]
    fn test_search_mode_and_filtering() {
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new(false);

        // Enable search mode
        app.search_active = true;
        app.search_query.push_str("nonexistent_item_xyz");
        app.clamp_selection();

        assert_eq!(app.filtered_indices().len(), 0);
        assert_eq!(app.table_state.selected(), None);

        terminal.draw(|f| ui(f, &mut app)).unwrap();
        let buffer = terminal.backend().buffer();
        let rendered = format!("{:?}", buffer);

        assert!(rendered.contains("Search"));
        assert!(rendered.contains("nonexistent_item_xyz"));

        // Clear search
        app.search_query.clear();
        app.search_active = false;
        app.clamp_selection();

        assert_eq!(app.filtered_indices().len(), app.items.len());
    }

    #[test]
    fn test_forwarding_column_on_small_width() {
        for width in [112, 95, 80] {
            let backend = TestBackend::new(width, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            let mut app = App::new(false);

            app.items = vec![crate::cli::tunnel::config::TunnelItem {
                id: Some("1".to_string()),
                tunnel_type: "docker-gcloud-tunnel".to_string(),
                name: "db2 prod clog ch".to_string(),
                description: "database v2 production".to_string(),
                host: None,
                local_bind: "127.0.0.1".to_string(),
                local_port: 11011,
                remote_bind: "10.128.0.45".to_string(),
                remote_port: 5432,
                project: Some("prj".to_string()),
                zone: Some("zone".to_string()),
                instance: Some("gcp-prod-v2".to_string()),
                service_account_key: None,
                image: None,
            }];

            terminal.draw(|f| ui(f, &mut app)).unwrap();
            let buffer = terminal.backend().buffer();
            let mut row_str = String::new();
            for y in 0..buffer.area.height {
                let mut line = String::new();
                for x in 0..buffer.area.width {
                    line.push_str(buffer[(x, y)].symbol());
                }
                if line.contains("127.0.0.1:11011") {
                    row_str = line;
                    break;
                }
            }
            assert!(
                row_str.contains("127.0.0.1:11011 -> 10.128.0.45:5432"),
                "Forwarding was truncated at width {}! Row was: {}",
                width,
                row_str
            );
        }
    }

    #[test]
    fn test_minimal_ui_mode() {
        for width in [60, 50, 45] {
            let backend = TestBackend::new(width, 25);
            let mut terminal = Terminal::new(backend).unwrap();
            let mut app = App::new(true);

            app.items = vec![crate::cli::tunnel::config::TunnelItem {
                id: Some("1".to_string()),
                tunnel_type: "docker-gcloud-tunnel".to_string(),
                name: "db2 prod main".to_string(),
                description: "database v2 production".to_string(),
                host: None,
                local_bind: "127.0.0.1".to_string(),
                local_port: 3210,
                remote_bind: "10.181.96.2".to_string(),
                remote_port: 5432,
                project: Some("prj".to_string()),
                zone: Some("zone".to_string()),
                instance: Some("ark".to_string()),
                service_account_key: None,
                image: None,
            }];

            terminal.draw(|f| ui(f, &mut app)).unwrap();
            let buffer = terminal.backend().buffer();

            let mut header_str = String::new();
            let mut row_str = String::new();
            for y in 0..buffer.area.height {
                let mut line = String::new();
                for x in 0..buffer.area.width {
                    line.push_str(buffer[(x, y)].symbol());
                }
                if line.contains("Name") && line.contains("Forwarding") {
                    header_str = line.clone();
                }
                if line.contains("●") && line.contains("127.0.0.1:3210") {
                    row_str = line;
                }
            }

            println!("Minimal UI (width {}):\nHeader: {}\nRow: {}", width, header_str, row_str);

            // Verify header has only minimal columns (Name, Forwarding, Duration)
            assert!(header_str.contains("Name"));
            assert!(header_str.contains("Forwarding"));
            assert!(header_str.contains("Duration"));
            assert!(!header_str.contains("Type"), "Header should not contain Type in minimal mode");
            assert!(!header_str.contains("Target"), "Header should not contain Target in minimal mode");
            assert!(!header_str.contains("Description"), "Header should not contain Description in minimal mode");

            // Verify row shows local forwarding only (127.0.0.1:3210) without remote
            assert!(row_str.contains("127.0.0.1:3210"), "Minimal row should contain local forwarding at width {}", width);
            assert!(!row_str.contains("->"), "Minimal table row should NOT contain '->'");
            assert!(!row_str.contains("10.181.96.2"), "Minimal table row should NOT contain remote bind in table row");
        }
    }
}
