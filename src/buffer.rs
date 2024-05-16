use crate::handler::{Dir, Mode, Motion, Cmd, Action};
use std::fmt::Display;

use crate::handler::MOVE_END;

#[derive(Debug)]
pub struct State {
  rows: (usize, usize),
  content: Vec<String>
}

impl State {
  pub fn empty() -> Self {
    State {
      rows: (0, 1),
      content: vec![String::new()]
    }
  }
}

#[derive(Debug, Clone)]
pub struct Buffer {
  buf: Vec<String>,
  row: usize,
  byte: usize,
  char: usize,
  last_mod_rows: (usize, usize) // (from..to) exclusive
}

impl Buffer {
  pub fn empty() -> Self {
    Buffer { 
      buf: vec!["".to_string()], 
      row: 0, 
      byte: 0, 
      char: 0,
      last_mod_rows: (0, 1)
    }
  }

  
  pub fn apply_motion(&mut self, m: Motion, mode: Mode) -> State {
    println!("Applying motion: {}", m);

    let by = match m.buf[1] {
      Cmd::By(n) => n,
      _ => 0,
    };
    let dir = match m.buf[2] {
      Cmd::To(d) => d,
      _ => panic!("Motion has no direction:\n\t{}", m),
    };

    match m.buf[0] {
      Cmd::Verb(Action::Move) => {
        self.move_cursor(by, dir, mode)
      },
      Cmd::Verb(Action::Cut) => {
        self.last_mod_rows = self.touches_rows(by, dir);
        self.remove_at_cursor(by, dir, mode)
      },
      Cmd::Verb(Action::Replace) => {
        self.delete_lines_down(by)
      }
      _=> {}
    }

    State { 
      rows: self.last_mod_rows,
      content: match self.buf.get(self.last_mod_rows.0..self.last_mod_rows.1) {
        Some(t) => Vec::from(t),
        None => vec![self.nth(0).to_owned()]
      }
    }
  }

  fn touches_rows(&self, by: usize, dir: Dir) -> (usize, usize) {
    match dir {
      Dir::L | Dir::R => (self.row, self.row + 1),
      Dir::U => (by, self.row),
      _ => (0, 0)
    }
  }

  
  pub fn remove_at_cursor(&mut self, n: usize, dir: Dir, mode: Mode) {
    match dir {
      Dir::L => {
        if self.char == 0 && self.row > 0 && mode == Mode::Insert {
          let a = std::mem::take(&mut self.buf[self.row]);
          self.row -= 1;
          self.apply_motion(MOVE_END, mode);
          self.buf[self.row].push_str(&a);
          return
        }
        if self.char >= n {
          let end = self.byte;
          self.move_cursor(n, dir, mode);
          println!("Buffer {}..{}: {:?}", self.byte, end, self.nth(self.row).get(self.byte..end));
          self.buf[self.row].replace_range(self.byte..end, "");
        } else {
          self.buf[self.row].replace_range(..self.byte, "");
          self.char = 0;
          self.byte = 0;
        }
      },
      Dir::R => {
        assert_eq!(mode, Mode::Edit);
        if self.char.saturating_add(n) < self.char_len() {
          let end = self.nth(self.row)
            .char_indices()
            .nth(self.char.saturating_add(n))
            .expect("Invalid end").0;
          self.buf[self.row].replace_range(self.byte..end, "");
        } else {
          self.buf[self.row].replace_range(self.byte.., "");
          self.char = self.char_len().saturating_sub(1);
          self.byte = match self.nth(self.row).char_indices().last() {
            Some(n) => n.0,
            None => 0,
          }
        }
      },
      Dir::Line => {
        self.delete_lines_down(n)
      }
      _      => {}
    }
  }

