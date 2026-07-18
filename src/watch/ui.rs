mod banner;
pub(super) mod format;

use backpack_battles::{Archetype, BattleResult, Hero};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};

use super::{App, BagLayout};
use format::bag_lines;

const BG: Color = Color::Rgb(7, 8, 10);
const SURFACE: Color = Color::Rgb(16, 17, 17);
const BORDER: Color = Color::Rgb(47, 48, 49);
const TEXT: Color = Color::Rgb(249, 249, 249);
const MUTED: Color = Color::Rgb(156, 156, 157);
const BLUE: Color = Color::Rgb(85, 179, 255);
const RED: Color = Color::Rgb(255, 99, 99);
const GREEN: Color = Color::Rgb(95, 201, 146);
const YELLOW: Color = Color::Rgb(255, 188, 51);
const PURPLE: Color = Color::Rgb(195, 125, 255);

pub(super) fn archetype_color(archetype: Archetype) -> Color {
    match archetype {
        Archetype::Aggression => RED,
        Archetype::Defense => BLUE,
        Archetype::Scaling => PURPLE,
        Archetype::Control => YELLOW,
        Archetype::Support => GREEN,
    }
}

pub(super) fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    frame.render_widget(Block::new().style(Style::new().bg(BG)), area);
    if area.width < 72 || area.height < 20 {
        frame.render_widget(
            Paragraph::new("Terminal too small\nResize to at least 72 x 20")
                .alignment(Alignment::Center)
                .style(Style::new().fg(YELLOW).bg(BG)),
            area,
        );
        return;
    }

    let compact = area.width < 96 || area.height < 30;
    let header_height = if compact { 2 } else { 3 };
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(header_height),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(area);
    render_header(frame, app, header);
    render_footer(frame, app, footer, compact);

    if !compact {
        let [heroes, journal] =
            Layout::horizontal([Constraint::Percentage(46), Constraint::Percentage(54)])
                .areas(body);
        let [left, right] = Layout::vertical([Constraint::Fill(1); 2]).areas(heroes);
        render_hero(
            frame,
            app.battle.left_hero(),
            &app.bags[0],
            left,
            BLUE,
            false,
        );
        render_hero(
            frame,
            app.battle.right_hero(),
            &app.bags[1],
            right,
            RED,
            false,
        );
        render_journal(frame, app, journal);
    } else {
        let [heroes, journal] =
            Layout::vertical([Constraint::Length(12), Constraint::Min(0)]).areas(body);
        let [left, right] = Layout::horizontal([Constraint::Fill(1); 2]).areas(heroes);
        render_hero(
            frame,
            app.battle.left_hero(),
            &app.bags[0],
            left,
            BLUE,
            true,
        );
        render_hero(
            frame,
            app.battle.right_hero(),
            &app.bags[1],
            right,
            RED,
            true,
        );
        render_journal(frame, app, journal);
    }
    if let Some(result) = app.result {
        banner::render(frame, result, body);
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let state = app.result.map_or_else(
        || if app.paused { "PAUSED" } else { "RUNNING" },
        |result| result.outcome.name(),
    );
    let line = Line::from(vec![
        Span::styled(" BACKPACK BATTLES ", Style::new().fg(TEXT).bold()),
        Span::styled("// ", Style::new().fg(BORDER)),
        Span::styled(format!("seed {}  ", app.seed), Style::new().fg(MUTED)),
        Span::styled(format!("tick {:03}  ", app.tick), Style::new().fg(TEXT)),
        Span::styled(format!("{:.1}x  ", app.speed), Style::new().fg(BLUE)),
        Span::styled(
            state.to_uppercase(),
            Style::new().fg(status_color(app)).bold(),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(line)
            .block(Block::new().borders(Borders::BOTTOM).border_style(BORDER))
            .style(Style::new().bg(BG)),
        area,
    );
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect, compact: bool) {
    let controls = match (compact, app.result.is_some()) {
        (true, true) => " arrows scroll  home/end jump  q quit ",
        (true, false) => " space pause  +/- speed  arrows scroll  home/end jump  q quit ",
        (false, true) => " up/down scroll  pgup/pgdn page  home/end jump  q quit ",
        (false, false) => {
            " space pause  +/- speed  up/down scroll  pgup/pgdn page  home/end jump  q quit "
        }
    };
    frame.render_widget(
        Paragraph::new(controls)
            .alignment(Alignment::Center)
            .style(Style::new().fg(MUTED).bg(BG)),
        area,
    );
}

fn render_hero(
    frame: &mut Frame,
    hero: &Hero,
    layout: &BagLayout,
    area: Rect,
    side_color: Color,
    compact: bool,
) {
    let title = format!(
        " {}  HP {}/{}  BLOCK {} ",
        hero.name(),
        hero.health(),
        hero.max_health(),
        hero.block()
    );
    let panel = Block::bordered()
        .title(Line::styled(title, Style::new().fg(TEXT).bold()))
        .border_style(side_color)
        .style(Style::new().bg(SURFACE));
    frame.render_widget(panel, area);
    let horizontal_margin = if compact { 1 } else { 2 };
    let inner = area.inner(Margin::new(horizontal_margin, 1));
    let sections = if compact {
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(inner)
    } else {
        Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner)
    };
    let health = sections[0];
    let bag = sections[sections.len() - 1];
    let max_health = hero.max_health().max(1);
    frame.render_widget(
        Gauge::default()
            .ratio(f64::from(hero.health()) / f64::from(max_health))
            .label(format!("{} / {}", hero.health(), max_health))
            .gauge_style(Style::new().fg(health_color(hero)).bg(BORDER).bold()),
        health,
    );
    frame.render_widget(
        Paragraph::new(bag_lines(layout, hero.bag(), compact)).style(Style::new().bg(SURFACE)),
        bag,
    );
}

fn render_journal(frame: &mut Frame, app: &mut App, area: Rect) {
    app.visible_journal_rows = usize::from(area.height.saturating_sub(2)).max(1);
    let max_scroll = app.max_scroll();
    let scroll = if app.follow {
        max_scroll
    } else {
        app.scroll.min(max_scroll)
    };
    let end = (scroll + app.visible_journal_rows).min(app.journal.len());
    let lines = app.journal[scroll..end].to_vec();
    let mode = if app.follow { "FOLLOW" } else { "SCROLL" };
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::bordered()
                    .title(Line::styled(
                        " FIGHT JOURNAL ",
                        Style::new().fg(TEXT).bold(),
                    ))
                    .title_bottom(Line::styled(
                        format!(
                            " {mode}  {}/{} ",
                            scroll.saturating_add(1),
                            app.journal.len()
                        ),
                        Style::new().fg(MUTED),
                    ))
                    .border_style(BORDER),
            )
            .style(Style::new().fg(TEXT).bg(SURFACE)),
        area,
    );
}

fn health_color(hero: &Hero) -> Color {
    match u32::from(hero.health()) * 3 / u32::from(hero.max_health().max(1)) {
        2.. => GREEN,
        1 => YELLOW,
        _ => RED,
    }
}

fn status_color(app: &App) -> Color {
    app.result.map_or(
        if app.paused { YELLOW } else { GREEN },
        status_color_for_result,
    )
}

fn status_color_for_result(result: BattleResult) -> Color {
    match result.outcome {
        backpack_battles::Outcome::LeftWins => BLUE,
        backpack_battles::Outcome::RightWins => RED,
        backpack_battles::Outcome::Draw => YELLOW,
    }
}
