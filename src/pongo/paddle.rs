extern crate clock_ticks;
extern crate rand;
extern crate sdl2;
extern crate sdl2_gfx;
extern crate sdl2_image;
extern crate sdl2_mixer;
extern crate sdl2_ttf;

use pongo::ui::{Drawable, Ui};

use rand::distributions::{IndependentSample, Range};

use sdl2::{AudioSubsystem, Sdl};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture};

use sdl2_gfx::primitives::DrawRenderer;
use sdl2_image::{LoadTexture, INIT_PNG}; 
use sdl2_mixer::{AUDIO_S16LSB, DEFAULT_FREQUENCY, Music}; 
use sdl2_ttf::{Font, Hinting, Sdl2TtfContext}; 

use std::cell::RefCell;
use std::f32;
use std::path::Path;
use std::rc::Rc;
use std::thread;
use std::vec::Vec;

use super::Resettable;

pub struct Paddle {
    pub color: Color,   
    pub initial_x: f32,         // The initial x location. Stored so that we can reset the paddle.
    pub initial_y: f32,         // The initial y location. Stored so that we can reset the paddle.
    pub x: f32,                 // x pixel coordinate of top left corner
    pub y: f32,                 // y pixel coordinate of top left corner
    pub width: f32,     
    pub height: f32,    
    pub speed: f32,             // Speed in pixels per second. Never changes. 
    pub speed_multiplier: f32   // Used to adjust the speed.
}

impl Paddle {

    pub fn new(color: Color, 
           x: f32, 
           y: f32, 
           width: f32, 
           height: f32, 
           speed: f32) -> Paddle {

        let mut paddle = Paddle { 
            color: color,
            initial_x: x, 
            initial_y: y, 
            x: x, 
            y: y, 
            width: width, 
            height: height, 
            speed: speed,
            speed_multiplier: 1.0
        };

        paddle.reset();
        return paddle;
    }

}

impl Resettable for Paddle {

    fn reset(&mut self) {
        
        // Revert to the initial x and y coordinates.
        self.x = self.initial_x;
        self.y = self.initial_y;

        // Revert to initial speed by setting the multiplier back to 1.
        self.speed_multiplier = 1.;
    }

}

impl Drawable for Paddle {

    fn draw(&self, ui: &mut Ui) {
        ui.renderer.set_draw_color(self.color);
        ui.renderer.fill_rect(Rect::new_unwrap(self.x as i32, 
                                    self.y as i32, 
                                    self.width as u32, 
                                    self.height as u32));
    }

}
