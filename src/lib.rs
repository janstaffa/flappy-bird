use rand::Rng;
use sdl2::{
    pixels::Color,
    rect::Rect,
    render::{Canvas, Texture, TextureQuery},
    ttf::{Font, FontStyle},
    video::Window,
};
use std::cmp;

pub fn scale_scalar(scalar: i32, scale: f32) -> i32 {
    return (scalar as f32 * scale) as i32;
}

pub const HOLE_HEIGHT: i32 = 150;
pub const SPACE_BETWEEN_PIPES: i32 = 300;

#[derive(Debug)]
pub struct Pipe {
    pub id: u32,
    pub x_offset: i32,
    pub hole_y: i32,
}

impl Pipe {
    pub fn new(x_offset: i32, hole_y: i32) -> Self {
        let mut rng = rand::thread_rng();

        Pipe {
            id: rng.gen::<u32>(),
            x_offset,
            hole_y,
        }
    }
}

pub const GRAVITY: i32 = -6;
const JUMP_FORCE: i32 = 12;

#[derive(Debug)]
pub struct Player {
    pub y: i32,
    pub velocity: i32,
    pub angle: i32,
    pub is_alive: bool,
}

impl Player {
    pub fn new(y: i32) -> Self {
        Player {
            y,
            velocity: 0,
            angle: 0,
            is_alive: true,
        }
    }

    pub fn update(&mut self) {
        self.y += -self.velocity;
        if !self.is_alive {
            return;
        }
        if self.velocity > GRAVITY {
            self.velocity -= 1;
        }
        if self.velocity < 0 && self.angle < 45 {
            self.angle = cmp::min(self.angle + 5, 45);
        }
        if self.velocity > 0 && self.angle > -45 {
            self.angle = cmp::max(self.angle - 8, -45);
        }
    }
    pub fn jump(&mut self) {
        if !self.is_alive {
            return;
        }
        self.velocity = JUMP_FORCE;
        self.update();
    }

    pub fn die(&mut self) {
        self.is_alive = false;
        self.angle = 90;
        self.velocity = GRAVITY * 2;
    }
}

pub enum GameState {
    BeforeStart,
    Running,
    GameOver,
}

pub fn draw_text(
    canvas: &mut Canvas<Window>,
    font: &Font,
    text: &str,
    color: Color,
    mut x: i32,
    y: i32,
    center: bool
) -> Result<(), String> {
    let surface = font.render(text).solid(color).map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    let TextureQuery { width, height, .. } = texture.query();

    if center {
        x = x - width as i32 / 2;
    }
    let target = Rect::new(x, y, width, height);

    canvas.copy(&texture, None, target)?;

    return Ok(());
}
