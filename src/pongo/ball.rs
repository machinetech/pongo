extern crate rand;

use pongo::ui::{Drawable,Ui};
use rand::distributions::{IndependentSample, Range};
use sdl2::pixels::Color;
use sdl2_gfx::primitives::DrawRenderer;
use super::Resettable;

// The ball is rendered as a circle, but treated as a square to simplify game mechanics. 
pub struct Ball {
    pub color: Color,                   
    pub initial_x: f32,         // The initial x location. Stored so that we can reset the ball.
    pub initial_y: f32,         // The initial y location. Stored so that we can reset the ball.
    pub x: f32,                 // x pixel coordinate of top left corner.
    pub y: f32,                 // y pixel coordinate of top left corner.
    pub diameter: f32,           
    pub speed: f32,             // Speed in pixels per second. Never changes.
    pub speed_multiplier: f32,  // Used to adjust the speed.
    pub vx: f32,                // Horizontal velocity in pixels per second.
    pub vy: f32,                // Vertical velocity in pixels per second.
    pub max_launch_angle: f32,  // Maximum angle at which the ball will launch. 
    pub max_bounce_angle: f32   // Maximum angle at which ball will bounce when hitting paddle.
                                // The angle is taken as up or down from an imaginary line
                                // running perpendicular to the paddle (i.o.w. running horizontal)
}

impl Ball {

    pub fn new(color: Color, 
           x: f32, 
           y: f32, 
           diameter: f32, 
           speed: f32, 
           max_launch_angle: f32, 
           max_bounce_angle: f32) -> Ball {

        let mut ball = Ball { 
            color: color, 
            initial_x: x, 
            initial_y: y, 
            x: x, 
            y: y, 
            diameter: diameter, 
            speed: speed, 
            speed_multiplier: 1.0, 
            vx: 0., 
            vy: 0., 
            max_launch_angle: max_launch_angle, 
            max_bounce_angle: max_bounce_angle 
        };
        
        ball.reset();
        return ball
    }
}

impl Resettable for Ball {

    fn reset(&mut self) {
        
        // Restore the initial x and y coordinates.
        self.x = self.initial_x;
        self.y = self.initial_y;

        // Revert back to the initial speed by setting the multiplier to 1.
        self.speed_multiplier = 1.;

        // Calculate a new launch angle. The launch angle is always random, but never greater
        // than the configured maximum launch angle.
        let mut rng = rand::thread_rng();
        let launch_angle = Range::new(0., self.max_launch_angle).ind_sample(&mut rng);
        
        // Posible direction can be either up (-1) or down (+1).
        let dir = [-1., 1.];

        // Use the sine of the angle to determine the vertical speed. Then, 
        // choose a direction (up or down) to select a vertical velocity.
        let up_or_down = rand::sample(&mut rng, dir.into_iter(),1)[0]; 
        let vy = launch_angle.sin() * self.speed * up_or_down; 
        let left_or_right = rand::sample(&mut rng, dir.into_iter(),1)[0]; 
        
        // Use Pythagoras to determine the horizontal speed. Then, choose a
        // direction (left or right) to select a horizontal velocity.
        let vx = ((self.speed * self.speed) - (vy * vy)).sqrt() * left_or_right;

        // Assign the newly calculated horizontal and vertical velocities.
        self.vx = vx;
        self.vy = vy;
    }
}

impl Drawable for Ball {

    fn draw(&self, ui: &mut Ui) {
        let x = self.x + self.diameter / 2.;
        let y = self.y + self.diameter / 2.;
        let radius = self.diameter / 2.;
        ui.renderer.filled_circle(x as i16, y as i16, radius as i16, self.color);
    }
    
}
