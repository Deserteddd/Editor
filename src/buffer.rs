use crate::motion::{Dir, Motion, Cmd, Action, Dest};
use crate::handler::Mode;
use std::fmt::Display;

use crate::motion::MOVE_END; 



#[derive(Debug, Clone)]
pub struct Buffer {
  buf: Vec<String>,
  row: usize,
  byte: usize,
  char: usize,
}

impl Buffer {
  pub fn empty() -> Self {
    Buffer { 
      buf: vec!["".to_string()], 
      row: 0, 
      byte: 0, 
      char: 0,
    }
  }

  pub fn _from_lines(lines: Vec<String>) -> Self {
    Buffer { 
      buf: lines, 
      row: 0, 
      byte: 0, 
      char: 0,
    }
  }

  pub fn set(&mut self, buf: Self) {
    *self = buf
  }

  pub fn apply_motion(&mut self, m: Motion, mode: Mode) {
    let verb = match m.buf[0] {
      Cmd::Verb(v) => v,
      _ => panic!("Motion has no verb\n{}", m)
    };
    match m.buf[1] {
      Cmd::By(n) => self.apply_w_dir(verb, n, m.buf[2].unwrap_dir()),
      Cmd::ToDest(dest) => self.apply_no_dir(verb, dest),
      _ => panic!("Motion[1] is empty\n{}", m),
    };
    if mode == Mode::Edit {
      self.jump_back_if_end();
    }
  }

  fn apply_w_dir(&mut self, verb: Action, by: usize, dir: Dir) {
    match verb {
      Action::Cut => {
        self.remove_at_cursor(by, dir)
      },
      Action::Move => {
        self.move_cursor(by, dir)
      },
    }
  }

  fn apply_no_dir(&mut self, verb: Action, dest: Dest) {
    match verb {
      Action::Cut => match dest {
        // Maybe should make this actually work, but this is how it works in vim.
        Dest::TxtStart => self.delete_lines_down(1),
        Dest::Line     => self.delete_lines_down(1),
        Dest::Endl     => self.remove_at_cursor(usize::MAX, Dir::R),
      },
      Action::Move => match dest {
        Dest::Endl => {
          self.byte = self.nth(self.row).len();
          self.char = self.char_len();
        },
        Dest::TxtStart => {
          self.byte = self.leading_whitespaces(self.row);
          self.char = self.byte;
        }
        _ => panic!("Invalid dest for Action::Move\n{:?}", dest)
      }
    }
  }

  fn remove_at_cursor(&mut self, n: usize, dir: Dir) {
    match dir {
      Dir::L => {
        if self.char == 0 && self.row > 0 {
          let a = std::mem::take(&mut self.buf[self.row]);
          self.row -= 1;
          self.char = self.char_len();
          self.byte = self.byte_len();
          self.buf[self.row].push_str(&a);
          return
        }
        if self.char >= n {
          let end = self.byte;
          self.move_cursor(n, dir);
          self.buf[self.row].replace_range(self.byte..end, "");
        } else {
          self.buf[self.row].replace_range(..self.byte, "");
          self.char = 0;
          self.byte = 0;
        }
      },
      Dir::R => {
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
      Dir::D => {
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

  fn move_cursor(&mut self, by: usize, dir: Dir) { 
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
        if self.char.saturating_add(by) < self.char_len() {
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
          self.byte = self.nth(self.row).len();
          self.char = self.char_len();
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
          None    => self.char_len(),
        };
        self.byte = match self.nth(self.row).char_indices().nth(self.char) {
          Some(n) => n.0,
          None => self.buf[self.row].len()
        };
      },
    }
  }

  fn leading_whitespaces(&self, row: usize) -> usize {
    match self.buf.iter().nth(row) {
      Some(str) => str.chars().take_while(|c| c.is_whitespace()).count(),
      None => 0
    }
  }

  fn char_len(&self) -> usize {
    match self.buf.iter().nth(self.row) {
      Some(str) => str.chars().count(),
      None => 0
    }
  }

  fn byte_len(&self) -> usize {
    match self.buf.iter().nth(self.row) {
      Some(str) => str.len(),
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

  pub fn insert_newline(&mut self, dir: Dir, split: bool) {
    let indent = self.leading_whitespaces(self.row);
    let mut leading = (0..indent)
      .map(|_| " ").collect::<String>();
    match dir {
      Dir::D => {
        if split {
          if let Some(content) = self.nth(self.row).get(self.byte..) {
            leading.push_str(content);
            self.buf.insert(self.row+1, leading);
            self.buf[self.row].drain(self.byte..);
          } else {
            self.buf.insert(self.row+1, leading)
          }
        } else {
          self.buf.insert(self.row+1, leading)
        }
        self.row += 1;
        self.char = indent;
        self.byte = indent;
      },
      Dir::U => {
        self.buf.insert(self.row, leading);
        self.char = indent;
        self.byte = indent;
      },
      _ => panic!("Should insert newline only up or down")
    }
  }

  #[inline]
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

  #[inline]
  pub fn row(&self) -> usize {
    self.row
  }

  #[inline]
  pub fn char_idx(&self) -> usize {
    self.char
  }

  pub fn jump_back_if_end(&mut self) {
    if self.nth(self.row).chars().nth(self.byte).is_none() {
      self.char = self.char.saturating_sub(1);
      self.byte = self.byte.saturating_sub(self.last_char_len());
    }
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

    let longest_line_index: (usize, &String) = self.buf
      .iter()
      .enumerate()
      .max_by_key(|x| x.1.len())
      .unwrap();

    self.nth(longest_line_index.0).chars().enumerate().for_each(|b| {
      if b.0 != self.char {
        formatted.push_str(&format!("{}", b.0%10));
      } else {
        formatted.push('!');
      }
    });

    formatted.push('\n');

    self.buf.iter().enumerate().for_each(|s| 
      formatted.push_str(&format!("{}: {}\n", s.0, s.1))
    );

    write!(f, "\trow: {}, char: {}, byte: {} char: {:?}\n{formatted}",
      self.row, self.char, self.byte, self.char_at_cursor()
    )
  }
}

