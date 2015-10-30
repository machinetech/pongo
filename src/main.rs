extern crate clock_ticks;
extern crate rand;
extern crate sdl2;

use rand::distributions::{IndependentSample, Range};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::rect::Rect;
use sdl2::render::Renderer;
use sdl2::Sdl;
use sdl2::TimerSubsystem;
use sdl2::VideoSubsystem;
 
use std::default::Default;
use std::f32;
use std::path::Path;
use std::thread;

pub const FPS: u32 = 40;
pub const SCREEN_WIDTH: f32 = 600.;
pub const SCREEN_HEIGHT: f32 = 300.;

pub struct Ui {
    sdl_ctx: Sdl,
    renderer: Renderer<'static>
}

impl Ui {
    pub fn new() -> Ui {
        let sdl_ctx = sdl2::init().unwrap();
        let video_subsystem = sdl_ctx.video().unwrap();
        let window = video_subsystem.window("pong", 
                SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
                .position_centered()
                .build()
                .unwrap();
        let renderer = window.renderer().build().unwrap();
        Ui {
            sdl_ctx: sdl_ctx,
            renderer: renderer
        }  
    } 

    pub fn poll_event(&self) -> Option<Event> {
        let mut event_pump = self.sdl_ctx.event_pump().unwrap();
        return event_pump.poll_event();
    }
}

pub struct Ball {
    pub x: f32,         // x pixel co-ordinate of top left corner
    pub y: f32,         // y pixel co-ordinate of top left corner
    pub width: f32,     // pixels
    pub height: f32,    // pixels
    pub speed: f32,     // pixels per second 
    pub vx: f32,        // pixels per second
    pub vy: f32         // pixels per second
}

impl Ball {
    pub fn new() -> Ball {
        let width = 10.;
        let height = 10.;
        
        // Place ball at center of screen. 
        let x = (SCREEN_WIDTH - width) / 2.;
        let y = (SCREEN_HEIGHT - height) / 2.;

        // Able to travel height of screen in 1 second
        let speed = SCREEN_HEIGHT;
        let mut rng = rand::thread_rng();

        // Launch at an angle less than or equal to 45 degrees.
        let angle = Range::new(0., f32::consts::PI/4.).ind_sample(&mut rng);
        let dir = [-1., 1.];

        // Use the sine of the angle to determine the vertical speed. Then, 
        // choose a direction (up or down) to select a vertical velocity.
        let up_or_down = rand::sample(&mut rng, dir.into_iter(),1)[0]; 
        let vy = angle.sin() * speed * up_or_down; 
        let left_or_right = rand::sample(&mut rng, dir.into_iter(),1)[0]; 

        // Use Pythagoras to determine the horizontal speed. Then, choose a
        // direction (left or right) to select a horizontal velocity.
        let vx = ((speed * speed) - (vy * vy)).sqrt() * left_or_right;
        Ball {
            x: x,
            y: y,
            width: width,
            height: height,
            speed: speed,
            vx: vx,
            vy: vy
        }
    }
}

pub struct Game {
    ui: Ui,
    ball: Ball,
    running: bool
}

impl Game {

    /// Create initial game state. 
    pub fn new() -> Game { 
        Game {
            ui: Ui::new(),
            ball: Ball::new(),
            running: false,
        }
    }

    /// Start the game and block until finished. 
    pub fn start(&mut self) {
        self.running = true;
        let mut time_last_invocation = clock_ticks::precise_time_ms();
        while self.running {
            let time_this_invocation = clock_ticks::precise_time_ms();
            let delta_time = time_this_invocation - time_last_invocation;
            self.update(delta_time as f32 / 1000.); 
            self.cap_fps(delta_time);
            time_last_invocation = time_this_invocation;
        } 
    }

    /// Called once per frame. 
    fn update(&mut self, dt_sec: f32) {
        self.handle_input();
        self.update_ball_position(dt_sec);
        self.check_for_ball_and_wall_collisions();
        self.ui.renderer.clear();
        let ball = &mut self.ball;
        let rect = Rect::new_unwrap(ball.x as i32, 
                                    ball.y as i32, 
                                    ball.width as u32,
                                    ball.height as u32);
        self.ui.renderer.fill_rect(rect);
        self.ui.renderer.present();
    }

    fn handle_input(&mut self) {
        match self.ui.poll_event() {
            Some(event) => {
                match event {
                    Event::Quit{..} => {
                        self.running = false;
                    },
                    Event::KeyDown{keycode,..} => match keycode {
                        Option::Some(Keycode::Escape) => {
                            self.running = false;
                        },
                        _ => {}
                    },
                    _ => {}
                }
            },
            None => {}
        }
    }
    
    fn update_ball_position(&mut self, dt_sec: f32) {
        let ball = &mut self.ball;
        ball.x += ball.vx * dt_sec;
        ball.y += ball.vy * dt_sec;
    }
    
    fn check_for_ball_and_wall_collisions(&mut self) {
        self.check_for_ball_and_top_wall_collision();
        self.check_for_ball_and_right_wall_collision();
        self.check_for_ball_and_bottom_wall_collision();
        self.check_for_ball_and_left_wall_collision();
    }

    fn check_for_ball_and_top_wall_collision(&mut self) {
        let ball = &mut self.ball;
        if ball.y <= 0. && ball.vy < -0. {
            ball.y = 0.;
            ball.vy = -ball.vy; 
        }
    }

    fn check_for_ball_and_right_wall_collision(&mut self) {
        let ball = &mut self.ball; 
        if ball.x + ball.width >= SCREEN_WIDTH && ball.vx > 0. {
            ball.x = SCREEN_WIDTH - ball.width;
            ball.vx = -ball.vx;
        } 
    }

    fn check_for_ball_and_bottom_wall_collision(&mut self) {
        let ball = &mut self.ball;
        if ball.y + ball.height >= SCREEN_HEIGHT && ball.vy > 0. {
            ball.y = SCREEN_HEIGHT - ball.height;
            ball.vy = -ball.vy;
        } 
    }

    fn check_for_ball_and_left_wall_collision(&mut self) {
        let ball = &mut self.ball; 
        if ball.x <= 0. && ball.vx < -0. {
            ball.x = 0.;
            ball.vx = -ball.vx; 
        } 
    }

    // Ensure we run no faster than the desired fps by introducing
    // a delay if necessary.
    fn cap_fps(&self, took_ms: u64) {
        let max_ms = 1000 / FPS as u64;
        if max_ms > took_ms {
            thread::sleep_ms((max_ms - took_ms) as u32);
        }
    }
}

fn main() {
    let mut game = Game::new(); 
    game.start();
}
