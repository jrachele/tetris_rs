use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{KeyCode, KeyMods};
use ggez::event::{self, EventHandler};
use ggez::graphics;
use ggez::graphics::{Color, Rect, Drawable};

use num_enum::TryFromPrimitive;
use rand::Rng;
use std::convert::TryFrom;
use std::time::{Instant, Duration};
use std::process::exit;

fn main() {
    // Make a Context and an EventLoop.
    let (mut ctx, mut event_loop) =
        ContextBuilder::new("Tetris", "Julian Rachele")
            .window_setup(ggez::conf::WindowSetup::default().title("tetris.rs"))
            .window_mode(ggez::conf::WindowMode::default().dimensions(640.0,640.0))
            .build()
            .unwrap();

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object
    // so it can load resources like images during setup.
    let mut tetris = Tetris::new(&mut ctx);

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut tetris) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}

const GRID_SIZE: (usize, usize) = (20, 10);
const UNIT: f32 = 32.0;
const FPS: u8 = 60;

enum Colors {
    CYAN,
    YELLOW,
    PURPLE,
    GREEN,
    RED,
    BLUE,
    ORANGE,
    BACKGROUND,
}

impl Colors {
    pub fn get_color(self) -> Color {
        match self {
            Colors::CYAN => Color::from_rgb(115, 218, 255),
            Colors::YELLOW => Color::from_rgb(255, 255, 54),
            Colors::PURPLE => Color::from_rgb(134, 54, 255),
            Colors::GREEN => Color::from_rgb(158, 255, 54),
            Colors::RED => Color::from_rgb(255, 87, 54),
            Colors::BLUE => Color::from_rgb(74, 54, 255),
            Colors::ORANGE => Color::from_rgb(255, 155, 54),
            Colors::BACKGROUND => Color::from_rgba(180, 202, 237, 128),
        }
    }
}

#[derive(Copy, Clone, Debug, TryFromPrimitive, PartialEq)]
#[repr(i32)]
enum Tetrimonos {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
    BLANK
}

impl Tetrimonos {
    pub fn generate_color(&self) -> Color {
        match self {
            Tetrimonos::I => Colors::CYAN.get_color(),
            Tetrimonos::O => Colors::YELLOW.get_color(),
            Tetrimonos::T => Colors::PURPLE.get_color(),
            Tetrimonos::S => Colors::GREEN.get_color(),
            Tetrimonos::Z => Colors::RED.get_color(),
            Tetrimonos::J => Colors::BLUE.get_color(),
            Tetrimonos::L => Colors::ORANGE.get_color(),
            Tetrimonos::BLANK => Colors::BACKGROUND.get_color()
        }
    }
}

struct Piece {
    tetrimono: Tetrimonos,
    positions: [[(f32, f32); 4]; 4], // Represents relative positions of all blocks in all states
    position: (f32, f32), // relative to origin
    shadow_position: (f32, f32),
    state: usize,
    color: Color,
    environment: Grid,
}

impl Piece {
    pub fn new(grid: Grid) -> Piece {
        let t: Tetrimonos =
            match Tetrimonos::try_from(rand::thread_rng().gen_range(0, 7)) {
                Ok(tetrimonos) => tetrimonos,
                Err(_) => Tetrimonos::BLANK
            };
        let state = rand::thread_rng().gen_range(0, 4);
        let mut p = Piece {
            positions: Piece::generate_positions(&t),
            // The position is always second row, fourth column
            position: (2.0, GRID_SIZE.1 as f32 / 2.0),
            shadow_position: (0.0, 0.0),
            // represents the state of rotation
            state,
            color: t.generate_color(),
            tetrimono: t,
            environment: grid,
        };
        p.calculate_fall_position();
        return p;
    }

