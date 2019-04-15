extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};

use std::collections::LinkedList;
use std::iter::FromIterator;

#[derive(Clone, PartialEq)]
enum Direction{
    Right, 
    Left, 
    Up, 
    Down,
}

struct Game{
    gl:GlGraphics,
    rows: u32,
    cols: u32,
    snek: Snek,
    ate: bool,
    square_width: u32,
    target: Meal,
    score: u32,
}

impl Game{
    fn render(&mut self, arg: &RenderArgs){
        use graphics;

        let blue: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

        self.gl.draw(arg.viewport(), |_c, gl|{
            graphics::clear(blue, gl);
        });

        self.snek.render(&mut self.gl, arg);
        self.target.render(&mut self.gl, arg, self.square_width)
    }

    fn update(&mut self, args: &UpdateArgs) -> bool {
        if !self.snek.update(self.ate, self.cols, self.rows) {
            return false;
        }

        if self.ate {
            self.score += 1;
            self.ate = false;
        }

        self.ate = self.target.update(&self.snek);
        if self.ate {
            use rand::Rng;
            use rand::thread_rng;
            // try my luck
            let mut r = thread_rng();
            loop {
                let new_x = r.gen_range(0, self.cols);
                let new_y = r.gen_range(0, self.rows);
                if !self.snek.is_collide(new_x, new_y) {
                    self.target = Meal { x: new_x, y: new_y };
                    break;
                }
            }
        }
        true
    }

    fn pressed(&mut self, btn: &Button){
        let last_face = self.snek.facing.clone();

        self.snek.facing = match btn{
            &Button::Keyboard(Key::Up)
                if last_face != Direction::Down => Direction::Up,
            &Button::Keyboard(Key::Down)
                if last_face != Direction::Up => Direction::Down,
            &Button::Keyboard(Key::Right)
                if last_face != Direction::Left => Direction::Right,
            &Button::Keyboard(Key::Left)
                if last_face != Direction::Right => Direction::Left,
            _ => last_face
        };
    }
}

struct Snek{
    gl: GlGraphics,
    body: LinkedList<BodyPiece>,
    facing: Direction,
    width: u32,
}

#[derive(Clone)]
pub struct BodyPiece(u32, u32);

impl Snek{
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs){
        use graphics;

        let red: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let squares: Vec<graphics::types::Rectangle> = self.body
            .iter()
            .map(|p| BodyPiece(p.0 * self.width, p.1 * self.width))
            .map(|p| graphics::rectangle::square(p.0 as f64, p.1 as f64, self.width as f64))
            .collect();

        gl.draw(args.viewport(), |c, gl|{
            let transform = c.transform;
            squares.into_iter()
                .for_each(|square|graphics::rectangle(red, square, transform, gl));
        });
    }

    /// Move the snake if valid, otherwise returns false.
    pub fn update(&mut self, just_eaten: bool, cols: u32, rows: u32) -> bool {
        let mut new_front: BodyPiece =
            (*self.body.front().expect("No front of snake found.")).clone();

        if (self.facing == Direction::Up && new_front.1 == 0)
            || (self.facing == Direction::Left && new_front.0 == 0)
            || (self.facing == Direction::Down && new_front.1 == rows - 1)
            || (self.facing == Direction::Right && new_front.0 == cols - 1)
        {
            return false;
        }

        match self.facing {
            Direction::Up => new_front.1 -= 1,
            Direction::Down => new_front.1 += 1,
            Direction::Left => new_front.0 -= 1,
            Direction::Right => new_front.0 += 1,
        }

        if !just_eaten {
            self.body.pop_back();
        }

        // Checks self collision.
        if self.is_collide(new_front.0, new_front.1) {
            return false;
        }

        self.body.push_front(new_front);
        true
    }

    fn is_collide(&self, x: u32, y: u32) -> bool {
        self.body.iter().any(|p| x == p.0 && y == p.1)
    }
}

pub struct Meal {
    x: u32,
    y: u32,
}

impl Meal {
    // Return true if snek ate a meal this update
    fn update(&mut self, s: &Snek) -> bool {
        let front = s.body.front().unwrap();
        if front.0 == self.x && front.1 == self.y {
            true
        } else {
            false
        }
    }

    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, width: u32) {
        use graphics;

        const BLACK: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

        let x = self.x * width;
        let y = self.y * width;

        let square = graphics::rectangle::square(x as f64, y as f64, width as f64);

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            graphics::rectangle(BLACK, square, transform, gl)
        });
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    const COLS: u32 = 64;
    const ROWS: u32 = 32;
    const SQUARE_WIDTH: u32 = 8;

    let WIDTH = COLS * SQUARE_WIDTH;
    let HEIGHT = ROWS * SQUARE_WIDTH;

    let mut window: GlutinWindow = WindowSettings::new("Rusty Snek Gaem",[WIDTH, HEIGHT])
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut game = Game{
        gl: GlGraphics::new(opengl),
        rows: ROWS,
        cols: COLS,
        snek: Snek {
            gl: GlGraphics::new(opengl),
            body: LinkedList::from_iter((vec![BodyPiece(COLS / 2, ROWS / 2)]).into_iter()),
            facing: Direction::Right,
            width: SQUARE_WIDTH,
        },
        ate: false,
        target: Meal{x:1, y:1},
        score: 0,
        square_width: SQUARE_WIDTH,
    };

    let mut events = Events::new(EventSettings::new()).ups(8);
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(u) = e.update_args(){
            if !game.update(&u) {
                break;
            }
        }

        if let Some(k) = e.button_args() {
            if k.state == ButtonState::Press {
                game.pressed(&k.button);
            }
        }
    }
    println!("Congratulations, your score was: {}", game.score);
}