use crate::porwarder::StringListSelector;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState},
    DefaultTerminal, TerminalOptions,
};

pub struct TUIStringListSelector {
    terminal: DefaultTerminal,
    state: ListState,
}
impl TUIStringListSelector {
    pub fn inline_view(lines: u16) -> Self {
        let terminal = ratatui::init_with_options(TerminalOptions {
            viewport: ratatui::Viewport::Inline(lines),
        });
        Self {
            terminal,
            state: ListState::default(),
        }
    }
}

impl StringListSelector for TUIStringListSelector {
    fn select(&mut self, title: String, options: Vec<String>) -> Option<(usize, String)> {
        if options.len() == 0 {
            return None;
        }
        let mut index = 0;
        let mut selected: Option<String> = None;
        while selected.is_none() {
            self.state = self.state.clone().with_selected(Some(index));
            self.terminal
                .draw(|frame| {
                    let area = frame.area();

                    let items: Vec<_> = options
                        .iter()
                        .enumerate()
                        .map(|(idx, item)| ListItem::from(format!("{}. {}", idx + 1, item)))
                        .collect();
                    let list = List::new(items)
                        .block(
                            Block::default().borders(Borders::ALL).title(
                                Line::from(format!("{} [{}/{}]", title, index + 1, options.len()))
                                    .left_aligned(),
                            ),
                        )
                        .highlight_symbol("〉")
                        .highlight_spacing(HighlightSpacing::Always)
                        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
                    frame.render_stateful_widget(list, area, &mut self.state);
                })
                .ok()?;
            match event::read().ok()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => {
                        selected = options.get(index).cloned();
                    }
                    KeyCode::Up => {
                        index += options.len() - 1;
                        index %= options.len();
                    }
                    KeyCode::Down => {
                        index += 1;
                        index %= options.len();
                    }
                    KeyCode::Esc => {
                        return None;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        self.terminal
            .draw(|frame| {
                frame.render_widget(Block::new(), frame.area());
            })
            .ok()?;
        Some((index, selected.unwrap()))
    }
}
