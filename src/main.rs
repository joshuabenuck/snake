use rand::{self, Rng};
use sdl2::{self, event::Event, keyboard::Keycode, render::TextureQuery};
use std::time::Duration;

const FPS: u8 = 15;
const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const CELLSIZE: usize = 20;

const WHITE: (u8, u8, u8, u8) = (255, 255, 255, 255);
const BLACK: (u8, u8, u8, u8) = (0, 0, 0, 255);
const RED: (u8, u8, u8, u8) = (255, 0, 0, 255);
const GREEN: (u8, u8, u8, u8) = (0, 255, 0, 255);
const DARKGREEN: (u8, u8, u8, u8) = (0, 155, 0, 255);
const DARKGRAY: (u8, u8, u8, u8) = (40, 40, 40, 255);

const HEAD: usize = 0;

fn run() -> Result<(), String> {
    assert!(
        WIDTH % CELLSIZE == 0,
        "Window width must be a multiple of cell size."
    );
    assert!(
        HEIGHT % CELLSIZE == 0,
        "Window height must be a multiple of cell size."
    );
    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let font = ttf_context.load_font("freesansbold.ttf", 18)?;
    let mut game = Game::new(sdl_context, font)?;
    loop {
        game.run();
        game.show_game_over_screen()
    }
}

