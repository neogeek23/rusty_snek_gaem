extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};

use std::collections::LinkedList;
use std::iter::FromIterator;

#[derive(Clone, PartialEq)]
enum Direction{
    Right, Left, Up, Down
}

struct Game{
    gl:GlGraphics,
    snek: Snek,
}

impl Game{
    fn render(&mut self, arg: &RenderArgs){
        use graphics;

        let blue: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

        self.gl.draw(arg.viewport(), |_c, gl|{
            graphics::clear(blue, gl);
        });

        self.snek.render(&mut self.gl, arg);
    }

    fn update(&mut self){
        self.snek.update();
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
    body: LinkedList<(i32,i32)>,
    facing: Direction,
}

impl Snek{
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs){
        use graphics;

        let red: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let squares: Vec<graphics::types::Rectangle> = self.body
            .iter()
            .map(|&(x, y)|{
                graphics::rectangle::square(
                    (x * 16) as f64, 
                    (y * 16) as f64, 
                    16_f64);
            })
            .collect();

        gl.draw(args.viewport(), |c, gl|{
            let transform = c.transform;
            squares.into_iter()
                .for_each(|square|graphics::rectangle(red, square, transform, gl));
        });
    }

    fn update(&self){
        let mut new_head = (*self.body.front().expect("Snek has no body")).clone();
        match self.facing{
            Direction::Left => new_head.0 -= 1,
            Direction::Right => new_head.0 += 1,
            Direction::Up => new_head.1 -= 1,
            Direction::Down => new_head.1 += 1,
        }

        self.body.push_front(new_head);
        self.body.pop_back().unwrap();
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: GlutinWindow = WindowSettings::new(
        "Rusty Snek Gaem",
        [512, 512]
    ).opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut game = Game{
        gl: GlGraphics::new(opengl),
        snek: Snek {
            body: LinkedList::from_iter((vec![(0,0), (0,1)]).into_iter()),
            facing: Direction::Right
        },
    };

    let mut events = Events::new(EventSettings::new()).ups(16);
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(u) = e.update_args(){
            game.update();
        }
    }
}