  fn delete_lines_down(&mut self, n: usize) {
    if n == 0 {
      self.buf[self.row].clear();
      self.byte = 0;
      self.char = 0;
    }
    for i in 0..n {
      println!("Deleting line {i}");
      if self.height() > 1 {
        self.buf.remove(self.row);     
        if self.height() == self.row {
          self.row -= 1;
        }
      } else {
        self.buf[0].clear();
        self.row = 0;
        break
      }
    }
    if self.char > self.char_len() {
      self.apply_motion(MOVE_END, Mode::Edit);
    }
  }

  
  pub fn move_cursor(&mut self, by: usize, dir: Dir, mode: Mode) {
    match dir {
      Dir::L => {
        if self.char >= by {
          self.char -= by;
          self.byte = match self.nth(self.row).char_indices().nth(self.char) {
            Some(n) => n.0,
            None    => 0,
          };

          if self.char == 0 { self.byte = 0 };

        } else {
          self.char = 0;
          self.byte = 0;
        }
      },
      Dir::R => {
        if self.char.saturating_add(by) < self.char_len().saturating_sub(mode as usize) {
          self.byte = self.byte.saturating_add(match self.char_at_cursor() {
            Some(c) => c.len_utf8(),
            None => by,
          });
          self.char = self.char.saturating_add(by);
          while !self.nth(self.row).is_char_boundary(self.byte) {
            println!("loop 66");
            self.byte += 1;
          }
        } else {
          self.byte = self.nth(self.row).len().saturating_sub(self.last_char_len() * mode as usize);
          self.char = self.char_len().saturating_sub(mode as usize);
        }
      }
      Dir::U | Dir::D => {
        let target_idx = match dir {
          Dir::U => self.row.saturating_sub(by),
          Dir::D => if self.row + by < self.height() { 
            self.row+by 
          } else { 
            self.height().saturating_sub(1) 
          },
          _ => panic!("What the fuck")
        };
        self.row = target_idx;
        self.char = match self.nth(self.row).chars().enumerate().nth(self.char) {
          Some(n) => n.0,
          None    => self.char_len().saturating_sub(mode as usize),
        };
        self.byte = match self.nth(self.row).char_indices().nth(self.char) {
          Some(n) => n.0,
          None => self.buf[self.row].len().saturating_sub(self.last_char_len() * mode as usize)
        }
      }
      Dir::Line => {}
    }
  }

  pub fn char_len(&self) -> usize {
    match self.buf.iter().nth(self.row) {
      Some(str) => str.chars().count(),
      None => 0
    }
  }

  pub fn insert_at_cursor(&mut self, s: &str) {
    self.buf
      .iter_mut()
      .nth(self.row)
      .expect("buffer::insert_at_cursor was called while buffer state is invalid")
      .insert_str(self.byte, s);
    self.byte += s.len();
    self.char += s.chars().count();
  }

  pub fn insert_newline(&mut self, dir: Dir) {
    match dir {
      Dir::D => {
        if let Some(content) = self.nth(self.row).get(self.byte..) {
          self.buf.insert(self.row+1, content.to_string());
          self.buf[self.row].drain(self.byte..);
        } else {
          self.buf.insert(self.row+1, String::new())
        }
        self.row += 1;
        self.char = 0;
        self.byte = 0;
      },
      Dir::U => {
        self.buf.insert(self.row, String::new());
        self.char = 0;
        self.byte = 0;
      },
      _ => panic!("Should insert newline only up or down")
    }
  }

  // 
  pub fn nth(&self, n: usize) -> &str {
    self.buf
      .iter()
      .nth(n)
      .expect("buffer::nth called with out of bounds row")
      .as_str()
  }

  pub fn is_empty(&self) -> bool {
    self.buf.is_empty()
  }

  pub fn height(&self) -> usize {
    self.buf.len()
  }

  pub fn row(&self) -> usize {
    self.row
  }

  pub fn char_idx(&self) -> usize {
    self.char
  }

  fn char_at_cursor(&self) -> Option<char> {
    self.buf[self.row].chars().nth(self.char)
  }

  fn last_char_len(&self) -> usize {
    self.nth(self.row).chars().last().unwrap_or(' ').len_utf8()
  }
}

impl Display for Buffer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
   let mut formatted = "   ".to_string();
    self.nth(0).chars().enumerate().for_each(|b| {
      if b.0 != self.char {
        formatted.push_str(&format!("{}", b.0));
      } else {
        formatted.push('!');
      }
    });
    formatted.push('\n');
    self.buf.iter().enumerate().for_each(|s| 
      formatted.push_str(&format!("{}: {}\n", s.0, s.1))
    );
    write!(f, "cursor: row: {}, char: {}, byte: {} char: {:?}\nlast affected rows: {:?}\n{formatted}",
      self.row, self.char, self.byte, self.char_at_cursor(), self.last_mod_rows
    )
  }
}

