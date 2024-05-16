use std::fmt::Display;
use crate::buffer::Buffer;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::Sdl;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Mod;
use crate::buffer::State;

macro_rules! motion {
  ($cmd1:expr, $cmd2:expr, $cmd3:expr) => {
    Motion {
      buf: [$cmd1, $cmd2, $cmd3],
    }
  };
}

const CUTBACK: Motion = motion!(Cmd::Verb(Action::Cut), Cmd::By(1), Cmd::To(Dir::L));
const NEXTLINE: Motion = motion!(Cmd::Verb(Action::Move), Cmd::By(1), Cmd::To(Dir::D));

const MOVE_L: Motion = motion!(Cmd::Verb(Action::Move), Cmd::By(1), Cmd::To(Dir::L));
const MOVE_R: Motion = motion!(Cmd::Verb(Action::Move), Cmd::By(1), Cmd::To(Dir::R));
const MOVE_U: Motion = motion!(Cmd::Verb(Action::Move), Cmd::By(1), Cmd::To(Dir::U));
const MOVE_D: Motion = motion!(Cmd::Verb(Action::Move), Cmd::By(1), Cmd::To(Dir::D));
pub const MOVE_END: Motion = motion!(Cmd::Verb(Action::Move), Cmd::By(usize::MAX), Cmd::To(Dir::R));

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
  Insert = 0,
  Edit = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
  U,
  D,
  L,
  R,
  Line,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
  Move,
  Cut,
  Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
  Verb(Action),
  By(usize),
  To(Dir),
  None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Motion {
  pub buf: [Cmd; 3],
}

impl Motion {
  fn new() -> Motion {
    Motion { buf: [Cmd::None; 3] }
  }

  fn reset(&mut self) {
    self.buf = [Cmd::None; 3];
  }

  // buf should always be [Verb, By, Destination]
  fn push(&mut self, c: Option<char>) -> Option<Motion> {
    use Action::*;
    use Cmd::*;
    if c.is_none() {
      return Option::None
    }
    let c = c.unwrap();
    let mut ready = false;

    if c.is_alphabetic(){
      match c {
        'x' => {
          self.buf[0] = Verb(Cut);
          if self.buf[1] == Cmd::None { self.buf[1] = Cmd::By(1) }
          self.buf[2] = To(Dir::R);
          ready = true
        }
        'l' => {
          if self.buf[0] == None {self.buf[0] = Verb(Move)}
          if self.buf[1] == None {self.buf[1] = By(1)}
          self.buf[2] = To(Dir::R);
          ready = true
        }
        'h' => {
          if self.buf[0] == None {self.buf[0] = Verb(Move)}
          if self.buf[1] == None {self.buf[1] = By(1)}
          self.buf[2] = To(Dir::L);
          ready = true
        }
        'k' | 'K' => {
          if self.buf[0] == None {self.buf[0] = Verb(Move)}
          if self.buf[1] == None {self.buf[1] = match c.is_uppercase() {
            true => By(usize::MAX),
            false => By(1),
          }}
          self.buf[2] = To(Dir::U);
          ready = true
        }
        'j' | 'J' => {
          if self.buf[0] == None {self.buf[0] = Verb(Move)}
          if self.buf[1] == None {self.buf[1] = match c.is_uppercase() {
            true => By(usize::MAX),
            false => By(1),
          }}
          self.buf[2] = To(Dir::D);
          ready = true
        }
        'c' => {
          if self.buf[0] == Verb(Replace) {
            self.buf[2] = To(Dir::Line);
            if self.buf[1] == None {self.buf[1] = By(0)}
            ready = true
          }
          self.buf[0] = Verb(Replace);
        }
        'd' => {
          if self.buf[0] == Verb(Cut) {
            self.buf[2] = To(Dir::Line);
            if self.buf[1] == None {self.buf[1] = By(1)}
            ready = true
          }
          self.buf[0] = Verb(Cut)
        }
        _ => {}
      }
    }

    if c.is_ascii_punctuation() {
      match c {
        '=' => {
          if self.buf[0] == Cmd::None {
            self.buf[0] = Cmd::Verb(Action::Move);
          }
          self.buf[1] = Cmd::By(usize::MAX);
          self.buf[2] = Cmd::To(Dir::L);
          ready = true
        }
        _   => {}
      }
    }

    if c.is_ascii_digit() {
      if let By(n) = self.buf[1] {
        self.buf[1] = By(n.saturating_mul(10).saturating_add((c as u8 - 48) as usize));
      } else if c as u8 != 48 {
        self.buf[1] = By((c as u8 - 48) as usize)
      } else {
        if self.buf[0] == Cmd::None {
          self.buf[0] = Cmd::Verb(Action::Move)
        }
        self.buf[1] = By(usize::MAX);
        self.buf[2] = To(Dir::R);
        ready = true;
      }
    }

    match ready {
      true  => {
        let command = Some(*self);
        self.buf = [None; 3];
        command
      },
      false => Option::None
    }
  }


}

