// chat.rs — chat message types and scrollable chat pane renderer

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::App;

// ─── Message types ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

// ─── Render ─────────────────────────────────────────────────────────────────

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let last_idx = app.messages.len().saturating_sub(1);

    // Build a flat vec of styled Lines from all messages
    let mut lines: Vec<Line> = Vec::new();

    for (i, msg) in app.messages.iter().enumerate() {
        let (prefix, color) = match msg.role {
            MessageRole::User => ("You", Color::Cyan),
            MessageRole::Assistant => ("Everthink", Color::Green),
            MessageRole::System => ("System", Color::DarkGray),
        };

        // Header row: "─── You ───"
        lines.push(Line::from(vec![Span::styled(
            format!("─── {} ───", prefix),
            Style::default().fg(color),
        )]));

        // Content lines — may be multi-line
        let content_lines: Vec<&str> = msg.content.lines().collect();
        let is_streaming_this = app.is_streaming
            && i == last_idx
            && matches!(msg.role, MessageRole::Assistant);

        if content_lines.is_empty() {
            // Empty placeholder while streaming starts
            if is_streaming_this {
                lines.push(Line::from(vec![
                    Span::styled("▊", Style::default().fg(Color::Green)),
                ]));
            }
        } else {
            let last_content = content_lines.len() - 1;
            for (j, content_line) in content_lines.iter().enumerate() {
                if is_streaming_this && j == last_content {
                    // Append streaming cursor to the last content line
                    lines.push(Line::from(vec![
                        Span::raw(content_line.to_string()),
                        Span::styled("▊", Style::default().fg(Color::Green)),
                    ]));
                } else {
                    lines.push(Line::from(content_line.to_string()));
                }
            }
        }

        // Blank spacer between messages
        lines.push(Line::from(""));
    }

    // Scrolling: scroll_offset=0 → show newest (bottom); higher → show older
    let visible_height = area.height.saturating_sub(2) as usize; // 2 = borders
    let total_lines = lines.len();

    let scroll_row: u16 = if total_lines > visible_height {
        let max_scroll = total_lines - visible_height;
        let clamped = app.scroll_offset.min(max_scroll);
        // scroll_offset=0 means pinned to bottom → invert
        (max_scroll - clamped) as u16
    } else {
        0
    };

    let agent_label = format!("{}", app.agent);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Chat [{}] ", agent_label));

    let para = Paragraph::new(lines)
        .block(block)
        .scroll((scroll_row, 0));

    frame.render_widget(para, area);
}
