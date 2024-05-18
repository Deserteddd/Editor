mod handler;
mod motion;

use handler::{EventHandler, HandleResult, Mode};
use motion::*;

mod buffer;
use buffer::Buffer;

extern crate sdl2;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::Window;
use sdl2::render::{Canvas, TextureQuery};
use sdl2::pixels::Color;
use sdl2::rect::Rect;


const BACKROUND: Color = Color::RGB(25, 25, 25);
const STATUSBAR: Color = Color::RGB(60, 60, 60);
const CURSOR: Color = Color::RGB(180, 180, 180);
const TEXT: Color = Color::RGB(255, 255, 255);
const FONTSIZE: u16 = 24;


macro_rules! rect(
  ($x:expr, $y:expr, $w:expr, $h:expr) => (
    Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
  )
);


struct App {
  canvas: Canvas<Window>,
  font: &'static str,
  ttf_context: Sdl2TtfContext,
  event_handler: EventHandler,
  buffer: Buffer,
}

impl App {
  fn new() -> Result<App, String> {
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;

    let window = video_subsys
      .window("The Editor", 1400, 600)
      .resizable()
      .opengl()
      .build()
      .map_err(|e| e.to_string())?;

    let canvas = window
      .into_canvas()
      // .present_vsync()
      .build()
      .map_err(|e| e.to_string())?;

    Ok(App { 
      canvas: canvas,
      event_handler: EventHandler::new(&sdl_context)?,
      buffer: Buffer::empty(),
      ttf_context: sdl2::ttf::init().map_err(|e| e.to_string())?,
      font: "./Courier_prime.ttf",
    })
  }

  fn render(&mut self) -> Result<(), String> {
    self.canvas.set_draw_color(BACKROUND);
    self.canvas.clear();
    self.render_cursor()?;
    self.render_txt_buffer()?;
    self.render_status_bar()?;
    self.canvas.present();
    Ok(())
  }

  fn _set_buffer(&mut self, buffer: Buffer) {
    self.buffer = buffer;
  }

  fn render_status_bar(&mut self) -> Result<(), String> {
    let size = self.canvas.output_size()?;
    let rect = rect!(0, size.1-20, size.0, 20);
    let texture_creator = self.canvas.texture_creator();
    let mut font = self.ttf_context.load_font(self.font, 16)?;

    self.canvas.set_draw_color(STATUSBAR);
    self.canvas.fill_rect(rect)?;
    font.set_style(sdl2::ttf::FontStyle::NORMAL);

    // MODE
    let surface = font
      .render(&format!("mode: {:?}", self.event_handler.mode()))
      .blended(TEXT)
      .map_err(|e| e.to_string())?;
    let texture = texture_creator
      .create_texture_from_surface(&surface)
      .map_err(|e| e.to_string())?;
    let TextureQuery {width, height, ..} = texture.query();

    self.canvas.copy(&texture, None, rect!(
      rect.x,
      rect.y,
      width, 
      height
    ))?;   

    Ok(())
  }

  fn render_cursor(&mut self) -> Result<(), String> {
    self.canvas.set_draw_color(CURSOR);
    let x = self.buffer.char_idx();
    let width = match self.event_handler.mode() {
      Mode::Edit => 14,
      Mode::Insert => 3,
    };
    self.canvas.fill_rect(rect!(x*14, self.buffer.row()*25, width, 24))?;
    Ok(())
  }

  fn render_txt_buffer(&mut self) -> Result<(), String> {
    if self.buffer.is_empty() {
      return Ok(())
    }
    let texture_creator = self.canvas.texture_creator();
    let mut font = self.ttf_context.load_font(self.font, FONTSIZE)?;
    font.set_style(sdl2::ttf::FontStyle::NORMAL);
    let mut vert_offset = 0;
    for i in 0..self.buffer.height() {
      if self.buffer.nth(i).len() == 0 {
        vert_offset += 25;
        continue;
      }
      let surface = font
        .render(&self.buffer.nth(i))
        .blended(TEXT)
        .map_err(|e| e.to_string())?;
      let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;
      let TextureQuery {width, height, ..} = texture.query();
      let rect = rect!(0, 0, self.canvas.logical_size().0, self.canvas.logical_size().1);

      self.canvas.copy(&texture, None, rect!(
        rect.x,
        rect.y + vert_offset,
        width, 
        height
      ))?;
      vert_offset += 25;

    }
    
    Ok(()) 
  }

  fn run(&mut self) -> Result<(), String> {
    let mut m_buff = String::new();
    'running: loop {
      let result = self.event_handler.handle(&mut m_buff);
      if result != HandleResult::None {
        println!("\nReceived handleResult: {:?}\n", result);
      }
      match result {
        HandleResult::None => {},
        HandleResult::Quit => break 'running,
        HandleResult::Motion(m) => { self.buffer.apply_motion(m, self.event_handler.mode() ); },
        HandleResult::Insert => self.buffer.insert_at_cursor(&m_buff),
        HandleResult::NewlineSplit => self.buffer.insert_newline(Dir::D, true),
        HandleResult::NewlineNoSplit => self.buffer.insert_newline(Dir::D, false),
        HandleResult::NewlineUp => self.buffer.insert_newline(Dir::U, false),
        HandleResult::SetEditMode => self.buffer.jump_back_if_end()
      }
      self.render()?;
      m_buff.clear();
    }
    Ok(())
  }
}

fn main() -> Result<(), String> {
  let mut app = App::new()?;
  app.run()?;
  Ok(())
}