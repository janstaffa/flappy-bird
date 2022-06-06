use flappy_bird::{
    draw_text, scale_scalar, GameState, Pipe, Player, GRAVITY, HOLE_HEIGHT, SPACE_BETWEEN_PIPES,
};
use rand::Rng;
use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadSurface, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::ttf::FontStyle;
use std::time::Duration;

pub fn main() {
    let mut rng = rand::thread_rng();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    const WIDTH: i32 = 432;
    const HEIGHT: i32 = 768;

    let window_icon = Surface::from_file("assets/favicon.ico").unwrap();
    // Window setup
    let mut window = video_subsystem
        .window("Flappy bird", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();
    window.set_icon(window_icon);

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();

    let background = texture_creator
        .load_texture("assets/sprites/background.png")
        .unwrap();

    let ground = texture_creator
        .load_texture("assets/sprites/base.png")
        .unwrap();
    let ground_query = ground.query();
    let ground_height = scale_scalar(ground_query.height as i32, SCALE);

    let bird_midflap = texture_creator
        .load_texture("assets/sprites/bird-midflap.png")
        .unwrap();
    let bird_upflap = texture_creator
        .load_texture("assets/sprites/bird-upflap.png")
        .unwrap();
    let bird_downflap = texture_creator
        .load_texture("assets/sprites/bird-downflap.png")
        .unwrap();
    let bird_query = bird_midflap.query();
    let bird_height = scale_scalar(bird_query.height as i32, SCALE);
    let bird_width = scale_scalar(bird_query.width as i32, SCALE);

    let pipe = texture_creator
        .load_texture("assets/sprites/pipe.png")
        .unwrap();
    let pipe_query = pipe.query();
    let pipe_width = scale_scalar(pipe_query.width as i32, SCALE);

    let title_screen = texture_creator
        .load_texture("assets/sprites/message.png")
        .unwrap();

    let mut numbers: Vec<Texture> = Vec::with_capacity(10);

    for n in 0..10 {
        let number = texture_creator
            .load_texture(format!("assets/sprites/numbers/{}.png", n))
            .unwrap();
        numbers.push(number);
    }

    let number_width = 40;
    let number_height = 60;

    let game_over_text = texture_creator
        .load_texture("assets/sprites/gameover.png")
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    const FRAMERATE: i32 = 60;
    const SCALE: f32 = 1.5;

    const PLAYER_X: i32 = 80;
    let player_right_edge = PLAYER_X + bird_width;

    let mut moving_layer_x = 0;

    // Initialize the bird and center it
    let mut player = Player::new(
        (HEIGHT - scale_scalar(ground.query().height as i32, SCALE)) / 2 - bird_height / 2,
    );
    let mut game_state = GameState::BeforeStart;

    let mut pipes: Vec<Pipe> = Vec::new();
    pipes.push(Pipe::new(WIDTH * 2, HEIGHT / 2 - HOLE_HEIGHT / 2));

    const MOVING_SPEED: i32 = 2;

    const TITLE_SCREEN_PADDING: i32 = 100;

    let mut animation_frame = 1;

    let mut score = 0;

    let mut last_scored_id: Option<u32> = None;

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();

    let mut font = ttf_context
        .load_font("assets/fonts/FlappybirdyRegular.ttf", 64)
        .unwrap();

    'main_loop: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main_loop,
                Event::KeyDown {
                    keycode: Some(Keycode::Space | Keycode::Up),
                    ..
                } => {
                    if let GameState::BeforeStart = game_state {
                        game_state = GameState::Running;
                        player.velocity = GRAVITY;
                    }
                    if let GameState::Running = game_state {
                        player.jump()
                    }
                }
                _ => {}
            }
        }

        let (w, h) = canvas.window().size();
        let (w, h) = (w as i32, h as i32);

        // Clear the old state
        canvas.set_draw_color(Color::WHITE);
        canvas.clear();

        // Draw the background
        canvas.copy(&background, None, None).unwrap();

        // Draw the pipes

        for p in &pipes {
            if p.x_offset > w {
                continue;
            }

            let top_pipe_rect_src = Rect::new(
                0,
                0,
                pipe_query.width,
                scale_scalar(p.hole_y, 1. / SCALE) as u32,
            );

            let top_pipe_rect_dst = Rect::new(p.x_offset, 0, pipe_width as u32, p.hole_y as u32);

            let bottom_pipe_y = p.hole_y + HOLE_HEIGHT;
            let bottom_pipe_h = (h - ground_height) - bottom_pipe_y;

            let bottom_pipe_rect_src = Rect::new(
                0,
                0,
                pipe_query.width,
                scale_scalar(bottom_pipe_h, 1. / SCALE) as u32,
            );
            let bottom_pipe_rect_dst = Rect::new(
                p.x_offset,
                bottom_pipe_y,
                pipe_width as u32,
                bottom_pipe_h as u32,
            );

            canvas
                .copy_ex(
                    &pipe,
                    top_pipe_rect_src,
                    top_pipe_rect_dst,
                    0.,
                    None,
                    false,
                    true,
                )
                .unwrap();
            canvas
                .copy(&pipe, bottom_pipe_rect_src, bottom_pipe_rect_dst)
                .unwrap();
        }

        // Draw the bird

        let animation = (animation_frame / 10) % 3;
        let mut sprite = match animation {
            1 => &bird_upflap,
            2 => &bird_midflap,
            _ => &bird_downflap,
        };

        // The bird is gliding
        if player.velocity < -5 {
            sprite = &bird_upflap;
        }

        let bird_rect = Rect::new(PLAYER_X, player.y, bird_width as u32, bird_height as u32);

        if let GameState::BeforeStart = game_state {
            let title_screen_rect = Rect::new(
                TITLE_SCREEN_PADDING,
                TITLE_SCREEN_PADDING / 4,
                (w - TITLE_SCREEN_PADDING * 2) as u32,
                (h - ground_height - TITLE_SCREEN_PADDING * 2) as u32,
            );
            canvas.copy(&title_screen, None, title_screen_rect).unwrap();
        }
        canvas
            .copy_ex(
                sprite,
                None,
                bird_rect,
                player.angle as f64,
                None,
                false,
                false,
            )
            .unwrap();

        // Draw the ground
        let ground_rect = Rect::new(
            -moving_layer_x,
            h - ground_height,
            w as u32,
            scale_scalar(ground.query().height as i32, SCALE) as u32,
        );

        canvas.copy(&ground, None, ground_rect).unwrap();

        // Display the current score
        if let GameState::Running = game_state {
            let digits = score
                .to_string()
                .chars()
                .map(|d| d.to_digit(10).unwrap())
                .collect::<Vec<_>>()
                .into_iter();

            let score_start = w / 2 - ((number_width * digits.len() as i32) / 2);
            for (i, d) in digits.enumerate() {
                let number_rect = Rect::new(
                    score_start + (i as i32 * number_width),
                    20,
                    number_width as u32,
                    number_height as u32,
                );

                let number_texture = numbers.get(d as usize).unwrap();

                canvas.copy(&number_texture, None, number_rect).unwrap();
            }
        }
        if moving_layer_x > 0 {
            let ground_rect = Rect::new(
                w - moving_layer_x,
                h - ground_height,
                w as u32,
                scale_scalar(ground.query().height as i32, SCALE) as u32,
            );

            canvas.copy(&ground, None, ground_rect).unwrap();
        }

        // Update the player
        if !matches!(game_state, GameState::BeforeStart) {
            player.update();
        }

        // Check for collision with the ground
        if player.y + bird_query.height as i32 >= h - ground_height {
            player.y = h - ground_height - bird_query.height as i32;
            game_state = GameState::GameOver;
            player.die();
        }

        // Check for collision with any of the pipes
        for p in pipes.iter() {
            // Check if the players x intersects
            if player_right_edge >= p.x_offset && PLAYER_X <= p.x_offset + pipe_width {
                if player.y <= p.hole_y || player.y + bird_height >= p.hole_y + HOLE_HEIGHT {
                    game_state = GameState::GameOver;
                    player.die();
                    break;
                }
                // The player has successfully passed this pipe
                if matches!(last_scored_id, None) || last_scored_id.unwrap() != p.id {
                    score += 1;
                    last_scored_id = Some(p.id);
                }
            }
        }

        //   draw_text(&mut canvas, &font, "hello world", Color::BLACK, 50, 50).unwrap();
        canvas.present();

        // Update game state

        if let GameState::Running | GameState::BeforeStart = game_state {
            if moving_layer_x >= w {
                moving_layer_x = 0;
            } else {
                moving_layer_x += MOVING_SPEED;
            }

            if animation_frame == FRAMERATE {
                animation_frame = 1;
            } else {
                animation_frame += 1;
            }
        }

        if let GameState::Running = game_state {
            if pipes.len() > 0 {
                if pipes.len() < 3 {
                    // Padding from top and bottom for pipe holes
                    let padding = 50;

                    let new_pipe = Pipe::new(
                        pipes.last().unwrap().x_offset + SPACE_BETWEEN_PIPES,
                        rng.gen_range(padding..(h - ground_height - HOLE_HEIGHT - padding) + 1),
                    );
                    pipes.push(new_pipe);
                }
                // If the first pipe has gone off screen completely, remove it
                if pipes.get(0).unwrap().x_offset + pipe_width <= 0 {
                    pipes.remove(0);
                }

                for p in &mut pipes {
                    p.x_offset -= MOVING_SPEED;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(1000 / FRAMERATE as u64));
    }
}
