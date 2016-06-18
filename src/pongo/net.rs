extern crate clock_ticks;
extern crate rand;
extern crate sdl2;
extern crate sdl2_gfx;
extern crate sdl2_image;
extern crate sdl2_mixer;
extern crate sdl2_ttf;

use pongo::ui::{Drawable,Ui};

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

pub struct Net {
    pub color: Color,   
    pub x: f32,            // x pixel coordinate of top left corner  
    pub dot_width: f32,
    pub dot_height: f32,
    pub num_dots: i32
}

impl Net {
    
    pub fn new(color: Color, x: f32, dot_width: f32, dot_height: f32, num_dots: i32) -> Net {
        return Net {
            color: color,
            x: x,
            dot_width: dot_width,
            dot_height: dot_height,
            num_dots: num_dots
        };
    }

}

impl Drawable for Net {

    fn draw(&self, ui: &mut Ui) {
        let dot_x = self.x;
        let num_gaps = self.num_dots - 1;
        for i in 0..self.num_dots + num_gaps + 1 {
            if i % 2 == 0 {
                let dot_y = i as f32 * self.dot_height; 
                ui.renderer.set_draw_color(self.color);
                let dot_rect = Rect::new_unwrap(dot_x as i32, dot_y as i32, 
                                                self.dot_width as u32, 
                                                self.dot_height as u32);
                ui.renderer.fill_rect(dot_rect);
            }
        }
    }
}
