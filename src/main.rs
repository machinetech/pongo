extern crate rand;
extern crate sdl2;
extern crate time;

use rand::distributions::{IndependentSample, Range};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::rect::Rect;
use sdl2::Sdl;
use sdl2::TimerSubsystem;
use sdl2::VideoSubsystem;

use std::f32;
use std::path::Path;
use std::thread;

use time::Duration;
use time::SteadyTime;

const WIN_W: i32 = 800;
const WIN_H: i32 = 600;
const BALL_W: i32 = 10;
const BALL_H: i32 = 10;
const PADDLE_W: i32 = 10;
const PADDLE_H: i32 = 100;
const FRAMES_PER_SECOND: i32 = 40;

struct Ball { x: i32, y: i32, w: i32, h: i32, dx: i32, dy: i32 }
struct Paddle { x: i32, y: i32, w: i32, h: i32, dy: i32 }

pub struct Ball {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    velocity: u32,
    x_velocity: f32,
    y_velocity: f32,
}

impl Ball {

    pub fn new(x: u32, y: u32, width: u32, height: u32, velocity: u32) -> Ball {
        let yv = (velocity as f32/2.0) 
        let launch_angle = Range::new(0.75_f32 * f32::consts::PI, 
                                      1.25_f32 * f32::consts::PI);
        let mut rng = rand::thread_rng();
        let speed_in_pix_per_frame = 3_f32;
        let angle_in_radians = between.ind_sample(&mut rng);
        Ball {
            x: x,
            y: y,
            width: width,
            height: height,
            velocity: velocity,
            x_velocity: 0,
            y_velocity: 0
        }
    }

    fn calculate x_y_velocities(velocity: u32) {
         
    }
}

pub struct Game {
    screen_width: u32,
    screen_height: u32,
    fps: u32,
    ball_velocity: u32,
    paddle_velocity: u32,
    sdl: Sdl,
    timer_subsys: TimerSubsystem,
    video_subsys: VideoSubsystem
}

impl Game {

    fn new(screen_width: u32, screen_height: u32, fps: u32,
           ball_velocity: u32, paddle_velocity: u32) -> Game { 
        let sdl = sdl2::init().unwrap();
        let timer_subsys = sdl.timer().unwrap();
        let video_subsys = sdl.video().unwrap();
        Game {
            screen_width: screen_width,
            screen_height: screen_height,
            fps: fps,
            ball_velocity: ball_velocity,
            paddle_velocity: paddle_velocity,
            sdl: sdl,
            timer_subsys: timer_subsys,
            video_subsys: video_subsys
        }    
    }

    pub fn start(&mut self) {
        let mut t0_ms = self.timer_subsys.ticks(); 
        loop {
            let t1_ms = self.timer_subsys.ticks(); 
            self.update(t1_ms - t0_ms); 
            let t2_ms = self.timer_subsys.ticks(); 
            self.cap_fps(t2_ms - t1_ms);
            t0_ms = t1_ms;
        } 
    }

    fn update(&mut self, dt: u32) {
    
    }

    fn cap_fps(&self, took_ms: u32) {
        let max_ms = 1000 / self.fps;
        if max_ms > took_ms {
            thread::sleep_ms(max_ms - took_ms);
        }
    }
}

pub struct GameBuilder {
    screen_width: u32,
    screen_height: u32,
    fps: u32,
    ball_velocity: u32,
    paddle_velocity: u32
}

impl GameBuilder {

    pub fn new() -> GameBuilder {
        GameBuilder {
            screen_width: 600,
            screen_height: 300,
            fps: 40,
            ball_velocity: 10,
            paddle_velocity: 20
        }
    }

    pub fn with_screen_width(mut self, width: u32) -> GameBuilder {
        self.screen_width = width;
        return self; 
    }

    pub fn with_screen_height(mut self, height: u32) -> GameBuilder {
        self.screen_height = height;
        return self;
    }

    pub fn with_fps(mut self, fps: u32) -> GameBuilder {
        self.fps = fps;
        return self;
    }

    pub fn with_ball_velocity_pix_per_ms(mut self, ball_velocity: u32) -> 
        GameBuilder {
        self.ball_velocity = ball_velocity;
        return self;
    }

    pub fn with_paddle_velocity_pix_per_ms(mut self, paddle_velocity: u32) -> 
        GameBuilder {
        self.paddle_velocity = paddle_velocity;
        return self;
    }

    pub fn build(mut self) -> Game {
        Game::new(self.screen_width,
                  self.screen_height,
                  self.fps,
                  self.ball_velocity,
                  self.paddle_velocity)
    } 
}

