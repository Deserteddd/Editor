mod handler;
use handler::{EventHandler, Mode};

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
  event_handler: EventHandler,
  ttf_context: Sdl2TtfContext,
  buffer: Buffer,
  font: &'static str,
}

impl App {
  fn new() -> Result<App, String> {
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;

    let window = video_subsys
      .window("The Editor", 800, 600)
      .resizable()
      .opengl()
      .build()
      .map_err(|e| e.to_string())?;

    let canvas = window
      .into_canvas()
      .build()
      .map_err(|e| e.to_string())?;

    Ok(App { 
      canvas: canvas,
      event_handler: EventHandler::new(&sdl_context)?,
      buffer: Buffer::empty(),
      ttf_context: sdl2::ttf::init().map_err(|e| e.to_string())?,
      font: "./Courier_prime.ttf"
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

  fn render_status_bar(&mut self) -> Result<(), String> {
    let size = self.canvas.output_size()?;
    let rect = rect!(0, size.1-20, size.0, 20);
    let texture_creator = self.canvas.texture_creator();
    let mut font = self.ttf_context.load_font(self.font, 16)?;

    self.canvas.set_draw_color(STATUSBAR);
    self.canvas.fill_rect(rect)?;
    font.set_style(sdl2::ttf::FontStyle::NORMAL);

    // Mode
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
    let x = self.buffer.chars_before_cursor();
    let width = match self.event_handler.mode() {
      Mode::Edit => 14,
      Mode::Insert => 3,
    };
    self.canvas.fill_rect(rect!(x*14, self.buffer.row*25, width, 24))?;
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
    for i in 0..self.buffer.height(){
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
    'running: loop {
      match self.event_handler.handle(&mut self.buffer) {
        Ok(None) => {},
        Ok(Some(motion)) => self.buffer.apply_motion(motion, self.event_handler.mode()),
        Err(()) => break 'running,
      }
      self.render()?;
    }
    assert!(self.buffer.buf[self.buffer.row].is_char_boundary(self.buffer.col));
    Ok(())
  }
}


fn main() -> Result<(), String> {
  let mut app = App::new()?;
  app.run()?;
  Ok(())
}
