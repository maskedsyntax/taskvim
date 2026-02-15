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
    widgets::{Block, Borders, List, ListItem, Paragraph},
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
                    match state.mode {
                        Mode::Normal => match key.code {
                            KeyCode::Char('q') => state.running = false,
                            KeyCode::Char('j') => state.move_selection_down(),
                            KeyCode::Char('k') => state.move_selection_up(),
                            KeyCode::Char('i') => state.mode = Mode::Insert,
                            KeyCode::Char(':') => {
                                state.mode = Mode::Command;
                                state.command_buffer.clear();
                            }
                            KeyCode::Char('d') => {
                                // Simple dd for now
                                state.delete_selected_task()?;
                            }
                            _ => {}
                        },
                        Mode::Insert => match key.code {
                            KeyCode::Esc => state.mode = Mode::Normal,
                            KeyCode::Enter => {
                                if !state.command_buffer.is_empty() {
                                    state.add_task(state.command_buffer.clone())?;
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
                                match cmd.as_str() {
                                    "q" => state.running = false,
                                    "wq" => {
                                        state.running = false;
                                    }
                                    _ => {}
                                }
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

    let tasks: Vec<ListItem> = state
        .tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let style = if i == state.selected_index {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(format!("[{}] {}", task.status, task.title)).style(style)
        })
        .collect();

    let task_list = List::new(tasks)
        .block(Block::default().borders(Borders::ALL).title(" Tasks "));
    
    f.render_widget(task_list, chunks[0]);

    let status_bar = match state.mode {
        Mode::Normal => Paragraph::new("-- NORMAL --"),
        Mode::Insert => Paragraph::new(format!("-- INSERT -- {}", state.command_buffer)),
        Mode::Command => Paragraph::new(format!(":{}", state.command_buffer)),
        _ => Paragraph::new(""),
    };
    f.render_widget(status_bar, chunks[1]);
}
