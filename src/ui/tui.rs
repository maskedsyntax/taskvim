use crate::core::{AppState, Mode};
use crate::error::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Table, Row, Cell},
    Terminal,
};
use std::io;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn run(&mut self, state: &mut AppState) -> Result<()> {
        while state.running {
            self.terminal.draw(|f| ui(f, state))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    // Try to get action from keymap first (Normal, Visual, Stats)
                    if let Some(action) = state.config.keymap.get_action(state.mode, key) {
                        state.handle_action(action)?;
                        if key.code != KeyCode::Char('g') {
                            state.pending_g = false;
                        }
                        continue;
                    }

                    // Fallback to manual handling for Insert/Command or special keys
                    match state.mode {
                        Mode::Normal => match key.code {
                            KeyCode::Char('g') => {
                                if state.pending_g {
                                    state.move_to_top();
                                    state.pending_g = false;
                                } else {
                                    state.pending_g = true;
                                }
                            }
                            KeyCode::Char('t') if state.pending_g => {
                                state.handle_action(crate::core::actions::Action::NextProject)?;
                                state.pending_g = false;
                            }
                            KeyCode::Char('T') if state.pending_g => {
                                state.handle_action(crate::core::actions::Action::PrevProject)?;
                                state.pending_g = false;
                            }
                            KeyCode::Char('z') => {
                                state.pending_z = true;
                            }
                            KeyCode::Char('a') if state.pending_z => {
                                state.handle_action(crate::core::actions::Action::ToggleCollapse)?;
                                state.pending_z = false;
                            }
                            _ => { 
                                state.pending_g = false; 
                                state.pending_z = false;
                            }
                        },
                        Mode::Visual => match key.code {
                            KeyCode::Char('g') => {
                                if state.pending_g {
                                    state.move_to_top();
                                    state.pending_g = false;
                                } else {
                                    state.pending_g = true;
                                }
                            }
                            _ => { state.pending_g = false; }
                        },
                        Mode::Insert => match key.code {
                            KeyCode::Esc => {
                                state.mode = Mode::Normal;
                                state.editing_task_id = None;
                            }
                            KeyCode::Enter => {
                                if !state.command_buffer.is_empty() {
                                    match state.insert_action {
                                        crate::core::state::InsertAction::AddEnd => state.add_task(state.command_buffer.clone())?,
                                        crate::core::state::InsertAction::AddBelow => state.add_task_below(state.command_buffer.clone())?,
                                        crate::core::state::InsertAction::AddAbove => state.add_task_above(state.command_buffer.clone())?,
                                        crate::core::state::InsertAction::Edit => state.commit_edit()?,
                                    }
                                    state.command_buffer.clear();
                                    state.mode = Mode::Normal;
                                }
                            }
                            KeyCode::Char(c) => state.command_buffer.push(c),
                            KeyCode::Backspace => {
                                state.command_buffer.pop();
                            }
                            _ => {}
                        },
                        Mode::Command => match key.code {
                            KeyCode::Esc => state.mode = Mode::Normal,
                            KeyCode::Enter => {
                                let cmd = state.command_buffer.clone();
                                state.execute_command(&cmd)?;
                                state.command_buffer.clear();
                                state.mode = Mode::Normal;
                            }
                            KeyCode::Char(c) => state.command_buffer.push(c),
                            KeyCode::Backspace => {
                                state.command_buffer.pop();
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .unwrap();
        self.terminal.show_cursor().unwrap();
    }
}

fn ui(f: &mut ratatui::Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.size());

    match state.mode {
        Mode::Stats => {
            let total = state.tasks.len();
            let done = state.tasks.iter().filter(|t| t.status == crate::domain::TaskStatus::Done).count();
            let todo = state.tasks.iter().filter(|t| t.status == crate::domain::TaskStatus::Todo).count();
            let doing = state.tasks.iter().filter(|t| t.status == crate::domain::TaskStatus::Doing).count();
            let archived = state.tasks.iter().filter(|t| t.status == crate::domain::TaskStatus::Archived).count();
            
            let completion_rate = if total > 0 {
                (done as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            let stats_text = vec![
                ListItem::new(format!("Total Tasks: {}", total)),
                ListItem::new(format!("Todo: {}", todo)),
                ListItem::new(format!("Doing: {}", doing)),
                ListItem::new(format!("Done: {}", done)),
                ListItem::new(format!("Archived: {}", archived)),
                ListItem::new(format!("Completion Rate: {:.1}%", completion_rate)),
            ];

            let stats_list = List::new(stats_text)
                .block(Block::default().borders(Borders::ALL).title(" Statistics "));
            
            f.render_widget(stats_list, chunks[0]);
        }
        _ => {
            let header_cells = ["ID", "Status", "Priority", "Title", "Project"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
            let header = Row::new(header_cells)
                .style(Style::default().bg(Color::Blue))
                .height(1)
                .bottom_margin(1);

            let rows = state.tasks.iter().enumerate().map(|(i, task)| {
                let is_selected = if state.mode == Mode::Visual {
                    if let Some(anchor) = state.selection_anchor {
                        let start = anchor.min(state.selected_index);
                        let end = anchor.max(state.selected_index);
                        i >= start && i <= end
                    } else {
                        i == state.selected_index
                    }
                } else {
                    i == state.selected_index
                };

                let style = if is_selected {
                    Style::default().bg(Color::DarkGray).fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                
                let id_short = &task.id.to_string()[..8];
                let status = task.status.to_string();
                let priority = task.priority.to_string();
                let title = task.title.clone();
                let project = task.project.clone().unwrap_or_else(|| "-".to_string());

                Row::new(vec![
                    Cell::from(id_short.to_string()),
                    Cell::from(status),
                    Cell::from(priority),
                    Cell::from(title),
                    Cell::from(project),
                ]).style(style)
            });

            let task_table = Table::new(
                rows,
                [
                    Constraint::Length(10),
                    Constraint::Length(10),
                    Constraint::Length(10),
                    Constraint::Percentage(50),
                    Constraint::Percentage(20),
                ],
            )
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(" TaskVim "));
            
            f.render_widget(task_table, chunks[0]);
        }
    }

    let status_bar = match state.mode {
        Mode::Normal => Paragraph::new("-- NORMAL --"),
        Mode::Insert => Paragraph::new(format!("-- INSERT -- {}", state.command_buffer)),
        Mode::Command => Paragraph::new(format!(":{}", state.command_buffer)),
        Mode::Visual => Paragraph::new("-- VISUAL --"),
        Mode::Stats => Paragraph::new("-- STATS --"),
        Mode::Filter => Paragraph::new(format!("-- FILTER -- {}", state.command_buffer)),
    };
    f.render_widget(status_bar, chunks[1]);
}
