//------------------------------------------------------
//------------------------------------------------------
// Token
//------------------------------------------------------
//------------------------------------------------------
use std::fmt::Debug;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token {
  kind: TokenKind,
  pos: (usize, usize),
}

impl Token {
  pub fn new(kind: TokenKind, pos: (usize, usize)) -> Self {
    Token {
      kind,
      pos
    }
  }

  pub fn kind(&self) -> TokenKind {
    self.kind
  }

  pub fn position(&self) -> (usize, usize) {
    self.pos
  }
}
//------------------------------------------------------
// Trait impls
//------------------------------------------------------
// impl Display for Token {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//       write!(f, "{}", self.literal)
      
//   }
// }

impl Debug for Token{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(
        f,
        "{:?} at [{}..{}]",
        self.kind(),
        self.position().0,
        self.position().1,
      )
      
  }
}

//------------------------------------------------------
// Helper functions
//------------------------------------------------------
pub fn get_kind(c: char) -> TokenKind {
  match c.len_utf8() {
    1 | 2 => {
      if c.is_whitespace() { 
        match c {
          '\n' => TokenKind::NewLine,
          _    => TokenKind::Whitespace,
        }
      }
      else if c.is_alphabetic() { TokenKind::Word }
      else if c.is_ascii_digit() { TokenKind::Number }
      else { TokenKind::Other }
    },

    4 => TokenKind::Emoji,
    _ => TokenKind::Other
  }
  // println!("{:} => {:?}", c, a);
}

//------------------------------------------------------
// TokenKind
//------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenKind {
  Punctuation,
  Number,
  Word,
  NewLine,
  Whitespace,
  Emoji,
  Other,
}