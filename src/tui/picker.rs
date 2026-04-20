// tui/picker.rs — centered interactive selection modal (OpenCode-style)
//
// When active, a floating bordered box appears in the middle of the screen.
// ↑↓ move the cursor, Enter confirms, Esc cancels.
//
// Callers populate `Picker` via `App::open_picker()`.  On confirmation,
// `App::confirm_picker()` dispatches the selected value to the right handler
// based on `PickerKind`.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

// ─── Public types ─────────────────────────────────────────────────────────────

/// A single row in the picker list.
#[derive(Debug, Clone)]
pub struct PickerItem {
    /// Primary label shown in bold/highlighted colour.
    pub label: String,
    /// Secondary annotation shown dim on the right (can be empty).
    pub hint: String,
    /// Opaque value returned to the caller on confirmation.
    pub value: String,
}

impl PickerItem {
    pub fn new(label: impl Into<String>, hint: impl Into<String>, value: impl Into<String>) -> Self {
        PickerItem {
            label: label.into(),
            hint: hint.into(),
            value: value.into(),
        }
    }

    /// Shorthand when label == value and there's no hint.
    pub fn simple(s: impl Into<String> + Clone) -> Self {
        PickerItem::new(s.clone(), "", s)
    }
}

/// What the picker was opened for — drives the confirm handler.
#[derive(Debug, Clone, PartialEq)]
pub enum PickerKind {
    Model,
    Agent,
    Compression,
    Session,   // load <id>
    Skills,    // install <name>
}

/// The picker state lives on `App` and is inactive by default.
#[derive(Debug, Clone)]
pub struct Picker {
    pub active: bool,
    pub kind: PickerKind,
    pub title: String,
    pub items: Vec<PickerItem>,
    pub selected: usize,
    /// Top-of-window row index for scrolling (updated in render).
    pub scroll: usize,
}

impl Picker {
    pub fn inactive() -> Self {
        Picker {
            active: false,
            kind: PickerKind::Model,
            title: String::new(),
            items: Vec::new(),
            selected: 0,
            scroll: 0,
        }
    }

    pub fn open(kind: PickerKind, title: impl Into<String>, items: Vec<PickerItem>) -> Self {
        Picker {
            active: true,
            kind,
            title: title.into(),
            items,
            selected: 0,
            scroll: 0,
        }
    }

    pub fn up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.adjust_scroll(MAX_VISIBLE);
        }
    }

    pub fn down(&mut self) {
        if self.selected + 1 < self.items.len() {
            self.selected += 1;
            self.adjust_scroll(MAX_VISIBLE);
        }
    }

    pub fn selected_item(&self) -> Option<&PickerItem> {
        self.items.get(self.selected)
    }

    /// Adjust `scroll` so `selected` is always in the visible window.
    fn adjust_scroll(&mut self, visible: usize) {
        if visible == 0 { return; }
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + visible {
            self.scroll = self.selected + 1 - visible;
        }
    }
}

// ─── Render ───────────────────────────────────────────────────────────────────

/// Maximum width of the picker as a fraction of the terminal width.
const PICKER_WIDTH_PERCENT: u16 = 60;
/// Maximum number of list rows visible at once (not counting borders/footer).
const MAX_VISIBLE: usize = 12;

/// Compute a centered rect for the picker.
pub fn picker_rect(area: Rect, n_items: usize) -> Rect {
    let width = (area.width * PICKER_WIDTH_PERCENT / 100).max(50);
    // borders(2) + footer(1) + items (capped)
    let visible = n_items.min(MAX_VISIBLE) as u16;
    let height = visible + 2 + 1; // borders + footer hint row
    let height = height.min(area.height.saturating_sub(4));

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect { x, y, width, height }
}

/// Render the picker modal over whatever is beneath.
pub fn render(frame: &mut Frame, area: Rect, picker: &mut Picker) {
    if picker.items.is_empty() { return; }

    let rect = picker_rect(area, picker.items.len());

    // Body height = total height - 2 (borders) - 1 (footer)
    let visible = (rect.height.saturating_sub(3)) as usize;
    let visible = visible.min(MAX_VISIBLE).max(1);
    picker.adjust_scroll(visible);

    // Clear the background so the modal is opaque.
    frame.render_widget(Clear, rect);

    let title = format!(" {} ", picker.title);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
        .border_style(Style::default().fg(Color::Cyan));

    // Build visible items
    let window = picker.items
        .iter()
        .enumerate()
        .skip(picker.scroll)
        .take(visible);

    let items: Vec<ListItem> = window
        .map(|(i, item)| {
            let is_sel = i == picker.selected;

            let label_style = if is_sel {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let hint_style = if is_sel {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let arrow = if is_sel { "▶ " } else { "  " };

            let mut spans = vec![
                Span::styled(arrow, label_style),
                Span::styled(item.label.clone(), label_style),
            ];
            if !item.hint.is_empty() {
                // pad to separate label from hint
                let pad = " ".repeat(2);
                spans.push(Span::styled(pad, hint_style));
                spans.push(Span::styled(item.hint.clone(), hint_style));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    // Scroll indicator suffix
    let scroll_info = if picker.items.len() > visible {
        format!(
            "  {}/{}",
            picker.selected + 1,
            picker.items.len()
        )
    } else {
        String::new()
    };

    // Footer hint — rendered as the last line inside the block
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("  ↑↓ navigate   Enter confirm   Esc cancel{}", scroll_info),
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    // Split the inner area: list rows + footer row
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    if inner.height > 1 {
        let list_area = Rect { height: inner.height - 1, ..inner };
        let foot_area = Rect {
            y: inner.y + inner.height - 1,
            height: 1,
            ..inner
        };
        frame.render_widget(List::new(items), list_area);
        frame.render_widget(footer, foot_area);
    } else {
        frame.render_widget(List::new(items), inner);
    }
}
