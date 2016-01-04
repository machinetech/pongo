extern crate clock_ticks;
extern crate rand;
extern crate sdl2;
extern crate sdl2_gfx;
extern crate sdl2_image;
extern crate sdl2_mixer;
extern crate sdl2_ttf;

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
use sdl2_ttf::{Font, Sdl2TtfContext}; 

use std::cell::RefCell;
use std::f32;
use std::path::Path;
use std::rc::Rc;
use std::thread;
use std::vec::Vec;

/// Interface for interacting with the user. For example, obtaining user input, drawing to the
/// screen and playing audio.
struct Ui {

    sdl_ctx: Sdl,
    renderer: Renderer<'static>,
    ttf_ctx: Sdl2TtfContext,
    pixel_font: Font,
    sdl_audio: AudioSubsystem, 
    ping_sound: Rc<Music>,
    pong_sound: Rc<Music>

}

impl Ui {

    fn new(sdl_ctx: Sdl, 
           renderer: Renderer<'static>, 
           ttf_ctx: Sdl2TtfContext, 
           pixel_font: Font,
           sdl_audio: AudioSubsystem, 
           ping_sound: Music, 
           pong_sound: Music) -> Ui {

        return Ui { 
            sdl_ctx: sdl_ctx, 
            renderer: renderer,
            ttf_ctx: ttf_ctx,
            pixel_font: pixel_font,
            sdl_audio: sdl_audio,
            ping_sound: Rc::new(ping_sound),
            pong_sound: Rc::new(pong_sound)
        };  

    } 

    /// Poll for a single user event.
    fn poll_event(&self) -> Option<Event> {
        return self.sdl_ctx.event_pump().unwrap().poll_event();
    }

}

/// Trait for types that can be drawn to the screen. 
trait Drawable {
    fn draw(&self, ui: &mut Ui); 
}

/// Trait for types that can be set back to an initial state. 
trait Resettable {
    fn reset(&mut self);
}

struct Ball {

    color: Color,                   
    initial_x: f32,                 // The initial x location. Stored so that we can reset the ball.
    initial_y: f32,                 // The initial y location. Stored so that we can reset the ball.
    x: f32,                         // x pixel co-ordinate of top left corner.
    y: f32,                         // y pixel co-ordinate of top left corner.
    diameter: f32,                   
    speed: f32,                     // Speed in pixels per second. Never changes.
    speed_multiplier: f32,          // Used to adjust the speed.
    vx: f32,                        // Horizontal velocity in pixels per second.
    vy: f32,                        // Vertical velocity in pixels per second.
    max_launch_angle: f32,          // Maximum angle at which the ball will launch. 
    max_bounce_angle: f32           // Maximum angle at which ball will bounce when hitting paddle.
                                    // The angle is taken as up or down from an imaginary line
                                    // running perpendicular to the paddle (i.o.w. running
                                    // horizontal)
}

impl Ball {

