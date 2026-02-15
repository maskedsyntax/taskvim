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
    widgets::{Block, Borders, Paragraph, Table, Row, Cell},
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
                    let mut handled = true;
                    match state.mode {
                        Mode::Normal => match key.code {
                            KeyCode::Char('q') => state.running = false,
                            KeyCode::Char('j') => state.move_selection_down(),
                            KeyCode::Char('k') => state.move_selection_up(),
                            KeyCode::Char('g') => {
                                if state.pending_g {
                                    state.move_to_top();
                                    state.pending_g = false;
                                } else {
                                    state.pending_g = true;
                                    handled = false;
                                }
                            }
                            KeyCode::Char('G') => state.move_to_bottom(),
                            KeyCode::Char('d') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                                state.page_down();
                            }
                            KeyCode::Char('u') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                                state.page_up();
                            }
                            KeyCode::Char('i') => {
                                state.mode = Mode::Insert;
                                state.insert_action = crate::core::state::InsertAction::AddEnd;
                            }
                            KeyCode::Char('o') => {
                                state.mode = Mode::Insert;
                                state.insert_action = crate::core::state::InsertAction::AddBelow;
                            }
                            KeyCode::Char('O') => {
                                state.mode = Mode::Insert;
                                state.insert_action = crate::core::state::InsertAction::AddAbove;
                            }
                            KeyCode::Char(':') => {
                                state.mode = Mode::Command;
                                state.command_buffer.clear();
                            }
                            KeyCode::Char('d') => {
                                // Simple dd for now
                                state.delete_selected_task()?;
                            }
                            KeyCode::Char('+') | KeyCode::Char('>') => {
                                state.increase_priority()?;
                            }
                            KeyCode::Char('-') | KeyCode::Char('<') => {
                                state.decrease_priority()?;
                            }
                            KeyCode::Enter => {
                                state.cycle_status()?;
                            }
                            KeyCode::Char('v') => {
                                state.mode = Mode::Visual;
                                state.selection_anchor = Some(state.selected_index);
                            }
                            _ => { handled = false; }
                        },
                        Mode::Visual => match key.code {
                            KeyCode::Char('v') | KeyCode::Esc => {
                                state.mode = Mode::Normal;
                                state.selection_anchor = None;
                            }
                            KeyCode::Char('j') => state.move_selection_down(),
                            KeyCode::Char('k') => state.move_selection_up(),
                            KeyCode::Char('d') => {
                                state.delete_selected_task()?;
                                state.mode = Mode::Normal;
                                state.selection_anchor = None;
                            }
                            KeyCode::Char('G') => state.move_to_bottom(),
                            KeyCode::Char('g') => {
                                if state.pending_g {
                                    state.move_to_top();
                                    state.pending_g = false;
                                } else {
                                    state.pending_g = true;
                                    handled = false;
                                }
                            }
                            _ => { handled = false; }
                        },
                        Mode::Insert => match key.code {
                            KeyCode::Esc => state.mode = Mode::Normal,
                            KeyCode::Enter => {
                                if !state.command_buffer.is_empty() {
                                    match state.insert_action {
                                        crate::core::state::InsertAction::AddEnd => state.add_task(state.command_buffer.clone())?,
                                        crate::core::state::InsertAction::AddBelow => state.add_task_below(state.command_buffer.clone())?,
                                        crate::core::state::InsertAction::AddAbove => state.add_task_above(state.command_buffer.clone())?,
                                    }
                                    state.command_buffer.clear();
                                    state.mode = Mode::Normal;
                                }
                            }
                            KeyCode::Char(c) => state.command_buffer.push(c),
                            KeyCode::Backspace => {
                                state.command_buffer.pop();
                            }
                            _ => { handled = false; }
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
                            _ => { handled = false; }
                        },
                        _ => { handled = false; }
                    }

                    if handled && key.code != KeyCode::Char('g') {
                        state.pending_g = false;
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

    let status_bar = match state.mode {
        Mode::Normal => Paragraph::new("-- NORMAL --"),
        Mode::Insert => Paragraph::new(format!("-- INSERT -- {}", state.command_buffer)),
        Mode::Command => Paragraph::new(format!(":{}", state.command_buffer)),
        Mode::Visual => Paragraph::new("-- VISUAL --"),
        Mode::Filter => Paragraph::new(format!("-- FILTER -- {}", state.command_buffer)),
    };
    f.render_widget(status_bar, chunks[1]);
}
