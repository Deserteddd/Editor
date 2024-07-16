use std::fmt::Display;
use crate::buffer::token::{Token, TokenKind};
use crate::buffer::lexer::Lexer;
use crate::{handler::Mode, Motion, Cmd, Action, Dir, Dest};


//------------------------------------------------------
// Buffer, Seeker & SeekTarget
//------------------------------------------------------
pub struct Buffer {
  pub content: String,
  pub cursor: usize,
  token_list: Vec<Token>
}

impl Buffer {
  pub fn new(s: &str) -> Self {
    let token_list = Lexer::from(s).collect();
    Buffer { 
      content: s.to_string(),
      cursor: 0,
      token_list
    }
  }

  pub fn row(&self) -> usize {
    self.content
      .char_indices()
      .take_while(|(idx, _)| *idx <= self.cursor)
      .filter(|(_, c)| *c == '\n')
      .count()
  }

  pub fn apply_motion(&mut self, m: Motion, mode: Mode) {
    println!("byte: {}, cursor: {}", self.byte_index(), self.cursor);
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
        for _ in 0..by {
          if dir == Dir::R {
            self.move_cursor(1, dir)
          }
          self.remove_at_cursor()
        }
      },
      Action::Move => {
        self.move_cursor(by, dir)
      },
    }
  }

  fn apply_no_dir(&mut self, verb: Action, dest: Dest) {
    // match verb {
    //   Action::Cut => match dest {
    //     // Maybe should make this actually work, but this is how it works in vim.
    //     Dest::TxtStart => self.delete_lines_down(1),
    //     Dest::Line     => self.delete_lines_down(1),
    //     Dest::Endl     => self.remove_at_cursor(usize::MAX, Dir::R),
    //   },
    //   Action::Move => match dest {
    //     Dest::Endl => {
    //       self.byte = self.nth(self.row).len();
    //       self.char = self.char_len();
    //     },
    //     Dest::TxtStart => {
    //       self.byte = self.leading_whitespaces(self.row);
    //       self.char = self.byte;
    //     }
    //     _ => panic!("Invalid dest for Action::Move\n{:?}", dest)
    //   }
    // }
  }

  fn remove_at_cursor(&mut self) {
    let byte = self.byte_index();
    if !self.content.is_empty() {
      self.content.remove(byte);
    }
    self.cursor = self.cursor.saturating_sub(1);
  }
  
  fn move_cursor(&mut self, by: usize, dir: Dir) {
    self.cursor = match dir {
      Dir::L => self.cursor.saturating_sub(by),
      Dir::R => {
        let r_bound = self.end_of_row();
        if self.cursor + by <= r_bound {
          self.cursor + by
        } else {
          r_bound
        }
      },
      _      => todo!("Move up and down")
    }
  }

  // absolute index of next end of line from cursor pov
  fn end_of_row(&self) -> usize {
    self.content
      .char_indices()
      .skip(self.cursor)
      .take_while(|(_, c)| *c != '\n')
      .last()
      .unwrap_or((self.cursor, ' '))
      .0
  }

  fn jump_back_if_end(&mut self) {
    if self.content.chars().nth(self.cursor+1) == Some('\n') {
      self.cursor = match self.content.char_indices().nth(self.cursor.saturating_sub(1)) {
        Some(c) => c.0,
        None => self.cursor
      };
      println!("Jumped back at end");
    }
  }
  pub fn height(&self) -> usize {
    self.content.lines().count()
  } 

  pub fn seek(&mut self, target: Seek, forwards: bool) {
    self.cursor = match target {
      Seek::Char(c) => match forwards {
        true  => self.seek_next_char(c),
        false => self.seek_prev_char(c)
      },
      Seek::Word => match forwards {
        true  => self.seek_next_word(),
        false => self.seek_prev_word()
      },
    }
  }

  pub fn insert_at_cursor(&mut self, s: &str) {
    println!("Byte index: {}, cursor: {}", self.byte_index(), self.cursor);
    let mut byte = self.content
      .char_indices()
      .take(self.cursor)
      .last()
      .map(|(idx, c)| idx + c.len_utf8().saturating_sub(1))
      .unwrap_or(0);

    if !self.content.is_empty() {
      byte += 1;
    }

    self.content.insert_str(byte, s);
    self.cursor += s.chars().count();
    self.retokenize();
  }

  pub fn set_cursor(&mut self, n: usize) {
    self.cursor = n
  }

  fn seek_next_char(&self, c: char) -> usize {
    self.content
      .chars()
      .enumerate()
      .skip(self.cursor+1)
      .find(|a| a.1 == c)
      .map(|f| f.0)
      .unwrap_or(self.cursor)
  }

  fn seek_prev_char(&self, c: char) -> usize{
    self.content
      .chars()
      .enumerate()
      .take_while(|f| f.0 < self.cursor)
      .filter(|f| f.1 == c)
      .map(|f| f.0)
      .last()
      .unwrap_or(self.cursor)
  }

  fn seek_next_word(&self) -> usize{
    match self.token_list
      .iter()
      .filter(|token| token.kind() == TokenKind::Word)
      .find(|token| token.position().0 > self.cursor) {
        Some(token) => token.position().0,
        None        => self.cursor
      }
  }

  fn seek_prev_word(&self) -> usize {
    todo!()
  }

  fn retokenize(&mut self) {
    self.token_list = Lexer::from(self.content.as_str()).collect()
  }

  fn byte_index(&self) -> usize {
    if let Some(idx) = self.content
      .char_indices()
      .nth(self.cursor.saturating_sub(1)) 
    {
      idx.0
    } else {
      0
    }
  }

  pub fn col(&self) -> usize {
    let mut column = 0;
    for c in self.content.chars().enumerate() {
      if c.1 != '\n' {
        column += 1
      } else {
        column = 0
      }
      if c.0 == self.cursor {
        break
      }
    }
    column
  }

  pub fn line_count(&self) -> usize {
    self.content.chars().filter(|c| *c == '\n').count()
  }

  pub fn nth(&self, n: usize) -> &str {
    self.content.lines().nth(n).unwrap_or("")
  }

  pub fn is_empty(&self) -> bool {
    self.content.is_empty()
  }
}

pub enum Seek{
  Char(char),
  Word,
}

impl Display for Buffer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut formatted = String::new();
    for (idx, c) in self.content
      .chars()
      .enumerate() 
    {
      if self.cursor == idx {
        formatted.push('|')
      }
      formatted.push(c)
    }
    write!(f, "Cursor: {}, \n{}\n", self.cursor, formatted)
  }
}