fn main() {
    let game = GameBuilder::new()
        .with_screen_width(600)
        .with_screen_height(300)
        .with_fps(40)
        .with_ball_velocity_pix_per_ms(10)
        .with_paddle_velocity_pix_per_ms(20)
        .build()
        .start();
    return;

    let sdl = sdl2::init().unwrap();
    let mut timer_subsystem = sdl.timer().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem.window("pong", WIN_W as u32, WIN_H as u32)
                                .position_centered()
                                .build()
                                .unwrap();

    let mut renderer = window.renderer().build().unwrap();
    let mut ball = Ball { x: WIN_W / 2, y: WIN_H / 2, w: BALL_W, h: BALL_H, dx: 1, dy: 2 };
    let between = Range::new(0.75_f32 * f32::consts::PI, 1.25_f32 * f32::consts::PI);
    let mut rng = rand::thread_rng();
    let speed_in_pix_per_frame = 3_f32;
    let angle_in_radians = between.ind_sample(&mut rng);
    ball.dx = (angle_in_radians.cos() * speed_in_pix_per_frame).ceil() as i32;
    ball.dy = (angle_in_radians.sin() * speed_in_pix_per_frame).ceil() as i32;
    let mut lpad = Paddle { x: 0, y: (WIN_H - PADDLE_H) / 2, w: PADDLE_W, 
                            h: PADDLE_H, dy: (speed_in_pix_per_frame * 3.0).ceil() as i32 }; 
    'game_loop: loop {
        let start_ms = timer_subsystem.ticks(); 
        match poll_event(&sdl) {
            None => {},
            Some(event) => {
                match event {
                    Event::Quit{..} => {
                        break 'game_loop;
                    },
                    Event::KeyDown{keycode,..} => match keycode {
                        Option::Some(Keycode::Escape) => {
                            break 'game_loop;
                        },
                        Option::Some(Keycode::Up) => {
                            lpad.y -= lpad.dy;
                            if lpad.y <= 0 { lpad.y = 0; } 
                        },
                        Option::Some(Keycode::Down) => {
                            lpad.y += lpad.dy;
                            if lpad.y + lpad.h >= WIN_H { lpad.y = WIN_H - lpad.h; } 
                        },
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        if ball.x + ball.dx <= lpad.x + lpad.w  && 
            ball.y + ball.dy >= lpad.y  && 
            ball.y + ball.dy <= lpad.y + lpad.h {
            ball.x = lpad.x + lpad.w;
            let ball_y_intersection = ball.y + (ball.h / 2);
            let ball_y_relative_intersection = lpad.y + (lpad.h / 2) - ball_y_intersection;
            let ball_y_norm_relative_intersection = ball_y_relative_intersection as f32 / (lpad.h / 2) as f32;
            let max_angle_in_radians = 5.0 / 12.0 * f32::consts::PI; 
            let angle_in_radians = max_angle_in_radians * ball_y_norm_relative_intersection;
            ball.dx = (angle_in_radians.cos() * speed_in_pix_per_frame).ceil() as i32;
            ball.dy = (angle_in_radians.sin() * speed_in_pix_per_frame).ceil() as i32;
        } else if ball.x + ball.w + ball.dx >= WIN_W {
            ball.x = WIN_W - ball.w;
            ball.dx *= -1;
        } else {
            ball.x += ball.dx;
        } 
        if ball.x + ball.dx <= 0 {
            ball.x = 0;
            ball.dx *= -1;
        } else if ball.x + ball.w + ball.dx >= WIN_W {
            ball.x = WIN_W - ball.w;
            ball.dx *= -1;
        } else {
            ball.x += ball.dx;
        } 
        if ball.y + ball.dy <= 0 {
            ball.y = 0;
            ball.dy *= -1;
        } else if ball.y + ball.h + ball.dy >= WIN_H {
            ball.y = WIN_H - ball.h;
            ball.dy *= -1;
        } else {
            ball.y += ball.dy;
        } 
        let ball_rect = Rect::new_unwrap(ball.x, ball.y, ball.w as u32, ball.h as u32);
        let lpad_rect = Rect::new_unwrap(lpad.x, lpad.y, lpad.w as u32, lpad.h as u32);
        renderer.clear();
        renderer.fill_rect(ball_rect);
        renderer.fill_rect(lpad_rect);
        renderer.present();
        let end_ms = timer_subsystem.ticks(); 
        cap_frames_per_sec(end_ms - start_ms);
    }

}

fn cap_frames_per_sec(actual_time_per_frame_ms: u32) {
    let expected_time_per_frame_sec = 1.0 / FRAMES_PER_SECOND as f32;
    let expected_time_per_frame_ms  = (expected_time_per_frame_sec  * 1000.) as u32; 
    if expected_time_per_frame_ms > actual_time_per_frame_ms {
        let diff_ms = expected_time_per_frame_ms - actual_time_per_frame_ms;
        thread::sleep_ms(diff_ms);
    }
}

fn poll_event(sdl: &Sdl) -> Option<Event> {
    let mut event_pump = sdl.event_pump().unwrap();
    return event_pump.poll_event();
}

fn get_updated_keys(sdl: &Sdl) -> [bool; 16] {
    let event_pump = sdl.event_pump().unwrap();
    let keyboard_state = event_pump.keyboard_state();
    let mut keys = [false; 16];
    keys[0x0] = keyboard_state.is_scancode_pressed(Scancode::X);
    keys[0x1] = keyboard_state.is_scancode_pressed(Scancode::Num1);
    keys[0x2] = keyboard_state.is_scancode_pressed(Scancode::Num2);
    keys[0x3] = keyboard_state.is_scancode_pressed(Scancode::Num3);
    keys[0x4] = keyboard_state.is_scancode_pressed(Scancode::Q);
    keys[0x5] = keyboard_state.is_scancode_pressed(Scancode::W);
    keys[0x6] = keyboard_state.is_scancode_pressed(Scancode::E);
    keys[0x7] = keyboard_state.is_scancode_pressed(Scancode::A);
    keys[0x8] = keyboard_state.is_scancode_pressed(Scancode::S);
    keys[0x9] = keyboard_state.is_scancode_pressed(Scancode::D);
    keys[0xA] = keyboard_state.is_scancode_pressed(Scancode::Z);
    keys[0xB] = keyboard_state.is_scancode_pressed(Scancode::C);
    keys[0xC] = keyboard_state.is_scancode_pressed(Scancode::Num4);
    keys[0xD] = keyboard_state.is_scancode_pressed(Scancode::R);
    keys[0xE] = keyboard_state.is_scancode_pressed(Scancode::F);
    keys[0xF] = keyboard_state.is_scancode_pressed(Scancode::V);
    keys
}