    fn generate_positions(t: &Tetrimonos) -> [[(f32, f32); 4]; 4] {
        match t {
            Tetrimonos::I => // States defined in 4x4 block at a (y,x) of (2,2)
                [
                    [(-1.0, -2.0), (-1.0, -1.0), (-1.0, 0.0), (-1.0, 1.0)], // Initial state
                    [(-2.0, 0.0), (-1.0, 0.0), (0.0, 0.0), (1.0, 0.0)], // After 1 right rotation
                    [(0.0, -2.0), (0.0, -1.0), (0.0, 0.0), (0.0, 1.0)], // 180 degrees
                    [(-2.0, -1.0), (-1.0, -1.0), (0.0, -1.0), (1.0, -1.0)], // final rotation
                ],
            Tetrimonos::O =>
                [
                    [(-1.0, -2.0), (-2.0, -2.0), (-1.0, -1.0), (-2.0, -1.0)],
                    [(-1.0, -2.0), (-2.0, -2.0), (-1.0, -1.0), (-2.0, -1.0)],
                    [(-1.0, -2.0), (-2.0, -2.0), (-1.0, -1.0), (-2.0, -1.0)],
                    [(-1.0, -2.0), (-2.0, -2.0), (-1.0, -1.0), (-2.0, -1.0)],
                ],
            Tetrimonos::T =>
                [
                    [(-1.0, -2.0), (-1.0, -1.0), (-2.0, -1.0), (-1.0, 0.0)],
                    [(-2.0, -1.0), (-1.0, -1.0), (0.0, -1.0), (-1.0, 0.0)],
                    [(-1.0, -2.0), (-1.0, -1.0), (0.0, -1.0), (-1.0, 0.0)],
                    [(-1.0, -2.0), (-2.0, -1.0), (-1.0, -1.0), (0.0, -1.0)],
                ],
            Tetrimonos::S =>
                [
                    [(-1.0, -2.0), (-1.0, -1.0), (-2.0, -1.0), (-2.0, 0.0)],
                    [(-2.0, -1.0), (-1.0, -1.0), (-1.0, 0.0), (0.0, 0.0)],
                    [(0.0, -2.0), (0.0, -1.0), (-1.0, -1.0), (-1.0, 0.0)],
                    [(-2.0, -2.0), (-1.0, -2.0), (-1.0, -1.0), (0.0, -1.0)],
                ],
            Tetrimonos::Z =>
                [
                    [(-2.0, -2.0), (-2.0, -1.0), (-1.0, -1.0), (-1.0, 0.0)],
                    [(0.0, -1.0), (-1.0, -1.0), (-1.0, 0.0), (-2.0, 0.0)],
                    [(-1.0, -2.0), (-1.0, -1.0), (0.0, -1.0), (0.0, 0.0)],
                    [(0.0, -2.0), (-1.0, -2.0), (-1.0, -1.0), (-2.0, -1.0)],
                ],
            Tetrimonos::J =>
                [
                    [(-2.0, -2.0), (-1.0, -2.0), (-1.0, -1.0), (-1.0, 0.0)],
                    [(0.0, -1.0), (-1.0, -1.0), (-2.0, -1.0), (-2.0, 0.0)],
                    [(-1.0, -2.0), (-1.0, -1.0), (-1.0, 0.0), (0.0, 0.0)],
                    [(0.0, -2.0), (0.0, -1.0), (-1.0, -1.0), (-2.0, -1.0)],
                ],
            Tetrimonos::L =>
                [
                    [(-1.0, -2.0), (-1.0, -1.0), (-1.0, 0.0), (-2.0, 0.0)],
                    [(-2.0, -1.0), (-1.0, -1.0), (0.0, -1.0), (0.0, 0.0)],
                    [(0.0, -2.0), (-1.0, -2.0), (-1.0, -1.0), (-1.0, 0.0)],
                    [(-2.0, -2.0), (-2.0, -1.0), (-1.0, -1.0), (0.0, -1.0)],
                ],
            Tetrimonos::BLANK =>
                [
                    [(0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)],
                    [(0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)],
                    [(0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)],
                    [(0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)],
                ],
        }
    }

    fn collides_with_environment(&self, x: f32, y: f32, state: usize) -> bool {
        let real_positions =
            self.positions[state].iter().map(|(i, j)| (i+y, j+x));
        for (i,j) in real_positions {
            if i < 0.0 || i >= GRID_SIZE.0 as f32 || j < 0.0 || j >= GRID_SIZE.1 as f32 ||
                self.environment.grid[i as usize][j as usize] != Tetrimonos::BLANK
            {
                return true;
            }
        }
        false
    }

