use std::{
    io::Write,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use termion::{
    self, clear, color, cursor,
    event::{Event, MouseEvent},
    input::{MouseTerminal, TermRead},
    raw::IntoRawMode,
    screen::AlternateScreen,
};

pub fn col2fg_str<T: color::Color>(col: T) -> Vec<u8> {
    let mut ret = Vec::new();
    write!(ret, "{}", color::Fg(col)).unwrap();
    ret
}

pub fn col2bg_str<T: color::Color>(col: T) -> Vec<u8> {
    let mut ret = Vec::new();
    write!(ret, "{}", color::Bg(col)).unwrap();
    ret
}

pub struct EveryNSync {
    interval: Duration,
    prevtime: Instant,
}

impl EveryNSync {
    pub fn new(interval: Duration) -> Self {
        EveryNSync {
            interval: interval,
            prevtime: Instant::now(),
        }
    }
    pub fn run(&mut self) -> bool {
        if self.prevtime.elapsed() > self.interval {
            self.prevtime += self.interval;
            true
        } else {
            false
        }
    }
}

impl From<f64> for EveryNSync {
    fn from(interval: f64) -> Self {
        Self::new(Duration::from_secs_f64(interval))
    }
}

struct Syncer {
    interval: Duration,
    prevtime: Instant,
}

impl Syncer {
    fn new(interval: Duration) -> Self {
        Syncer {
            interval: interval,
            prevtime: Instant::now(),
        }
    }
    fn from_fps(fps: f64) -> Self {
        Self::from(1f64 / fps)
    }
    fn sync(&mut self) {
        let dt = self.prevtime.elapsed();
        if self.interval - dt > Duration::new(0, 0) {
            thread::sleep(self.interval - dt);
        }
        self.prevtime += self.interval;
    }
}

impl From<f64> for Syncer {
    fn from(interval: f64) -> Self {
        Self::new(Duration::from_secs_f64(interval))
    }
}

pub trait TerminalGameDynamic {
    fn init(&mut self) {}
    fn input(&mut self, e: Event);
    fn update(&mut self);
    fn render(&mut self, buff: &mut Vec<u8>);
    fn running(&self) -> bool;
    fn fps(&self) -> f64;
    fn start(&mut self) {
        let mut stdout = AlternateScreen::from(cursor::HideCursor::from(
            MouseTerminal::from(std::io::stdout().into_raw_mode().unwrap()),
        ));
        let stdin = std::io::stdin();
        //
        let (tx, rx) = mpsc::channel();
        //
        thread::spawn(move || {
            let mut mouse = false;
            for e in stdin.events() {
                if let Ok(e) = e {
                    let e = match e {
                        Event::Mouse(MouseEvent::Press(_, _, _)) => {
                            mouse = true;
                            Some(e)
                        }
                        Event::Mouse(MouseEvent::Release(_, _)) => {
                            mouse = false;
                            Some(e)
                        }
                        _ => {
                            if mouse {
                                None
                            } else {
                                Some(e)
                            }
                        }
                    };
                    if let Some(e) = e {
                        tx.send(e).unwrap();
                    }
                }
            }
        });
        //
        writeln!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
        //
        self.init();
        //
        let buff = &mut Vec::new();
        let mut syncer = Syncer::from_fps(self.fps());
        //
        while self.running() {
            for e in rx.try_iter() {
                self.input(e);
            }
            //
            self.update();
            self.render(buff);
            //
            stdout.write_all(buff).unwrap();
            buff.clear();
            stdout.flush().unwrap();
            //
            syncer.sync();
        }
        writeln!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
        stdout.flush().unwrap();
    }
}

pub trait TerminalGameStatic {
    fn init(&mut self) {}
    fn update(&mut self, e: Event, buff: &mut Vec<u8>);
    fn running(&self) -> bool;
    fn start(&mut self) {
        let mut stdout = AlternateScreen::from(cursor::HideCursor::from(
            MouseTerminal::from(std::io::stdout().into_raw_mode().unwrap()),
        ));
        let stdin = std::io::stdin();
        //
        writeln!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
        //
        self.init();
        //
        let buff = &mut Vec::new();
        //
        self.update(Event::Unsupported(Vec::new()), buff);
        stdout.write_all(buff).unwrap();
        buff.clear();
        stdout.flush().unwrap();
        //
        let mut mouse = false;
        //
        for e in stdin.events() {
            if let Ok(e) = e {
                if let Some(e) = match e {
                    Event::Mouse(MouseEvent::Press(_, _, _)) => {
                        mouse = true;
                        Some(e)
                    }
                    Event::Mouse(MouseEvent::Release(_, _)) => {
                        mouse = false;
                        Some(e)
                    }
                    _ => {
                        if mouse {
                            None
                        } else {
                            Some(e)
                        }
                    }
                } {
                    self.update(e, buff);
                    stdout.write_all(buff).unwrap();
                    buff.clear();
                    stdout.flush().unwrap();
                }
            }
            if !self.running() {
                break;
            }
        }
    }
}

pub trait GameObject {
    fn input(&mut self, _: &Event) {}
    fn update(&mut self) {}
    fn render(&mut self, _: &mut Vec<u8>) {}
}

