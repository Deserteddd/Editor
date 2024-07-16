use std::fmt::Display;


macro_rules! motion {
  ($cmd1:expr, $cmd2:expr, $cmd3:expr) => {
    Motion {
      buf: [$cmd1, $cmd2, $cmd3],
    }
  };
}

pub const MOVE_L:   Motion = motion!(Cmd::Verb(Action::Move), Cmd::By(1), Cmd::ToDir(Dir::L));
pub const MOVE_R:   Motion = motion!(Cmd::Verb(Action::Move), Cmd::By(1), Cmd::ToDir(Dir::R));
pub const MOVE_U:   Motion = motion!(Cmd::Verb(Action::Move), Cmd::By(1), Cmd::ToDir(Dir::U));
pub const MOVE_D:   Motion = motion!(Cmd::Verb(Action::Move), Cmd::By(1), Cmd::ToDir(Dir::D));
pub const CUTBACK:  Motion = motion!(Cmd::Verb(Action::Cut),  Cmd::By(1), Cmd::ToDir(Dir::L));
pub const _NEXTLINE: Motion = motion!(Cmd::Verb(Action::Move), Cmd::By(1), Cmd::ToDir(Dir::D));

pub const MOVE_END: Motion       = motion!(Cmd::Verb(Action::Move), Cmd::ToDest(Dest::Endl),     Cmd::None);
pub const _MOVE_TXT_START: Motion = motion!(Cmd::Verb(Action::Move), Cmd::ToDest(Dest::TxtStart), Cmd::None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
  U,
  D,
  L,
  R,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dest {
  Line,
  TxtStart,
  Endl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
  Move,
  Cut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
  Verb(Action),
  By(usize),
  ToDest(Dest),
  ToDir(Dir),
  None
}

impl Cmd {
  pub fn unwrap_dir(&self) -> Dir {
    match self {
      Cmd::ToDir(d) => *d,
      _ => panic!("Called unwrap_dir() on {:?}", self)
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Motion {
  pub buf: [Cmd; 3],
}

impl Motion {
  pub fn new() -> Self {
    Motion { buf: [Cmd::Verb(Action::Move), Cmd::None, Cmd::None] }
  }
  pub fn from(buf: [Cmd; 3]) -> Self {
    Motion {buf}
  }
  // Check if Motion mutates buffer content
  pub fn is_disruptive(&self) -> bool {
    match self.buf[0] {
      Cmd::Verb(Action::Cut) => true,
      Cmd::Verb(Action::Move) => false,
      _ => panic!("Motion.buff[0] should always be a verb")
    }
  }

  // returned motion should always be [Verb, By, Dir] or [Verb, Dest, None]
  pub fn push(&mut self, c: Option<char>) -> Option<Motion> {
    use Action::*;
    use Cmd::*;
    if self.buf[0] == Cmd::None { panic!("Motion has no verb") }
    if c.is_none() {
      return Option::None
    }
    let c = c.unwrap();
    let mut ready = false;

    if c.is_alphabetic(){ match c {
      'x' => {
        self.buf[0] = Verb(Cut);
        if self.buf[1] == None { self.buf[1] = Cmd::By(1) }
        self.buf[2] = ToDir(Dir::R);
        ready = true
      }
      'l' => {
        if self.buf[1] == None {self.buf[1] = By(1)}
        self.buf[2] = ToDir(Dir::R);
        ready = true
      }
      'h' => {
        if self.buf[1] == None {self.buf[1] = By(1)}
        self.buf[2] = ToDir(Dir::L);
        ready = true
      }
      'k' => {
        if self.buf[1] == None {self.buf[1] = By(1)}
        self.buf[2] = ToDir(Dir::U);
        ready = true
      }
      'j' => {
        if self.buf[1] == None {self.buf[1] = By(1)}
        self.buf[2] = ToDir(Dir::D);
        ready = true
      }
      'd' => {
        if self.buf[0] == Verb(Cut) {
          match self.buf[1] {
            Cmd::None => self.buf[1] = ToDest(Dest::Line),
            Cmd::By(_) => self.buf[2] = ToDir(Dir::D),
            _ => panic!("Motion[1] != None | By(n) ")
          }
          ready = true;
        }
        self.buf[0] = Verb(Cut)
      }
      _ => {}
      }
    }

    if c.is_ascii_punctuation() {
      ready = true;
      match c {
        '_' => {
          self.buf[1] = Cmd::ToDest(Dest::TxtStart);
        },
        _   => ready = false
      }
    }

    if c.is_ascii_digit() {
      if let By(n) = self.buf[1] {
        self.buf[1] = By(n.saturating_mul(10).saturating_add((c as usize) - 48));
      } else if c as u8 != 48 {
        self.buf[1] = By((c as u8 - 48) as usize)
      } else {
        self.buf[1] = ToDest(Dest::Endl);
        ready = true;
      }
    }

    match ready {
      true  => {
        let command = Some(*self);
        self.reset();
        command
      },
      false => Option::None
    }
  }

  fn reset(&mut self) {
    self.buf = [Cmd::Verb(Action::Move), Cmd::None, Cmd::None];
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