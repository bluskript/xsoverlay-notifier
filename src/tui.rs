use std::{
    io,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Context;
use crossterm::terminal::enable_raw_mode;
use strum::IntoEnumIterator;
use tokio::sync::{watch, RwLock};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerSmartWidget, TuiLoggerWidget, TuiWidgetState};

use crate::config::{NotificationStrategy, NotifierConfig};

pub async fn start_tui(
    tui: &mut Tui,
    config_tx: watch::Sender<NotifierConfig>,
    config_rx: watch::Receiver<NotifierConfig>,
) -> anyhow::Result<()> {
    enable_raw_mode().context("Failed to run terminal in raw mode")?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();
    loop {
        let settings = config_rx.borrow().to_owned();
        terminal.draw(|frame| tui.ui(frame, &settings))?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
    }
}

#[derive(Default)]
pub struct Tui {
    list: ListState,
    logger_state: TuiWidgetState,
}

impl Tui {
    pub fn ui<B: Backend>(&mut self, f: &mut Frame<B>, settings: &NotifierConfig) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(4)].as_ref())
            .split(f.size());

        let items: Vec<ListItem> = NotificationStrategy::iter()
            .map(|strategy| ListItem::new(strategy.to_string()))
            .collect();
        let list_block = Block::default().borders(Borders::ALL).title("Settings");
        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Green),
            )
            .highlight_symbol("> ");
        f.render_widget(
            TuiLoggerWidget::default()
                .block(
                    Block::default()
                        .title("Independent Tui Logger View")
                        .border_style(Style::default().fg(Color::White).bg(Color::Black))
                        .borders(Borders::ALL),
                )
                .output_separator('|')
                .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
                .output_level(Some(TuiLoggerLevelOutput::Long))
                .output_target(false)
                .output_file(false)
                .output_line(false)
                .style(Style::default().fg(Color::White).bg(Color::Black)),
            chunks[0],
        );
        f.render_stateful_widget(list.block(list_block), chunks[1], &mut self.list);
    }
}
