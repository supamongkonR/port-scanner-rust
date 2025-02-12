use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

pub fn draw_ui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    results: &[String],
) -> io::Result<()> {
    terminal.draw(|frame| {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(frame.size());

        let header = Paragraph::new(Line::from(vec![Span::styled(
            "📡 Real-Time Port Scanner",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]))
        .block(Block::default().borders(Borders::BOTTOM));

        frame.render_widget(header, layout[0]);

        let items: Vec<ListItem> = results
            .iter()
            .map(|r| {
                ListItem::new(Span::styled(
                    r.clone(),
                    Style::default().fg(Color::LightGreen),
                ))
            })
            .collect();
        let list =
            List::new(items).block(Block::default().title("Open Ports").borders(Borders::ALL));

        frame.render_widget(list, layout[1]);

        let footer = Paragraph::new(Line::from(vec![Span::styled(
            "Press 'q' to exit | Scanning every 3s...",
            Style::default().fg(Color::Gray),
        )]))
        .block(Block::default().borders(Borders::TOP));

        frame.render_widget(footer, layout[2]);
    })?;
    Ok(())
}
