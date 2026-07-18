mod ui;

use std::{
    io,
    process::ExitCode,
    time::{Duration, Instant},
};

use backpack_battles::{
    BAG_HEIGHT, BAG_WIDTH, Bag, Battle, BattleConfig, BattleResult, BattleUpdate, ItemId, ItemKind,
    TICK_DURATION, demo_heroes,
};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    text::Line,
};

const MIN_SPEED: f64 = 0.1;
const MAX_SPEED: f64 = 100.0;

pub(crate) fn parse_speed(value: &str) -> Result<f64, String> {
    let speed = value
        .parse::<f64>()
        .map_err(|_| "speed must be a number".to_owned())?;
    if speed.is_finite() && (MIN_SPEED..=MAX_SPEED).contains(&speed) {
        Ok(speed)
    } else {
        Err(format!("speed must be between {MIN_SPEED} and {MAX_SPEED}"))
    }
}

pub(crate) fn run(seed: u64, ticks: u16, speed: f64) -> ExitCode {
    let config = match BattleConfig::new(ticks, seed) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    let (left, right) = demo_heroes(seed);
    let mut app = App::new(Battle::new(left, right, config), seed, speed);
    match ratatui::run(|terminal| app.run(terminal)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: terminal UI failed: {error}");
            ExitCode::FAILURE
        }
    }
}

pub(super) struct App {
    pub(super) battle: Battle,
    pub(super) bags: [BagLayout; 2],
    pub(super) journal: Vec<Line<'static>>,
    pub(super) tick: u16,
    pub(super) seed: u64,
    pub(super) speed: f64,
    pub(super) paused: bool,
    pub(super) follow: bool,
    pub(super) scroll: usize,
    pub(super) visible_journal_rows: usize,
    pub(super) result: Option<BattleResult>,
    next_tick: Instant,
}

impl App {
    fn new(battle: Battle, seed: u64, speed: f64) -> Self {
        let bags = [
            BagLayout::new(battle.left_hero().bag()),
            BagLayout::new(battle.right_hero().bag()),
        ];
        Self {
            battle,
            bags,
            journal: vec![ui::format::intro_line(seed)],
            tick: 0,
            seed,
            speed,
            paused: false,
            follow: true,
            scroll: 0,
            visible_journal_rows: 1,
            result: None,
            next_tick: Instant::now() + tick_delay(speed),
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| ui::render(frame, self))?;
            let timeout = if self.paused || self.result.is_some() {
                Duration::from_millis(100)
            } else {
                self.next_tick
                    .saturating_duration_since(Instant::now())
                    .min(Duration::from_millis(50))
            };
            if event::poll(timeout)?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && self.handle_key(key.code)
            {
                return Ok(());
            }
            if !self.paused && self.result.is_none() && Instant::now() >= self.next_tick {
                self.advance();
                self.next_tick = Instant::now() + tick_delay(self.speed);
            }
        }
    }

    fn advance(&mut self) {
        match self.battle.advance() {
            BattleUpdate::Tick(report) => {
                self.tick = report.tick;
                ui::format::append_report(&mut self.journal, &report);
            }
            BattleUpdate::Finished(result) => {
                self.result = Some(result);
                ui::format::append_result(&mut self.journal, result);
            }
        }
    }

    fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char(' ') if self.result.is_none() => self.paused = !self.paused,
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.speed = (self.speed * 2.0).min(MAX_SPEED);
                self.next_tick = Instant::now() + tick_delay(self.speed);
            }
            KeyCode::Char('-') => {
                self.speed = (self.speed / 2.0).max(MIN_SPEED);
                self.next_tick = Instant::now() + tick_delay(self.speed);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.leave_follow();
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.leave_follow();
                self.scroll = self.scroll.saturating_add(1).min(self.max_scroll());
            }
            KeyCode::PageUp => {
                self.leave_follow();
                self.scroll = self.scroll.saturating_sub(self.visible_journal_rows.max(1));
            }
            KeyCode::PageDown => {
                self.leave_follow();
                self.scroll = self
                    .scroll
                    .saturating_add(self.visible_journal_rows.max(1))
                    .min(self.max_scroll());
            }
            KeyCode::Home => {
                self.follow = false;
                self.scroll = 0;
            }
            KeyCode::End => self.follow = true,
            _ => {}
        }
        false
    }

    fn leave_follow(&mut self) {
        if self.follow {
            self.scroll = self.max_scroll();
            self.follow = false;
        }
    }

    pub(super) fn max_scroll(&self) -> usize {
        self.journal.len().saturating_sub(self.visible_journal_rows)
    }
}

fn tick_delay(speed: f64) -> Duration {
    Duration::from_secs_f64(TICK_DURATION.as_secs_f64() / speed)
}

pub(super) struct BagLayout {
    pub(super) cells: Vec<Option<usize>>,
    pub(super) items: Vec<(ItemId, ItemKind)>,
}

impl BagLayout {
    fn new(bag: &Bag) -> Self {
        let mut cells = vec![None; usize::from(BAG_WIDTH) * usize::from(BAG_HEIGHT)];
        let items = bag
            .items()
            .iter()
            .enumerate()
            .map(|(index, item)| {
                for offset in item.shape() {
                    let x = usize::from(item.position().x + offset.x);
                    let y = usize::from(item.position().y + offset.y);
                    cells[y * usize::from(BAG_WIDTH) + x] = Some(index);
                }
                (item.id(), item.kind())
            })
            .collect();
        Self { cells, items }
    }

    pub(super) fn is_alive(&self, index: usize, bag: &Bag) -> bool {
        let id = self.items[index].0;
        bag.items().iter().any(|item| item.id() == id)
    }
}
