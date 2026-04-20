// input.rs — bordered input bar with placeholder and cursor

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title(" Input ");

    let content = if app.input.is_empty() && !app.slash_mode.active {
        // Placeholder
        Line::from(vec![
            Span::raw("> "),
            Span::styled(
                "Type a message or / for commands...",
                Style::default().fg(Color::DarkGray),
            ),
        ])
    } else {
        Line::from(vec![Span::raw("> "), Span::raw(app.input.clone())])
    };

    let para = Paragraph::new(content).block(block);
    frame.render_widget(para, area);

    // Show cursor only when not in slash popup (slash popup has its own selection)
    if !app.slash_mode.active {
        // area.x + 1 (left border) + 2 ("> " prefix) + cursor position
        let cursor_x = area.x + 1 + 2 + app.input_cursor as u16;
        // area.y + 1 (top border)
        let cursor_y = area.y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}
