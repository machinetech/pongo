use pongo::ui::{Drawable,Ui};

use sdl2::pixels::Color;
use sdl2::rect::Rect;

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
