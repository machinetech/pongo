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

pub struct Ui {
    width: f32,
    height: f32,
    sdl_ctx: Sdl,
    renderer: Renderer<'static>
}

impl Ui {
    pub fn new(width: f32, height: f32, sdl_ctx: Sdl, renderer: Renderer<'static>) -> Self {
        Ui {
            width: width,
            height: height,
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
    pub fn new(x: f32, y: f32, width: f32, height: f32, 
               speed: f32, vx: f32, vy: f32) -> Self {
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

pub struct Paddle {
    pub x: f32,         // x pixel co-ordinate of top left corner
    pub y: f32,         // y pixel co-ordinate of top left corner
    pub width: f32,     // pixels
    pub height: f32,    // pixels
    pub speed: f32,     // pixels per second
    pub vy: f32,        // pixels per second
    pub score: u32
}

impl Paddle {
    pub fn new(x: f32, y: f32, width: f32, height: f32, speed: f32, vy: f32, 
               score: u32) -> Self {
        Paddle {
            x: x,
            y: y,
            width: width,
            height: height,
            speed: speed,
            vy: vy,
            score: score}
    }
}

pub struct Game {
    ui: Ui,
    fps: u32,
    ball: Ball,
    lpaddle: Paddle,
    rpaddle: Paddle,
    running: bool
}

impl Game {

    /// Create initial game state. 
    pub fn new(ui: Ui, fps: u32, ball: Ball, lpaddle: Paddle, 
               rpaddle: Paddle) -> Self { 
        Game {
            ui: ui,
            fps: fps,
            ball: ball,
            lpaddle: lpaddle,
            rpaddle: rpaddle,
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
        self.handle_input(dt_sec);
        self.update_ball_position(dt_sec);
        self.check_for_ball_and_wall_collisions();
        self.check_for_ball_and_lpaddle_collision();
        self.redraw()
    }

    fn handle_input(&mut self, dt_sec: f32) {
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
                        Option::Some(Keycode::Up) => {
                            let lpaddle = &mut self.lpaddle;
                            lpaddle.y -= lpaddle.speed * dt_sec;
                            if lpaddle.y < 0. { lpaddle.y = 0.; }
                        },
                        Option::Some(Keycode::Down) => {
                            let lpaddle = &mut self.lpaddle;
                            lpaddle.y += lpaddle.speed * dt_sec;
                            if lpaddle.y + lpaddle.height > self.ui.height {
                                lpaddle.y = self.ui.height - lpaddle.height; 
                            }
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
        let ball = &mut self.ball;
        let ui = &mut self.ui;

        // Left or right wall.
        if ball.x <= 0. && ball.vx < -0. {
            ball.x = 0.;
            ball.vx = -ball.vx; 
        } else if ball.x + ball.width >= ui.width && ball.vx > 0. {
            ball.x = ui.width - ball.width;
            ball.vx = -ball.vx;
        } 

        // Top or bottom wall.
        if ball.y <= 0. && ball.vy < -0. {
            ball.y = 0.;
            ball.vy = -ball.vy; 
        } else if ball.y + ball.height >= ui.height && ball.vy > 0. {
            ball.y = ui.height - ball.height;
            ball.vy = -ball.vy;
        }
    }

    fn check_for_ball_and_lpaddle_collision(&mut self) {
        let ball = &mut self.ball;
        let lpaddle = &mut self.lpaddle;

        if ball.vx < 0. && ball.x <= lpaddle.width && 
            ball.y >= lpaddle.y && ball.y <= lpaddle.y + lpaddle.height {

            // Calculate the ball's position relative to the center of the paddle. 
            let hit_loc = ball.y - lpaddle.y;
            
            // Calculate an angle multiplier as a percentage of half the paddle's height. 
            let angle_multiplier = (hit_loc / (lpaddle.height / 2.)).abs();

            // Calculate the angle the ball should return at.
            let angle = (f32::consts::PI / 6.) * angle_multiplier;

            // Calculate new vertical and horizontal velocities.
            let vx = angle.cos() * ball.speed;
            let vy = angle.sin() * ball.speed * if hit_loc > 0. {1.} else {-1.};
            
            ball.x = ball.width;
            ball.vx = vx;
            ball.vy = vy;
        } 
    }

    fn redraw(&mut self) {
        // Clear the screen.
        self.ui.renderer.clear();
        
        // Draw the ball.
        let ball = &mut self.ball;
        let ball_rect = Rect::new_unwrap(ball.x as i32, 
                                    ball.y as i32, 
                                    ball.width as u32,
                                    ball.height as u32);
        self.ui.renderer.fill_rect(ball_rect);

        // Draw the left paddle.
        let lpaddle = &mut self.lpaddle;
        let lpaddle_rect = Rect::new_unwrap(lpaddle.x as i32, 
                                    lpaddle.y as i32, 
                                    lpaddle.width as u32,
                                    lpaddle.height as u32);
        self.ui.renderer.fill_rect(lpaddle_rect);


        // Draw the right paddle.
        let rpaddle = &mut self.rpaddle;
        let rpaddle_rect = Rect::new_unwrap(rpaddle.x as i32, 
                                    rpaddle.y as i32, 
                                    rpaddle.width as u32,
                                    rpaddle.height as u32);
        self.ui.renderer.fill_rect(rpaddle_rect);

        // Flip backbuffer to front.
        self.ui.renderer.present();
    }

    // Ensure we run no faster than the desired fps by introducing
    // a delay if necessary.
    fn cap_fps(&self, took_ms: u64) {
        let max_ms = 1000 / self.fps as u64;
        if max_ms > took_ms {
            thread::sleep_ms((max_ms - took_ms) as u32);
        }
    }
}

struct GameBuilder {
    screen_width: f32,
    screen_height: f32,
    fps: u32,
    ball_speed: f32,
    paddle_speed: f32
}

impl GameBuilder {

    pub fn new() -> Self {
        GameBuilder {
            screen_width: 480.,
            screen_height: 320.,
            fps: 40,
            ball_speed: 320.,
            paddle_speed: 640.
        }
    }

    pub fn with_dimensions(mut self, width: f32, height: f32) -> Self {
        self.screen_width = width;
        self.screen_height = height;
        self
    }

    pub fn with_fps(mut self, fps: u32) -> Self {
        self.fps = fps; 
        self
    }
    
    pub fn with_ball_speed_per_sec(mut self, ball_speed: f32) -> Self {
        self.ball_speed = ball_speed; 
        self
    }

    pub fn with_paddle_speed_per_sec(mut self, paddle_speed: f32) -> Self {
        self.paddle_speed = paddle_speed; 
        self
    }

    fn create_ui(&self) -> Ui {
        let sdl_ctx = sdl2::init().unwrap();
        let video_subsystem = sdl_ctx.video().unwrap();
        let window = video_subsystem.window("pong", 
                self.screen_width as u32, self.screen_height as u32)
                .position_centered()
                .build()
                .unwrap();
        let renderer = window.renderer().build().unwrap();
        Ui::new(self.screen_width, self.screen_height, sdl_ctx, renderer)
    }

    fn create_ball(&self) -> Ball {
        let width = 10.;
        let height = 10.;
        
        // Place ball at center of screen. 
        let x = (self.screen_width - width) / 2.;
        let y = (self.screen_height - height) / 2.;

        let speed = self.ball_speed;
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
        Ball::new(x, y, width, height, speed, vx, vy)
    }    

    fn create_left_paddle(&self) -> Paddle {
        let width = 10.;
        let height = 60.;
        let x = 0.;
        let y = (self.screen_height - height) / 2.;
        let vy = 0.;
        let score = 0;
        Paddle::new(x, y, width, height, self.paddle_speed, vy, score)
    }

    fn create_right_paddle(&self) -> Paddle {
        let width = 10.;
        let height = 60.;
        let x = self.screen_width - width;;
        let y = (self.screen_height - height) / 2.;
        let vy = 0.;
        let score = 0;
        Paddle::new(x, y, width, height, self.paddle_speed, vy, score)
    }

    pub fn build(&self) -> Game {
        Game::new(self.create_ui(), 
                  self.fps, 
                  self.create_ball(),
                  self.create_left_paddle(),
                  self.create_right_paddle())
    }
}

fn main() {
    let mut game = GameBuilder::new()
        .with_dimensions(480., 320.)
        .with_fps(40)
        .with_ball_speed_per_sec(320.)
        .with_paddle_speed_per_sec(1000.)
        .build();
    game.start();
}