impl Display for Motion {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?} {}{:?}", 
    self.buf[0], 
    match self.buf[1] {
      Cmd::None => "".to_string(),
      _ => format!("{:?} ", self.buf[1])
    },
    self.buf[2])
  }
}

pub struct EventHandler {
  event_pump: EventPump,
  pub mode: Mode,
  command: Motion,
}

pub enum HandleResult {
  State(State),
  Quit,
  None,
}

impl EventHandler {
  pub fn new(context: &Sdl) -> Result<Self, String> {
    Ok(EventHandler {
      event_pump: context.event_pump()?,
      mode: Mode::Insert,
      command: Motion::new(),
    })
  }

  pub fn get_motion_str(&self) -> String {
    self.command.to_string()
  }

  fn set_mode(&mut self, c: &str, buffer: &mut Buffer) {
    self.command.reset();
    match c {
      "i" => {
        self.mode = Mode::Insert;
      }
      "a" => {
        self.mode = Mode::Insert;
        buffer.move_cursor(1, Dir::R, Mode::Insert)
      }
      "e" => {
        buffer.move_cursor(1, Dir::L, Mode::Edit);
        self.mode = Mode::Edit;
      }
      _ => panic!("Invalid call to handler::set_insert_mode()")
    }
  }

  pub fn handle(&mut self, buffer: &mut Buffer) -> HandleResult {
    let mut result = HandleResult::None;
    if let Some(event) = self.event_pump.poll_event() {
      match event {
        Event::TextInput { text, .. } => {
          if self.mode == Mode::Insert {
            buffer.insert_at_cursor(&text);
          }
          if self.mode == Mode::Edit {
            match text.as_str() {
              "a" => {
                self.set_mode("a", buffer);
                result = HandleResult::None
              },
              "i" => {
                self.set_mode("i", buffer);
                result = HandleResult::None
              },
              "o" => {
                buffer.apply_motion(MOVE_END, Mode::Insert);
                self.set_mode("a", buffer);
                buffer.insert_newline(Dir::D)
              },
              "O" => {
                self.set_mode("i", buffer);
                buffer.insert_newline(Dir::U);
              }
              _ => if let Some(motion) = self.command.push(text.chars().nth(0)) {
                result = HandleResult::State(buffer.apply_motion(motion, self.mode));
                if motion.buf[0] == Cmd::Verb(Action::Replace) {
                  self.set_mode("a", buffer);
                }
              }
            }
          }
        },
        Event::KeyDown { keycode, keymod, .. } => {
          match keycode {
            Some(Keycode::Escape)    => {
              self.set_mode("e", buffer)
            },
            Some(Keycode::Return) => match self.mode {
              Mode::Insert    => buffer.insert_newline(Dir::D),
              Mode::Edit      => result = HandleResult::State(buffer.apply_motion(NEXTLINE, Mode::Edit)),
            },
            Some(Keycode::Backspace) => {
              match self.mode {
                Mode::Insert  => result = HandleResult::State(buffer.apply_motion(CUTBACK, Mode::Insert)),
                Mode::Edit    => result = HandleResult::State(buffer.apply_motion(MOVE_L, Mode::Edit)),
              }
            },
            Some(Keycode::Tab)       => match self.mode { 
              Mode::Insert           => buffer.insert_at_cursor("  "),
              Mode::Edit             => result = HandleResult::State(buffer.apply_motion(MOVE_R, Mode::Edit))
            }
            Some(Keycode::Right)     => result = HandleResult::State(buffer.apply_motion(MOVE_R, self.mode)),
            Some(Keycode::Left)      => result = HandleResult::State(buffer.apply_motion(MOVE_L, self.mode)),
            Some(Keycode::Up)        => result = HandleResult::State(buffer.apply_motion(MOVE_U, self.mode)),
            Some(Keycode::Down)      => result = HandleResult::State(buffer.apply_motion(MOVE_D, self.mode)),
            Some(Keycode::C)         => if keymod.contains(Mod::LCTRLMOD) {result = HandleResult::Quit},

            _ => {}
          }
        },
        Event::Quit { .. } => result = HandleResult::Quit,
        _=> {}
      }

    }
    result
  }

  pub fn mode(&self) -> Mode {
    self.mode
  }
}