use rand::{self, Rng};
use sdl2::{self, event::Event, keyboard::Keycode, render::TextureQuery};
use std::time::Duration;

const FPS: u8 = 7;
const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const CELLSIZE: usize = 20;

const WHITE: (u8, u8, u8, u8) = (255, 255, 255, 255);
const BLACK: (u8, u8, u8, u8) = (0, 0, 0, 255);
const RED: (u8, u8, u8, u8) = (255, 0, 0, 255);
const GREEN: (u8, u8, u8, u8) = (0, 255, 0, 255);
const DARKGREEN: (u8, u8, u8, u8) = (0, 155, 0, 255);
const DARKGRAY: (u8, u8, u8, u8) = (40, 40, 40, 255);

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
    let font18 = ttf_context.load_font("freesansbold.ttf", 18)?;
    let font100 = ttf_context.load_font("freesansbold.ttf", 100)?;
    let font150 = ttf_context.load_font("freesansbold.ttf", 150)?;
    let mut game = Game::new(sdl_context, font18, font100, font150)?;
    game.show_start_screen()?;
    loop {
        game.run()?;
        game.show_game_over_screen()?;
    }
}

struct Game<'ttf> {
    cell_width: usize,
    cell_height: usize,
    font18: sdl2::ttf::Font<'ttf, 'static>,
    font100: sdl2::ttf::Font<'ttf, 'static>,
    font150: sdl2::ttf::Font<'ttf, 'static>,
    direction: Direction,
    snake_coords: Vec<(usize, usize)>,
    rng: rand::rngs::ThreadRng,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    event_pump: sdl2::EventPump,
}

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl<'ttf> Game<'ttf> {
    fn new(
        sdl_context: sdl2::Sdl,
        font18: sdl2::ttf::Font<'ttf, 'static>,
        font100: sdl2::ttf::Font<'ttf, 'static>,
        font150: sdl2::ttf::Font<'ttf, 'static>,
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
        let rng = rand::thread_rng();
        let event_pump = sdl_context.event_pump().unwrap();
        Ok(Game {
            direction: Direction::Right,
            font18,
            font100,
            font150,
            snake_coords: Vec::new(),
            cell_width,
            cell_height,
            rng,
            canvas,
            event_pump,
        })
    }

    fn run(&mut self) -> Result<(), String> {
        let start_x = self.rng.gen_range(5, self.cell_width - 6);
        let start_y = self.rng.gen_range(5, self.cell_height - 6);
        self.snake_coords = vec![
            (start_x, start_y),
            (start_x - 1, start_y),
            (start_x - 2, start_y),
        ];
        self.direction = Direction::Right;
        let background_color = BLACK;
        let mut apple = self.get_random_location();
        'running: loop {
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::Left),
                        ..
                    } => {
                        if self.direction != Direction::Right {
                            self.direction = Direction::Left;
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Right),
                        ..
                    } => {
                        if self.direction != Direction::Left {
                            self.direction = Direction::Right;
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Up),
                        ..
                    } => {
                        if self.direction != Direction::Down {
                            self.direction = Direction::Up;
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Down),
                        ..
                    } => {
                        if self.direction != Direction::Up {
                            self.direction = Direction::Down;
                        }
                    }
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
            self.canvas.set_draw_color(background_color);
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
            .font18
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

    fn check_for_key_press(&mut self) -> bool {
        let event = self.event_pump.poll_event();
        if event.is_none() {
            return false;
        }
        match event.unwrap() {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                println!("Exiting.");
                std::process::exit(0);
            }
            Event::KeyDown { .. } => {
                return true;
            }
            _ => {}
        }
        return false;
    }

    fn show_start_screen(&mut self) -> Result<(), String> {
        let white_snake = self
            .font100
            .render("Snake!")
            .shaded(WHITE, DARKGREEN)
            .map_err(|e| e.to_string())?;
        let green_snake = self
            .font100
            .render("Snake!")
            .blended(GREEN)
            .map_err(|e| e.to_string())?;
        let mut white_degrees = 0.0;
        let mut green_degrees = 0.0;
        let texture_creator = self.canvas.texture_creator();
        let white_texture = texture_creator
            .create_texture_from_surface(&white_snake)
            .map_err(|e| e.to_string())?;
        let green_texture = texture_creator
            .create_texture_from_surface(&green_snake)
            .map_err(|e| e.to_string())?;
        let TextureQuery {
            width: white_width,
            height: white_height,
            ..
        } = white_texture.query();
        let TextureQuery {
            width: green_width,
            height: green_height,
            ..
        } = green_texture.query();
        loop {
            self.canvas.set_draw_color(BLACK);
            self.canvas
                .fill_rect(sdl2::rect::Rect::new(0, 0, WIDTH as u32, HEIGHT as u32))?;
            self.canvas.copy_ex(
                &white_texture,
                None,
                sdl2::rect::Rect::from_center(
                    ((WIDTH / 2) as i32, (HEIGHT / 2) as i32),
                    white_width,
                    white_height,
                ),
                white_degrees,
                None,
                false,
                false,
            )?;
            self.canvas.copy_ex(
                &green_texture,
                None,
                sdl2::rect::Rect::from_center(
                    ((WIDTH / 2) as i32, (HEIGHT / 2) as i32),
                    green_width,
                    green_height,
                ),
                green_degrees,
                None,
                false,
                false,
            )?;
            self.draw_press_key_msg()?;
            if self.check_for_key_press() {
                return Ok(());
            }
            self.canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS as u32));
            green_degrees += 3.0;
            white_degrees += 7.0;
        }
    }

    fn draw_score(&mut self, score: usize) -> Result<(), String> {
        let press_key_surf = self
            .font18
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

    fn show_game_over_screen(&mut self) -> Result<(), String> {
        let game = self
            .font150
            .render("Game")
            .blended(WHITE)
            .map_err(|e| e.to_string())?;
        let over = self
            .font150
            .render("Over")
            .blended(WHITE)
            .map_err(|e| e.to_string())?;
        let texture_creator = self.canvas.texture_creator();
        let game_texture = texture_creator
            .create_texture_from_surface(&game)
            .map_err(|e| e.to_string())?;
        let over_texture = texture_creator
            .create_texture_from_surface(&over)
            .map_err(|e| e.to_string())?;
        let TextureQuery {
            width: game_width,
            height: game_height,
            ..
        } = game_texture.query();
        let TextureQuery {
            width: over_width,
            height: over_height,
            ..
        } = over_texture.query();
        let mut game_rect = sdl2::rect::Rect::from_center(
            ((WIDTH / 2) as i32, ((HEIGHT / 2) as i32)),
            game_width,
            game_height,
        );
        game_rect.y = 10;
        let mut over_rect = sdl2::rect::Rect::from_center(
            ((WIDTH / 2) as i32, ((HEIGHT / 2) as i32)),
            over_width,
            over_height,
        );
        over_rect.y = game_height as i32 + 10 + 25;
        self.canvas.copy(&game_texture, None, Some(game_rect))?;
        self.canvas.copy(&over_texture, None, Some(over_rect))?;
        self.draw_press_key_msg()?;
        self.canvas.present();
        ::std::thread::sleep(Duration::from_millis(500));
        loop {
            if self.check_for_key_press() {
                println!("Restarting game.");
                self.check_for_key_press();
                return Ok(());
            }
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS as u32));
        }
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
