use backpack_battles::BattleResult;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Clear, Paragraph},
};

use super::{MUTED, SURFACE, TEXT, status_color_for_result};

pub(super) fn render(frame: &mut Frame, result: BattleResult, area: Rect) {
    let color = status_color_for_result(result);
    let banner = centered_rect(area, 42, 7);
    frame.render_widget(Clear, banner);
    let lines = vec![
        Line::raw(""),
        Line::styled(
            result.outcome.name().to_uppercase(),
            Style::new().fg(color).bold(),
        ),
        Line::raw(""),
        Line::styled(
            format!(
                "left {}   right {}",
                result.left_health, result.right_health
            ),
            Style::new().fg(TEXT),
        ),
        Line::styled(format!("in {} ticks", result.ticks), Style::new().fg(MUTED)),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .border_style(color)
                    .title(Line::styled(" BATTLE OVER ", Style::new().fg(color).bold())),
            )
            .style(Style::new().bg(SURFACE)),
        banner,
    );
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}
