use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::Sdl;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Mod;
use crate::motion::Motion;
use crate::motion::{MOVE_D, MOVE_U, MOVE_L, MOVE_R, CUTBACK};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
  Insert = 0,
  Edit = 1,
}

pub struct EventHandler {
  event_pump: EventPump,
  mode: Mode,
  command: Motion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleResult {
  Motion(Motion),
  Insert,
  Quit,
  SetEditMode,
  None,
  NewlineSplit,
  NewlineNoSplit,
  NewlineUp,
  Undo,
  Redo,
  PrintHistory
}

impl EventHandler {
  pub fn new(context: &Sdl) -> Result<Self, String> {
    Ok(EventHandler {
      event_pump: context.event_pump()?,
      mode: Mode::Edit,
      command: Motion::new(),
    })
  }

  /* 
  Jos palautus vaatii tekstiä ( esim. jos Motion[0] == Insert ), 
  lisätään se  m_buffiin
  */
  pub fn handle(&mut self, m_buff: &mut String) -> HandleResult {
    let mut result = HandleResult::None;
    if let Some(event) = self.event_pump.poll_event() {
      match event {
        Event::TextInput { text, .. } => {

          if self.mode == Mode::Edit {
            match text.as_str() {
              "a" => {
                self.mode = Mode::Insert;
                result = HandleResult::Motion(MOVE_R)
              },
              "i" => {
                self.mode = Mode::Insert;
                result = HandleResult::None
              },
              "o" => {
                self.mode = Mode::Insert;
                result = HandleResult::NewlineNoSplit;
              },
              "O" => {
                self.mode = Mode::Insert;
                result = HandleResult::NewlineUp;
              }
              "u" => {
                result = HandleResult::Undo;
              }
              "U" => {
                result = HandleResult::Redo;
              }
              "H" => {
                result = HandleResult::PrintHistory;
              }
              _ => if let Some(motion) = self.command.push(text.chars().nth(0)) {
                result = HandleResult::Motion(motion);
              }
            }
          } else {
            result = HandleResult::Insert;
            m_buff.push_str(&text)
          }
        },
        Event::KeyDown { keycode, keymod, .. } => {
          match keycode {
            Some(Keycode::Escape)    => {
              result = match self.mode {
                Mode::Edit => HandleResult::Motion(MOVE_L),
                Mode::Insert => {
                  self.mode = Mode::Edit;
                  HandleResult::SetEditMode
                }
              }
            },
            Some(Keycode::Return) => match self.mode {
              Mode::Insert    => result = {
                HandleResult::NewlineSplit
              },
              Mode::Edit      => result = HandleResult::Motion(MOVE_L),
            },
            Some(Keycode::Backspace) => {
              match self.mode {
                Mode::Insert  => result = HandleResult::Motion(CUTBACK),
                Mode::Edit    => result = HandleResult::Motion(MOVE_L),
              }
            },
            Some(Keycode::Tab)       => result = match self.mode { 
              Mode::Insert => {
                m_buff.push_str("  ");
                HandleResult::Insert
              },
              Mode::Edit   => HandleResult::None
            },
            Some(Keycode::Right)     => result = HandleResult::Motion(MOVE_R),
            Some(Keycode::Left)      => result = HandleResult::Motion(MOVE_L),
            Some(Keycode::Up)        => result = HandleResult::Motion(MOVE_U),
            Some(Keycode::Down)      => result = HandleResult::Motion(MOVE_D),
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