    fn new(color: Color, 
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

struct Paddle {

    color: Color,   
    initial_x: f32,         // The initial x location. Stored so that we can reset the paddle.
    initial_y: f32,         // The initial y location. Stored so that we can reset the paddle.
    x: f32,                 // x pixel co-ordinate of top left corner
    y: f32,                 // y pixel co-ordinate of top left corner
    width: f32,     
    height: f32,    
    speed: f32,             // Speed in pixels per second. Never changes. 
    speed_multiplier: f32,  // Used to adjust the speed.
    score: u32              // Points won.              

}

impl Paddle {

    fn new(color: Color, 
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
            speed_multiplier: 1.0,
            score: 0
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

        // Set the score back to zero.
        self.score = 0;
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

/// Holds state that lasts for a single iteration of the game loop.
struct GameLoopContext {

    dt_sec: f32,                                          // Seconds since last game loop.
    layered_draw_queue: [Vec<Rc<RefCell<Drawable>>>; 2],  // Items to draw. 
    audible_queue: Vec<Rc<Music>>                         // Audio that needs to sound.

}

impl GameLoopContext {

    fn new(dt_sec: f32) -> GameLoopContext {

        return GameLoopContext {
            dt_sec: dt_sec,
            layered_draw_queue: [Vec::new(), Vec::new()],
            audible_queue: Vec::new()
        };

    }

}

struct Game {

    ui: Ui,
    background_color: Color,
    width: f32,
    height: f32,
    net_color: Color,
    fps: u32,
    ball: Rc<RefCell<Ball>>,
    lpaddle: Rc<RefCell<Paddle>>,
    rpaddle: Rc<RefCell<Paddle>>,
    time_ball_last_speedup_ms: Option<u64>,
    slow_motions_remaining: u32,
    time_slow_motion_started_ms: Option<u64>,
    running: bool,
    resettables: Vec<Rc<RefCell<Resettable>>>

}

/// Contains the game state and executes the game loop.
impl Game {

    fn new(ui: Ui, 
           background_color: Color, 
           width: f32,
           height: f32,
           net_color: Color,
           fps: u32, 
           ball: Ball, 
           lpaddle: Paddle, 
           rpaddle: Paddle) -> Game { 
        
        let mut game = Game { 
            ui: ui, 
            background_color: background_color,
            width: width,
            height: height,
            net_color: net_color,
            fps: fps, 
            ball: Rc::new(RefCell::new(ball)), 
            lpaddle: Rc::new(RefCell::new(lpaddle)), 
            rpaddle: Rc::new(RefCell::new(rpaddle)), 
            time_ball_last_speedup_ms: Option::None,
            slow_motions_remaining: 3,
            time_slow_motion_started_ms: Option::None,
            running: false, 
            resettables: Vec::new()
        };
        
        game.resettables.push(game.ball.clone());
        game.resettables.push(game.lpaddle.clone());
        game.resettables.push(game.rpaddle.clone());
        game.reset();

        return game;

    }
    
    /// Display welcome screen
    fn show_welcome_screen(&mut self) -> bool {
        self.ui.renderer.set_draw_color(self.background_color);
        self.ui.renderer.clear();
        let table_width = self.width;
        self.show_msg("Pong", table_width / 4., 100., table_width / 2., 150., 
                      Color::RGB(0xff, 0xff, 0xff));
        self.show_msg("Move the left paddle with the mouse", table_width / 4., 300., table_width / 2., 
                      18., Color::RGB(0xff, 0xff, 0xff));
        self.show_msg("Click the mouse to slow down time", table_width / 4., 330., table_width / 2., 
                      18., Color::RGB(0xff, 0xff, 0xff));
        self.show_msg("Press any key to start!", table_width / 4., 400., table_width / 2., 
                      50., Color::RGB(0xff, 0xff, 0xff));
        let mut start_game: Option<bool> = Option::None;
        self.ui.renderer.present();
        while start_game.is_none() {
            thread::sleep_ms(100);
            match self.ui.poll_event() {
                Some(event) => {
                    match event {
                        // Quit
                        Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                           start_game = Option::Some(false); 
                        },
                        // Press any other key or click the mouse to start the game 
                        Event::KeyDown { keycode: Some(..), .. } | Event::MouseButtonDown{..} => {
                           start_game = Option::Some(true); 
                        },
                        _ => {}
                    }
                },
                None => {}
            }
        }
        return start_game.unwrap();
    }

    /// Entry point into the game. Handles transition between showing the welcome screen, running
    /// the game and returning to the welcome screen.
    fn launch_then_block_until_exit(&mut self) {

        loop {
            
            // The game will exit when the user exits the welcome screen.
            if !self.show_welcome_screen() {
               return; 
            }
            
            // Execute the game loop over and over again until the user quits or someone wins.
            self.execute_game_loop();
            
            // Transition back to the welcome screen, but first revert the game to its initial 
            // state.
            self.reset();

        }
    
    }

    /// Execute the game loop over and over again until the user quits or someone wins. 
    fn execute_game_loop(&mut self) {

        self.running = true;
        let mut time_last_invocation = clock_ticks::precise_time_ms();

        while self.running {
            let time_this_invocation = clock_ticks::precise_time_ms();
            let dt_ms = time_this_invocation - time_last_invocation;
            let mut ctx = GameLoopContext::new(dt_ms as f32 / 1000.);
            self.execute_game_loop_iteration_per_frame(&mut ctx); 
            self.cap_frames_per_second(dt_ms);
            time_last_invocation = time_this_invocation;
        } 

    }
    
    // Called once per frame. 
    fn execute_game_loop_iteration_per_frame(&mut self, ctx: &mut GameLoopContext) {
        self.move_ball(ctx);
        self.move_left_paddle(ctx);
        self.move_right_paddle(ctx);
        self.draw(ctx);
        self.play_audio(ctx);
        if let Some(time_slow_motion_started_ms) = self.time_slow_motion_started_ms {
            if clock_ticks::precise_time_ms() - time_slow_motion_started_ms >= 5000 {
                self.time_slow_motion_started_ms = None;
            }
        }
        self.check_for_win(ctx);
    }
    
    // Move the left paddle based on user input. 
    fn move_left_paddle(&mut self, ctx: &mut GameLoopContext) {
        match self.ui.poll_event() {
            Some(event) => {
                match event {
                    // Quit the game.
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        self.running = false;
                    },
                    // Enter slow motion mode.
                    Event::MouseButtonDown{..} => {
                        if self.slow_motions_remaining > 0 && 
                            self.time_slow_motion_started_ms.is_none() {
                            self.slow_motions_remaining -= 1;
                            self.time_slow_motion_started_ms = Some(clock_ticks::precise_time_ms());
                        }
                    },
                    // Move left paddle with mouse. 
                    Event::MouseMotion{x,y, ..} => {
                        let y = y as f32;
                        let mut lpaddle = self.lpaddle.borrow_mut();
                        lpaddle.y = y; 
                        if lpaddle.y < 0. { 
                            lpaddle.y = 0.; 
                        } else if lpaddle.y + lpaddle.height > self.height {
                            lpaddle.y = self.height - lpaddle.height; 
                        }
                    }
                    _ => {}
                }
            },
            None => {}
        }
        ctx.layered_draw_queue[1].push(self.lpaddle.clone());
    }

    // The game moves the right paddle. 
    fn move_right_paddle(&mut self, ctx: &mut GameLoopContext) {
       
        let ball = self.ball.borrow();
        let mut rpaddle = self.rpaddle.borrow_mut(); 

        // If ball is moving toward the paddle, then track the ball. If the ball is moving away
        // from the paddle, then move toward the home position.
        let tracking_y = if ball.vx > 0. { ball.y + ball.diameter / 2.  } else { self.height / 2. }; 

        // We use non-overlapping segments of the paddle (3/4 vs 1/4) when deciding whether to move
        // the paddle up or down. Using the center of the paddle against the center of the ball is
        // very precise and will result in overshoots. Then in the next frame the paddle jumps up
        // to compensate. Using different segments, we stabilize the movement.
        if tracking_y > rpaddle.y + rpaddle.height * (3. / 4.) {

            rpaddle.y += self.mod_speed(rpaddle.speed, rpaddle.speed_multiplier) * ctx.dt_sec;

            // Guard against overshooting the ball.
            if rpaddle.y > tracking_y {
                rpaddle.y = tracking_y - rpaddle.height / 2.;
            }
            
        } else if tracking_y < rpaddle.y + rpaddle.height * (1. / 4.) {

            rpaddle.y -= self.mod_speed(rpaddle.speed, rpaddle.speed_multiplier) * ctx.dt_sec;

            // Guard against overshooting the ball.
            if rpaddle.y + rpaddle.height < tracking_y {
                rpaddle.y = tracking_y - rpaddle.height / 2.;
            }

        }

        // Guard against moving up or down beyond the screen bounds.
        if rpaddle.y < 0. { 
            rpaddle.y = 0.; 
        } else if rpaddle.y + rpaddle.height > self.height {
            rpaddle.y = self.height - rpaddle.height; 
        }
        
        // Push right paddle onto the top layer of the draw queue.
        ctx.layered_draw_queue[1].push(self.rpaddle.clone());

    }
        
    // Move the ball and deal with collisions. 
    fn move_ball(&mut self, ctx: &mut GameLoopContext) {

        let mut ball = self.ball.borrow_mut(); 
        let mut lpaddle = self.lpaddle.borrow_mut();
        let mut rpaddle = self.rpaddle.borrow_mut();
        
        // Calculate the tentative new ball coordinates based (before we take into account 
        // travelling beyond the screen bounds or hitting a wall or paddle).
        let mut new_ball_x = ball.x + self.mod_speed(ball.vx, ball.speed_multiplier) * ctx.dt_sec;
        let mut new_ball_y = ball.y + self.mod_speed(ball.vy, ball.speed_multiplier) * ctx.dt_sec;

        // If the ball hit the top or bottom wall, then the angle of deflection will be equal
        // to the angle of incidence. Instead of calculating the angle and new x and y coordinates,
        // we keep the x coordinate unchanged, but reverse the horizontal direction of the 
        // distance travelled beyond the wall bounds. Next, we also reverse the horizontal
        // velocity.
        if new_ball_y < 0. {
            new_ball_y = -new_ball_y;
            ball.vy = -ball.vy;
            ctx.audible_queue.push(self.ui.ping_sound.clone());
        } else if new_ball_y + ball.diameter >= self.height { 
            new_ball_y = self.height - (new_ball_y + ball.diameter - self.height) - ball.diameter;
            ball.vy = -ball.vy;
            ctx.audible_queue.push(self.ui.ping_sound.clone());
        } 

        let mut bounce_that_allows_speedup: bool = false;

        // If the ball hit the left or right paddle. 
        if new_ball_x < lpaddle.x + lpaddle.width && ball.x >= lpaddle.x + lpaddle.width {

            // The x position indicates a hit. Still need to check the y position...
            let bounce_x = lpaddle.x + lpaddle.width; 

            // The gradient of the straight line from (ball.x,ball.y) to (bounce_x,bounce_y) to
            // (new_ball_x,new_ball_y) stays constant, so we can use that to find the value of the
            // top left corner of the ball when it bounces.  
            let bounce_y = (new_ball_y - ball.y) / (new_ball_x - ball.x) * (bounce_x - ball.x) + ball.y;

            if bounce_y + ball.diameter >= lpaddle.y && bounce_y <= lpaddle.y + lpaddle.height {
               
                // The y position indicates a hit also! 
                
                // Calculate where the center of the ball hit relative to the center of the paddle.
                let relative_y = (lpaddle.y + lpaddle.height / 2.) - (bounce_y + ball.diameter / 2.);
                
                // Use the ratio of the bounce position to half the height of the paddle as an
                // angle multiplier.
                let bounce_angle_multiplier = (relative_y / (lpaddle.height / 2.)).abs();
                let bounce_angle = bounce_angle_multiplier * ball.max_bounce_angle;

                // Calculate completely new x and y velocities using simple trigonometric
                // identities.
                ball.vx = ball.speed * bounce_angle.cos();
                ball.vy = ball.speed * bounce_angle.sin() * if ball.vy < 0. {-1.} else {1.}; 

                // The imaginary distance travelled beyond the paddle equals the actual distance
                // travelled after the bounce. To calculate the time it took to travel the distance
                // after the bounce, we can take the total time and multiply that by a fraction
                // equal to the ratio of the distance travelled beyond the ball to the total 
                // distance travelled. This would equal the ratio of the hypotenuses of two similar
                // triangles. We don't want to calculate the hypotenuses, but there is a shortcut:
                // We can use the fact that the ratio of corresponding sides for similar triangles
                // are always the same... instead of using the ratio of the hypotenuses, we can use
                // the ratio of the opposite sides. In this case, that'd be the ratio of the y
                // distances travelled:
                let bounce_dt_sec = ctx.dt_sec * (new_ball_y - bounce_y) / (new_ball_y - ball.y);
                new_ball_x = bounce_x + ball.vx * bounce_dt_sec;
                new_ball_y = bounce_y + ball.vy * bounce_dt_sec;
                ctx.audible_queue.push(self.ui.pong_sound.clone()); 

                // May speedup after hitting the left paddle.
                bounce_that_allows_speedup = true;
            }

        } else if new_ball_x + ball.diameter > rpaddle.x && ball.x + ball.diameter <= rpaddle.x {
            
            // The logic around hitting the right paddle is very similar to that for hitting the
            // left paddle.

            let bounce_x = rpaddle.x - ball.diameter; 
            let bounce_y = (new_ball_y - ball.y) / (new_ball_x - ball.x) * (bounce_x - ball.x) + ball.y;

            if bounce_y + ball.diameter  >= rpaddle.y && bounce_y <= rpaddle.y + rpaddle.height {
                let relative_y = (rpaddle.y + rpaddle.height / 2.) - (bounce_y + ball.diameter / 2.);
                let bounce_angle_multiplier = (relative_y / (rpaddle.height / 2.)).abs();
                let bounce_angle = bounce_angle_multiplier * ball.max_bounce_angle;
                ball.vx = ball.speed * bounce_angle.cos() * -1.;
                ball.vy = ball.speed * bounce_angle.sin() * if ball.vy < 0. {-1.} else {1.}; 
                let bounce_dt_sec = ctx.dt_sec * (new_ball_y - bounce_y) / (new_ball_y - ball.y);
                new_ball_x = bounce_x + ball.vx * bounce_dt_sec;
                new_ball_y = bounce_y + ball.vy * bounce_dt_sec;
                ctx.audible_queue.push(self.ui.ping_sound.clone()); 

                // May speedup after hitting the right paddle.
                bounce_that_allows_speedup = true;
            }

        } 

        // If the ball hit the left or right wall, then the angle of deflection will be equal
        // to the angle of incidence. Instead of calculating the angle and new x and y coordinates,
        // we keep the x coordinate unchanged, but reverse the vertical direction of the distance 
        // travelled beyond the wall bounds. Next, we also reverse the vertical velocity.
        if new_ball_x < 0. { 
            new_ball_x = -new_ball_x;
            ball.vx = -ball.vx;
            // Right player scored.
            rpaddle.score += 1;
            ctx.audible_queue.push(self.ui.ping_sound.clone()); 
            bounce_that_allows_speedup = true;
        } else if new_ball_x + ball.diameter > self.width { 
            new_ball_x = self.width - (new_ball_x + ball.diameter - self.width) - ball.diameter;
            ball.vx = -ball.vx;
            // Left player scored.
            lpaddle.score += 1;
            ctx.audible_queue.push(self.ui.ping_sound.clone()); 
            bounce_that_allows_speedup = true;
        } 

        ball.x = new_ball_x;
        ball.y = new_ball_y;
        ctx.layered_draw_queue[1].push(self.ball.clone());

        // Speedup the ball periodically until max speed reached. 
        let time_now_ms = clock_ticks::precise_time_ms();
        match self.time_ball_last_speedup_ms {
            None => {   
                self.time_ball_last_speedup_ms = Option::Some(time_now_ms);
            },
            Some(time_ball_last_speedup_ms) => {
                if time_now_ms - time_ball_last_speedup_ms > 15000 && 
                    bounce_that_allows_speedup &&
                    ball.speed_multiplier < 1.5 && self.time_slow_motion_started_ms.is_none() {
                    ball.speed_multiplier += 0.1;
                    rpaddle.speed_multiplier += 0.1;
                    self.time_ball_last_speedup_ms = Option::Some(time_now_ms);
                }
            }
        }

    }

    fn draw(&mut self, ctx: &mut GameLoopContext) {
        
        // Set background color and clear the screen.
        self.ui.renderer.set_draw_color(self.background_color);
        self.ui.renderer.clear();
        
        //let lpaddle = self.lpaddle.borrow();
        //self.draw_score(lpaddle.score, lpaddle.color, self.width / 2. - 100., 5.);
        //let rpaddle = self.rpaddle.borrow();
        //self.draw_score(rpaddle.score, rpaddle.color, self.width / 2. + 20., 5.);
        
        self.draw_net();
        for layer in ctx.layered_draw_queue.iter() {
            for d in layer.iter() {
                d.borrow().draw(&mut self.ui);
            }
        }
        let mut png_texture = {
            let png_path = Path::new("assets/images/turtle.png");
            self.ui.renderer.load_texture(png_path).unwrap() 
        };
        let mut x = 300;
        let y = 550;
        let w = 15;
        for i in 0..3 {
            if i < self.slow_motions_remaining {
                match self.lpaddle.borrow().color {
                    Color::RGB(r,g,b) => png_texture.set_color_mod(r,g,b),
                    _ => {}
                }
            } else {
                png_texture.set_color_mod(0x69,0x69,0x69);
            }
            let target = Rect::new_unwrap(x, y, w, 20);
            self.ui.renderer.copy(&png_texture, None, Some(target));
            x += w as i32 + 5;
        }
        self.ui.renderer.present();
    }

    fn draw_score(&mut self, score: u32, color: Color, x: f32, y: f32) {
        let formatted_score = format!("{:^3}", score);
        let formatted_score_ref: &str = formatted_score.as_ref();
        let surface = self.ui.pixel_font.render(formatted_score_ref, sdl2_ttf::blended(color)).unwrap();
        let texture = self.ui.renderer.create_texture_from_surface(&surface).unwrap();
        let target = Rect::new_unwrap(x as i32, y as i32, 80, 60);
        self.ui.renderer.copy(&texture, None, Some(target));
    }

    fn draw_net(&mut self) {

        let num_net_dots = 20;
        let num_net_gaps = num_net_dots - 1;
        let net_dot_width = 10.;
        let net_dot_height = self.height / (num_net_dots + num_net_gaps) as f32;

        for i in 0..num_net_dots + num_net_gaps + 1 {
            let net_dot_x = self.width / 2. - net_dot_width / 2.;
            let net_dot_y = i as f32 * net_dot_height; 
            self.ui.renderer.set_draw_color(if i % 2 == 0 {self.net_color} else {self.background_color});
            let net_dot_rect = Rect::new_unwrap(net_dot_x as i32, net_dot_y as i32, 
                                                net_dot_width as u32, net_dot_height as u32);
            self.ui.renderer.fill_rect(net_dot_rect);
        }

    }

    fn play_audio(&mut self, ctx: &mut GameLoopContext) {
        for a in ctx.audible_queue.iter() {
            a.play(1);
        }
    }

    fn check_for_win(&mut self, ctx: &mut GameLoopContext) {

        let mut msg: Option<&str> = Option::None;
        let points_to_win = 5;

        if self.lpaddle.borrow().score >= points_to_win {
            msg = Option::Some("You win!");
        } else if self.lpaddle.borrow().score >= points_to_win {
            msg = Option::Some("I win!");
        }
        
        // We have a winner.
        if let Some(msg) = msg {
            self.running = false;
            self.ui.renderer.set_draw_color(self.background_color);
            self.ui.renderer.clear();
            let x = self.width /2. /2.;
            let y = self.height /2. - 100.;
            let color = Color::RGB(0xff, 0xff, 0xff);
            let width = self.width / 2.;
            let height = 60.;
            self.show_msg(msg, x, y, width, height, color);
            self.ui.renderer.present();
            thread::sleep_ms(1000);
        }

    }

    fn show_msg(&mut self, msg: &str, x: f32, y: f32, width: f32, height: f32, color: Color) {

        let surface = self.ui.pixel_font.render(msg, sdl2_ttf::blended(color)).unwrap();
        let texture = self.ui.renderer.create_texture_from_surface(&surface).unwrap();
        let target = Rect::new_unwrap(x as i32, y as i32, width as u32, height as u32);
        self.ui.renderer.copy(&texture, None, Some(target));

    }

    /// Modify speed by applying indicated multiplier. Additionally, if a slow motion turn is
    /// active, then halve the resulting speed.
    fn mod_speed(&self, speed: f32, speed_multiplier: f32) -> f32 {

        let mut modified_speed = speed * speed_multiplier;

        match self.time_slow_motion_started_ms {
            Some(time_slow_motion_started_ms) => {
                modified_speed *= 0.5;
            },
            None => {}
        }

        return modified_speed;

    }

    /// Ensure we run no faster than the desired fps by introducing a delay if necessary.
    fn cap_frames_per_second(&self, duration_of_last_frame_execution_ms: u64) {

        let max_delay_ms = 1000 / self.fps as u64;
        if max_delay_ms > duration_of_last_frame_execution_ms {
            thread::sleep_ms((max_delay_ms - duration_of_last_frame_execution_ms) as u32);
        }

    }
    
}

impl Resettable for Game {

    fn reset(&mut self) {

        self.time_ball_last_speedup_ms = Option::None;
        self.slow_motions_remaining = 3;
        self.time_slow_motion_started_ms = Option::None;

        for r in self.resettables.iter() {
            r.borrow_mut().reset();
        }

    } 

}

/// Assemble the game components and wire them together using dependency injection. 
fn build() -> Game {

    // Screen dimensions and background color.
    let screen_width = 800.;
    let screen_height = 600.;
    let screen_background_color = Color::RGB(0x25, 0x25, 0x25); 
    
    // Initialize SDL and capture the window renderer for later use. 
    let sdl_ctx = sdl2::init().unwrap();
    let video_subsystem = sdl_ctx.video().unwrap();
    let window = video_subsystem.window("pong", screen_width as u32, screen_height as u32)
        .position_centered()
        .build()
        .unwrap();
    let renderer = window.renderer().build().unwrap();
    
    //sdl_ctx.mouse().set_relative_mouse_mode(true);
    //sdl_ctx.mouse().show_cursor(false);

    // Initialize sdl_image for PNG image rendering. 
    sdl2_image::init(INIT_PNG);
    
    // Initialize sdl_ttf for true type font rendering, then load and store the fonts we will
    // use in the game.
    let ttf_ctx = sdl2_ttf::init().unwrap();
    let font_path = Path::new("assets/fonts/pixel.ttf");
    let font = sdl2_ttf::Font::from_file(font_path, 128).unwrap();

    // Initialize sdl_mixer for audio playback, then load and store the sounds we will use
    // in the game.
    let sdl_audio = sdl_ctx.audio().unwrap();
    sdl2_mixer::open_audio(DEFAULT_FREQUENCY, sdl2_mixer::AUDIO_S16LSB, 2, 1024);
    let ping_sound_path = Path::new("assets/sounds/ping.wav");
    let ping_sound = sdl2_mixer::Music::from_file(ping_sound_path).unwrap();
    let pong_sound_path = Path::new("assets/sounds/pong.wav");
    let pong_sound = sdl2_mixer::Music::from_file(pong_sound_path).unwrap();

    // Package the media we will use later on in the UI type. 
    let ui = Ui::new(sdl_ctx, renderer, ttf_ctx, font, sdl_audio, ping_sound, pong_sound);

    // Our ball will launch from the center of the screen.
    let ball = Ball::new(Color::RGB(0xff, 0xcc, 0x00), 
                         screen_width / 2., 
                         screen_height / 2., 
                         11., 
                         500.,
                         f32::consts::PI * 50. / 180.,
                         f32::consts::PI * 45. / 180.); 
    
    
    // Common ball properties.
    let paddle_x_offset = 4.;
    let paddle_width = 5.;
    let paddle_height = 60.;
    let paddle_initial_y = (screen_height - paddle_height) / 2.;
    
    // The left paddle starts in the left center of the screen and represents the human player.
    let left_paddle = Paddle::new(Color::RGB(0xf6, 0xf4, 0xda), 
                                  paddle_x_offset, 
                                  paddle_initial_y,
                                  paddle_width,
                                  paddle_height,
                                  0.); // There is no restriction on how fast the human may
                                      // may move the paddle.

    // The right paddle start in the right center of the screen and represents the computer
    // player.
    let right_paddle = Paddle::new(Color::RGB(0xd9, 0xe2, 0xe1), 
                                  screen_width - (paddle_x_offset + paddle_width), 
                                  paddle_initial_y,
                                  paddle_width,
                                  paddle_height,
                                  300.);
    
    // Assemble and return the game. We're ready to play!
    return Game::new(ui,
                     screen_background_color,
                     screen_width,
                     screen_height,
                     Color::RGB(0xff, 0xff, 0xff),
                     40,
                     ball,
                     left_paddle,
                     right_paddle);

}
    
fn main() {
    build().launch_then_block_until_exit();
}
