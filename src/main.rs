extern crate clock_ticks;
extern crate glium;
extern crate rand;


use glium::{Display, DisplayBuild, Surface};

use rand::distributions::{IndependentSample, Range};

use std::default::Default;
use std::f32;
use std::path::Path;
use std::thread;

pub struct Game {
    display: Display,
    fps: u32,
    ball_velocity: u32,
    paddle_velocity: u32,
}

impl Game {

    pub fn new(display: Display, fps: u32, ball_velocity: u32, paddle_velocity: u32) -> Game { 
        Game {
            display: display,
            fps: fps,
            ball_velocity: ball_velocity,
            paddle_velocity: paddle_velocity
        }
    }

    pub fn start(&mut self) {
        let mut t0_ms = clock_ticks::precise_time_ms(); 
        loop {
            let t1_ms = clock_ticks::precise_time_ms(); 
            if self.update(t1_ms - t0_ms) == false { return; } 
            let t2_ms = clock_ticks::precise_time_ms(); 
            self.cap_fps(t2_ms - t1_ms);
            t0_ms = t1_ms;
        } 
    }

    fn update(&mut self, dt: u64) -> bool {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        target.finish().unwrap();
        for ev in self.display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return false,
                _ => ()
            }
        }
        true
    }

    fn cap_fps(&self, took_ms: u64) {
        let max_ms = 1000 / self.fps as u64;
        if max_ms > took_ms {
            thread::sleep_ms((max_ms - took_ms) as u32);
        }
    }
}

pub struct GameBuilder {
    display_width: u32,
    display_height: u32,
    fps: u32,
    ball_velocity: u32,
    paddle_velocity: u32
}

impl Default for GameBuilder {
    #[inline]
    fn default() -> GameBuilder {
        GameBuilder {
            display_width: 600,
            display_height: 300,
            fps: 40,
            ball_velocity: 5,
            paddle_velocity: 10 
        }
    }
}

impl GameBuilder {

    pub fn new() -> GameBuilder {
        Default::default()
    }

    pub fn with_dimensions(mut self, width: u32, height: u32) -> GameBuilder {
        self.display_width = width;
        self.display_height = height;
        self 
    }

    pub fn with_fps(mut self, fps: u32) -> GameBuilder {
        self.fps = fps;
        self
    }

    pub fn with_ball_velocity_pix_per_ms(mut self, ball_velocity: u32) -> 
        GameBuilder {
        self.ball_velocity = ball_velocity;
        self
    }

    pub fn with_paddle_velocity_pix_per_ms(mut self, paddle_velocity: u32) -> 
        GameBuilder {
        self.paddle_velocity = paddle_velocity;
        self
    }

    pub fn build(mut self) -> Game {
        let display = glium::glutin::WindowBuilder::new()
            .with_dimensions(self.display_width, self.display_height)
            .with_title(format!("pong"))
            .build_glium().unwrap();
        Game::new(display,
                  self.fps,
                  self.ball_velocity,
                  self.paddle_velocity)
    } 
}

fn main() {
    let mut game = GameBuilder::new()
        .with_dimensions(600, 300)
        .with_fps(40)
        .with_ball_velocity_pix_per_ms(10)
        .with_paddle_velocity_pix_per_ms(20)
        .build();
    game.start();
}
