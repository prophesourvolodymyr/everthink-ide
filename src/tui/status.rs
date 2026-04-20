// status.rs — single-row status bar at the top of the TUI

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::App;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    // AUDIT badge — yellow when an AUDIT session is active
    let audit_badge: Vec<Span> = if let Some(ref session) = app.audit_session {
        vec![
            Span::styled(
                format!("  AUDIT {}/{}  ", session.progress(), ""),
                Style::default().fg(Color::Black).bg(Color::Yellow),
            ),
            Span::raw("  "),
        ]
    } else {
        vec![]
    };

    // Compression badge — magenta when any compression mode is active
    let compress_badge: Vec<Span> = if let Some(badge) = app.compression_mode.badge() {
        vec![
            Span::styled(
                format!("  {}  ", badge),
                Style::default().fg(Color::Black).bg(Color::Magenta),
            ),
            Span::raw("  "),
        ]
    } else {
        vec![]
    };

    let mut spans = vec![
        Span::styled(
            "  everthink  ",
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ),
        Span::raw("  "),
    ];
    spans.extend(audit_badge);
    spans.extend(compress_badge);
    spans.extend(vec![
        Span::styled(
            format!("model: {}", app.model),
            Style::default().fg(Color::White),
        ),
        Span::raw("  "),
        Span::styled(
            format!("tokens: {}", app.token_count),
            Style::default().fg(Color::White),
        ),
        Span::raw("  "),
        Span::styled(
            format!("agent: {}", app.agent),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw("  "),
        Span::styled(
            "[Tab] cycle agent  [/] commands  [Ctrl+C] quit",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}
