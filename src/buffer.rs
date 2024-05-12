#![allow(warnings)]

use crate::handler::{Dir, Mode, Motion, Cmd, Action};
use std::{fmt::Display, borrow::BorrowMut};



pub struct Buffer {
  pub buf: Vec<String>,
  pub row: usize,
  pub col: usize,
}

impl Buffer {
  pub fn empty() -> Self {
    Buffer { buf: vec!["".to_string()], row: 0, col: 0}
  }

  pub fn apply_motion(&mut self, m: Motion, mode: Mode) {
    println!("applying: {}", m);
    let by = match m.buf[1] {
      Cmd::By(n) => n,
      _ => panic!("Motion has no direction:\n\t{}", m),
    };
    let dir = match m.buf[2] {
      Cmd::To(d) => d,
      _ => panic!("Motion has no direction:\n\t{}", m),
    };

    match m.buf[0] {
      Cmd::Verb(Action::Cut) => {
      },
      Cmd::Verb(Action::Move) => {
        self.move_cursor(by, dir, mode)
      }
      _=> {}
    }
    // self.validate_cursor(mode); 
  }


  pub fn move_cursor(&mut self, by: usize, dir: Dir, mode: Mode){
    match dir {
      Dir::U => {
        let upper_len = match self.row >= by {
          true => self.nth_len(self.row-by),
          false => self.nth_len(0)
        };
        if upper_len > 0 {
          if self.col > upper_len - 1{
            self.col = upper_len - mode as usize;
          }
        } else {
          self.col = 0;
        }
        self.row = match self.row >= by { 
          true => self.row-by,
          false => 0,
        };
      },

      Dir::D => {
        let target_len = self.nth_len(self.row+by).saturating_sub(self.last_char_len(self.row+by) * mode as usize);
        println!("target len: {target_len}");
        if target_len > 0 {
          if self.col >= target_len {
            self.col = target_len;
            println!("yes")
          }
        }
        self.row = match self.row + by < self.buf.len() {
          true => self.row + by,
          false => self.buf.len()-1,
        };
      },

      Dir::L => {
        if self.col >= by { 
          self.col -= by;
          while !self.nth(self.row).is_char_boundary(self.col) {
            self.col -= 1
          } 
        } else {
          self.col = 0;
        }
      },
      // --------------------------------------------------------- //
      Dir::R => {
        let right_bound = self.nth_len(self.row).saturating_sub(self.last_char_len(self.row) * mode as usize);

        if self.col + by <= right_bound {
          self.col += by;
          while !self.buf[self.row].is_char_boundary(self.col) {
            println!("stuck in move_cursor(): self.col: {}", self.col);
            self.col += 1
          }
        } else {
          self.col = right_bound
        }
        println!("Right bound: {}\nCursor: {:?}", right_bound, (self.row, self.col));

      }
    }
  }

  fn last_char_len(&self, row: usize) -> usize {
    match self.buf.iter().nth(row) {
      Some(str) => match str.chars().last() {
        Some(c) => c.len_utf8(),
        None => 0,
      },
      None => 0,
    }
  }


  pub fn char_len(&self, row: usize) -> usize {
    match self.buf.iter().nth(self.row) {
      Some(str) => str.chars().count(),
      None => 0
    }
  }

  pub fn is_empty(&self) -> bool {
    self.buf.is_empty()
  }

  pub fn height(&self) -> usize {
    self.buf.len()
  }

  pub fn insert_newline(&mut self) {
    if self.row > self.len(){
      panic!("Invalid cursor")
    }
    if let Some(str) = self.buf[self.row].get(self.col..) {
      self.buf.insert(self.row+1, str.to_string());
      self.buf[self.row].drain(self.col..);
    } else {
      self.buf.insert(self.row, String::new())
    }
    self.col = 0;
    self.row += 1;
  }

  pub fn insert_at_cursor(&mut self, s: &str) {
    self.buf[self.row].insert_str(self.col, s);
    self.col += s.len();
    // retokenize here
  }

  pub fn nth(&self, n: usize) -> &str {
    match self.buf.iter().nth(n) {
      Some(s) => &s,
      None    => ""
    }
  }

  pub fn len(&self) -> usize {
    self.buf.len()
  }

  pub fn nth_len(&self, n: usize) -> usize {
    match self.buf.iter().nth(n) {
      Some(s) => s.len(),
      None => 0,
    }
  }

  pub fn chars_before_cursor(&self) -> usize {
    if let Some(str) = self.buf.iter().nth(self.row) {
      match str.get(0..self.col) {
        Some(s) => s.chars().count(),
        None => 0,
      }
    } else { 0 }
  } 
}

impl Display for Buffer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
   let mut formatted = String::new();
    self.buf.iter().enumerate().for_each(|s| 
      formatted.push_str(&format!("{}: {}\n", s.0, s.1))
    );
    write!(f, "\ncursor: {:?}\n-----------\n{}\n-----------\n", (self.row, self.col), formatted)
  }
}