struct Game<'ttf> {
    width: usize,
    height: usize,
    cell_width: usize,
    cell_height: usize,
    sdl_context: sdl2::Sdl,
    font: sdl2::ttf::Font<'ttf, 'static>,
    direction: Direction,
    snake_coords: Vec<(usize, usize)>,
    rng: rand::rngs::ThreadRng,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl<'ttf> Game<'ttf> {
    fn new(
        sdl_context: sdl2::Sdl,
        font: sdl2::ttf::Font<'ttf, 'static>,
    ) -> Result<Game<'ttf>, String> {
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Snake", 640, 480)
            .opengl()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

        let cell_width = WIDTH / CELLSIZE;
        let cell_height = HEIGHT / CELLSIZE;
        let mut rng = rand::thread_rng();
        let start_x = rng.gen_range(5, cell_width - 6);
        let start_y = rng.gen_range(5, cell_height - 6);
        let snake_coords = vec![
            (start_x, start_y),
            (start_x - 1, start_y),
            (start_x - 2, start_y),
        ];
        Ok(Game {
            width: 800,
            height: 600,
            direction: Direction::Right,
            sdl_context,
            font,
            snake_coords,
            cell_width,
            cell_height,
            rng,
            canvas,
        })
    }
    fn run(&mut self) -> Result<(), String> {
        let BGCOLOR = BLACK;
        let mut apple = self.get_random_location();
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::Left),
                        ..
                    } => self.direction = Direction::Left,
                    Event::KeyDown {
                        keycode: Some(Keycode::Right),
                        ..
                    } => self.direction = Direction::Right,
                    Event::KeyDown {
                        keycode: Some(Keycode::Up),
                        ..
                    } => self.direction = Direction::Up,
                    Event::KeyDown {
                        keycode: Some(Keycode::Down),
                        ..
                    } => self.direction = Direction::Down,
                    _ => {}
                }
            }

            let head = self.snake_coords[0];
            for coords in self.snake_coords.iter().skip(1) {
                if coords.0 == head.0 && coords.1 == head.1 {
                    return Ok(()); // Game over
                }
            }

            if head.0 == apple.0 && head.1 == apple.1 {
                apple = self.get_random_location();
            } else {
                self.snake_coords.pop();
            }

            let to_prepend = match self.direction {
                Direction::Up => {
                    if head.1 == 0 {
                        return Ok(()); // Game over
                    }
                    (head.0, head.1 - 1)
                }
                Direction::Down => {
                    if head.1 == self.cell_height - 1 {
                        return Ok(()); // Game over
                    }
                    (head.0, head.1 + 1)
                }
                Direction::Left => {
                    if head.0 == 0 {
                        return Ok(()); // Game over
                    }
                    (head.0 - 1, head.1)
                }
                Direction::Right => {
                    if head.0 == self.cell_width - 1 {
                        return Ok(()); // Game over
                    }
                    (head.0 + 1, head.1)
                }
            };
            self.snake_coords.insert(0, to_prepend);
            self.canvas.set_draw_color(BGCOLOR);
            self.canvas.fill_rect(None)?;
            self.draw_grid()?;
            self.draw_snake()?;
            self.draw_apple(apple)?;
            self.draw_score(self.snake_coords.len() - 3)?;
            //self.canvas.clear();
            self.canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS as u32));
        }
        Ok(())
    }

    fn draw_press_key_msg(&mut self) -> Result<(), String> {
        let press_key_surf = self
            .font
            .render("Press a key to play")
            .solid(DARKGRAY)
            .map_err(|e| e.to_string())?;
        let texture_creator = self.canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&press_key_surf)
            .map_err(|e| e.to_string())?;
        let TextureQuery { width, height, .. } = texture.query();
        self.canvas.copy(
            &texture,
            None,
            Some(sdl2::rect::Rect::new(
                (WIDTH - 200) as i32,
                (HEIGHT - 200) as i32,
                width,
                height,
            )),
        )?;
        Ok(())
    }

    fn draw_score(&mut self, score: usize) -> Result<(), String> {
        let press_key_surf = self
            .font
            .render(format!("Score: {}", score).as_str())
            .blended(WHITE)
            .map_err(|e| e.to_string())?;
        let texture_creator = self.canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&press_key_surf)
            .map_err(|e| e.to_string())?;
        let TextureQuery { width, height, .. } = texture.query();
        self.canvas.copy(
            &texture,
            None,
            Some(sdl2::rect::Rect::new(
                (WIDTH - 100) as i32,
                10,
                width,
                height,
            )),
        )?;
        Ok(())
    }

    fn get_random_location(&mut self) -> (usize, usize) {
        (
            self.rng.gen_range(0, self.cell_width),
            self.rng.gen_range(0, self.cell_height - 1),
        )
    }

    fn draw_snake(&mut self) -> Result<(), String> {
        for coord in &self.snake_coords {
            let x = coord.0 * CELLSIZE;
            let y = coord.1 * CELLSIZE;
            self.canvas.set_draw_color(DARKGREEN);
            self.canvas.fill_rect(sdl2::rect::Rect::new(
                x as i32,
                y as i32,
                CELLSIZE as u32,
                CELLSIZE as u32,
            ))?;
            self.canvas.set_draw_color(GREEN);
            self.canvas.fill_rect(sdl2::rect::Rect::new(
                (x + 4) as i32,
                (y + 4) as i32,
                (CELLSIZE - 8) as u32,
                (CELLSIZE - 8) as u32,
            ))?;
        }
        Ok(())
    }

    fn draw_apple(&mut self, coord: (usize, usize)) -> Result<(), String> {
        let x = coord.0 * CELLSIZE;
        let y = coord.1 * CELLSIZE;
        self.canvas.set_draw_color(RED);
        self.canvas.fill_rect(sdl2::rect::Rect::new(
            x as i32,
            y as i32,
            CELLSIZE as u32,
            CELLSIZE as u32,
        ))?;
        Ok(())
    }

    fn draw_grid(&mut self) -> Result<(), String> {
        self.canvas.set_draw_color(DARKGRAY);
        for x in (0..WIDTH).step_by(CELLSIZE) {
            self.canvas
                .draw_line((x as i32, 0), (x as i32, HEIGHT as i32))?;
        }
        for y in (0..HEIGHT).step_by(CELLSIZE) {
            self.canvas
                .draw_line((0, y as i32), (WIDTH as i32, y as i32))?;
        }
        Ok(())
    }

    fn show_game_over_screen(&mut self) {
        println!("Game over!");
        std::process::exit(1);
    }
}

fn main() {
    match run() {
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
        Ok(()) => {}
    }
}
