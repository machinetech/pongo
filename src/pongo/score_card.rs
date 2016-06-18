extern crate sdl2_ttf;

use pongo::ui::{Drawable, Ui};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2_ttf::Font; 
use std::rc::Rc;
use super::Resettable;

pub struct ScoreCard {
    pub color: Color,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub font: Rc<Font>,
    pub score: i32
}

impl ScoreCard {

    pub fn new(color: Color, x: f32, y: f32, width: f32, height: f32, font: Rc<Font>) -> ScoreCard {
        return ScoreCard {
            color: color,
            x: x,
            y: y,
            width: width,
            height: height,
            font: font,
            score: 0 
        };
    }

}

impl Drawable for ScoreCard {
   
    fn draw(&self, ui: &mut Ui) {
        let formatted_score = format!("{:^3}", self.score);
        let formatted_score_ref: &str = formatted_score.as_ref();
        let surface = self.font.render(formatted_score_ref, 
                                           sdl2_ttf::blended(self.color)).unwrap();
        let texture = ui.renderer.create_texture_from_surface(&surface).unwrap();
        let target = Rect::new_unwrap(self.x as i32, 
                                      self.y as i32, 
                                      self.width as u32, 
                                      self.height as u32);
        ui.renderer.copy(&texture, None, Some(target));
    } 

}

impl Resettable for ScoreCard {

    fn reset(&mut self) {
        self.score = 0;
    }

}
