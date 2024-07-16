//------------------------------------------------------
//------------------------------------------------------
/// Lexer
//------------------------------------------------------
//------------------------------------------------------
use super::token::{Token, get_kind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lexer<'a> {
  source: &'a str,
  byte_cursor: usize,
  char_cursor: usize,
}
impl<'a> Lexer<'a> {
  pub fn new(input: &'a str) -> Result<Lexer, String> {
    Ok(Lexer { 
      source: input, 
      byte_cursor: 0,
      char_cursor: 0,
    })
  }

  fn char_at_cursor(&self) -> Option<char> {
    self.source.chars().nth(self.char_cursor)
  }
}

//------------------------------------------------------
// Trait impls
//------------------------------------------------------
impl Iterator for Lexer<'_> {
  type Item = Token;
  fn next(&mut self) -> Option<Self::Item> {
    if self.byte_cursor >= self.source.len() {
      return None;
    }

    let token_kind = match self.char_at_cursor() {
      Some(c) => get_kind(c),
      None => return None,
    };

    let literal = self.source
      .chars()
      .skip(self.char_cursor)
      .take_while(|c| get_kind(*c) == token_kind)
      .collect::<String>();

    let raw_count = literal.len();
    let char_count = literal.chars().count();

    self.byte_cursor += raw_count;
    self.char_cursor += char_count;
      
    if self.source.get(self.byte_cursor-raw_count..self.byte_cursor).is_some(){
      return Some(Token::new(token_kind, (self.char_cursor-char_count, self.char_cursor-1)));
    }
    None
  }
}

impl<'a> From<&'a str> for Lexer<'a> {
  fn from(value: &'a str) -> Self {
    Lexer::new(value).unwrap()
  }
}

// impl<'a> From<Ref<'a, String>> for Lexer<'a> {
//   fn from(value: Ref<'a, String>) -> Self {
//     Lexer {
//       source: &value,
//       byte_cursor: 0,
//       char_cursor: 0
//     }
//   }
// }