    fn calculate_fall_position(&mut self) {
        let (y0, x) = self.position;
        let mut y: f32 = y0;
        while !self.collides_with_environment(x, y, self.state) {
            y+=1.0;
        }
        y-=1.0;
        self.shadow_position = (y, x);
    }

    fn rotate(&mut self) {
        let prospective_state = (self.state+1)%4;
        let (y, x) = self.position;
        if !self.collides_with_environment(x,y, prospective_state) {
            self.state = prospective_state;
            self.calculate_fall_position();
        }

    }

    fn shift(&mut self, dir: (f32, f32)) {
        let (y, x) = self.position;
        if !self.collides_with_environment(x+dir.1, y+dir.0, self.state) {
            self.position = (y+dir.0, x+dir.1);
            self.calculate_fall_position();
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        for i in 0..4 {
            // Draw the real piece
            let dims = Rect {
                x: (self.position.1 * UNIT) + (self.positions[self.state][i].1 as f32)*UNIT,
                y: (self.position.0 * UNIT) + (self.positions[self.state][i].0 as f32)*UNIT,
                w: UNIT,
                h: UNIT,
            };
            let rect = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                dims,
                self.color
            )?;
            graphics::draw(ctx, &rect, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;


            // Draw the phantom fall position
            let (fall_y, fall_x) = self.shadow_position;
            let fall_dims = Rect {
                x: (fall_x * UNIT) + (self.positions[self.state][i].1 as f32)*UNIT,
                y: (fall_y * UNIT) + (self.positions[self.state][i].0 as f32)*UNIT,
                w: UNIT,
                h: UNIT,
            };
            let fall_rect = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::stroke(4.0),
                fall_dims,
                self.color
            )?;
            graphics::draw(ctx, &fall_rect, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
        }
        Ok(())
    }



}

#[derive(Clone, PartialEq)]
struct Grid {
    grid: Vec<Vec<Tetrimonos>>
}

impl Grid {
    pub fn new() -> Grid {
        Grid {
            grid: vec![vec![Tetrimonos::BLANK; GRID_SIZE.1]; GRID_SIZE.0]
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        for i in 0..GRID_SIZE.0 {
            for j in 0..GRID_SIZE.1 {
                let dims = Rect {
                    x: (j as f32)*UNIT,
                    y: (i as f32)*UNIT,
                    w: UNIT,
                    h: UNIT,
                };
                let draw_mode =
                    if self.grid[i][j] == Tetrimonos::BLANK
                    {graphics::DrawMode::stroke(3.0)}
                    else {graphics::DrawMode::fill()};
                let rect = graphics::Mesh::new_rectangle(
                    ctx,
                    draw_mode,
                    dims,
                    self.grid[i][j].generate_color()
                )?;
                graphics::draw(ctx, &rect, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
            }
        }
        Ok(())
    }

    // Searches each row and removes it if full of non-blank tetrimono blocks
    // returns number of rows removed
    pub fn clean_rows(&mut self) -> usize {
        let mut new_grid: Vec<Vec<Tetrimonos>> = self.grid.clone().into_iter()
            .filter(|x| x.into_iter().any(|y| *y==Tetrimonos::BLANK))
            .collect();
        let num_removed = GRID_SIZE.0 - new_grid.len();
        for _ in 0..num_removed {
            new_grid.insert(0, vec![Tetrimonos::BLANK; GRID_SIZE.1]);
        }
        self.grid = new_grid;
        num_removed
    }
}

struct Level {
    number: i32,
    color: Color
}

impl Level {
    fn new() -> Level {
        Level {
            number: 1,
            color: graphics::BLACK
        }
    }

    fn get_speed(&self) -> u64 {
        // Gets the milliseconds between each single-grid drop
        match self.number {
            1 => 750,
            2 => 670,
            3 => 590,
            4 => 520,
            5 => 440,
            6 => 360,
            7 => 280,
            8 => 200,
            9 => 125,
            10 => 90,
            11 | 12 | 13 => 80,
            14 | 15 | 16 => 60,
            17 | 18 | 19 => 45,
            20..=30 => 30,
            _ => 20
        }
    }
}

struct Tetris {
    // Your state here...
    score: i32,
    total_lines: i32,
    level: Level,
    piece: Piece,
    last_tick: Instant,
    last_tetris: bool,
}

impl Tetris {
    pub fn new(_ctx: &mut Context) -> Tetris {
        // Load/create resources here: images, fonts, sounds, etc.
        Tetris {
            score: 0,
            total_lines: 0,
            level: Level::new(),
            piece: Piece::new(Grid::new()),
            last_tick: Instant::now(),
            last_tetris: false,
        }
    }

    fn assimilate_piece(&mut self) -> usize {
        // This assumes that Piece::calculate_fall_position works correctly
        let (y, x) = self.piece.shadow_position;
        let mut grid = self.piece.environment.clone();
        for i in 0..4 {
            let j = (y + (self.piece.positions[self.piece.state][i].0)) as usize;
            let k = (x + (self.piece.positions[self.piece.state][i].1)) as usize;
            grid.grid[j][k] = self.piece.tetrimono;
        }
        let rows_removed = grid.clean_rows();
        self.piece = Piece::new(grid);
        return rows_removed;
    }

    fn draw_hud(&mut self, ctx: &mut Context) -> GameResult {
        let font = graphics::Font::default();
        let score_display = format!("Score:\n{}", self.score);
        let mut score_text = graphics::Text::new(score_display);
        score_text.set_font(font, graphics::Scale::uniform(32.0));
        score_text.set_bounds(ggez::mint::Point2 {x: 160.0, y: 160.0}, graphics::Align::Center);
        graphics::draw(ctx, &score_text, ggez::graphics::DrawParam::new()
            .dest(ggez::mint::Point2 {x: 400.0, y: 80.0}).color(graphics::WHITE))?;
        let level_display = format!("Level:\n{}", self.level.number);
        let mut level_text = graphics::Text::new(level_display);
        level_text.set_font(font, graphics::Scale::uniform(32.0));
        level_text.set_bounds(ggez::mint::Point2 {x: 160.0, y: 160.0}, graphics::Align::Center);
        graphics::draw(ctx, &level_text, ggez::graphics::DrawParam::new()
            .dest(ggez::mint::Point2 {x: 400.0, y: 160.0}).color(graphics::WHITE))?;
        Ok(())
    }
}

impl EventHandler for Tetris {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // Update code here...
        if Instant::now() - self.last_tick >= Duration::from_millis(self.level.get_speed()) {
            if self.piece.position == self.piece.shadow_position {
                // We need to either assimilate the positions or the game is over
                let (y, x) = self.piece.shadow_position;
                if self.piece.collides_with_environment(x, y, self.piece.state) {
                    // You lose
                    exit(0);
                }
                let rows_reduced = self.assimilate_piece();
                if rows_reduced > 0 {
                    if rows_reduced == 4 && self.last_tetris {
                        self.score += 1200 * self.level.number;
                        self.last_tetris = true;
                    }
                    else if rows_reduced == 4 {
                        self.score += 800 * self.level.number;
                        self.last_tetris = true;
                    }
                    else {
                        self.score += 100 * self.level.number * rows_reduced as i32;
                        self.last_tetris = false;
                    }
                }
                self.total_lines += rows_reduced as i32;
                self.level.number = (self.total_lines / 10) + 1;
            } else {
                self.piece.shift((1.0, 0.0));
            }
            self.last_tick = Instant::now();
        }
        return Ok(());
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        self.piece.environment.draw(ctx)?;
        self.piece.draw(ctx)?;
        self.draw_hud(ctx)?;
        graphics::present(ctx)?;
        ggez::timer::yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Left => self.piece.shift((0.0, -1.0)),
            KeyCode::Right => self.piece.shift((0.0, 1.0)),
            KeyCode::Up => self.piece.rotate(),
            KeyCode::Down => self.piece.shift((1.0, 0.0)),
            KeyCode::Escape => exit(0),
            _ => ()
        }
    }
}
