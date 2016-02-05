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
use sdl2_ttf::{Font, Hinting, Sdl2TtfContext}; 

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
    sdl_audio: AudioSubsystem, 
    ping_sound: Rc<Music>,
    pong_sound: Rc<Music>

}

impl Ui {

    fn new(sdl_ctx: Sdl, 
           renderer: Renderer<'static>, 
           ttf_ctx: Sdl2TtfContext, 
           sdl_audio: AudioSubsystem, 
           ping_sound: Music, 
           pong_sound: Music) -> Ui {

        return Ui { 
            sdl_ctx: sdl_ctx, 
            renderer: renderer,
            ttf_ctx: ttf_ctx,
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

struct ScoreCard {
   
    color: Color,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    font: Rc<Font>,
    score: i32
    
}

impl ScoreCard {

    fn new(color: Color, x: f32, y: f32, width: f32, height: f32, font: Rc<Font>) -> ScoreCard {
    
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

struct Net {

    color: Color,   
    x: f32,            // x pixel coordinate of top left corner  
    dot_width: f32,
    dot_height: f32,
    num_dots: i32

}

impl Net {
    
    fn new(color: Color, x: f32, dot_width: f32, dot_height: f32, num_dots: i32) -> Net {
    
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
                let dot_rect = Rect::new_unwrap(dot_x as i32, 
                                                dot_y as i32, 
                                                self.dot_width as u32, 
                                                self.dot_height as u32);

                ui.renderer.fill_rect(dot_rect);
            
            }

        }
    
    }
}

// The ball is rendered as a circle, but treated as a square to simplify game mechanics. 
struct Ball {

    color: Color,                   
    initial_x: f32,                 // The initial x location. Stored so that we can reset the ball.
    initial_y: f32,                 // The initial y location. Stored so that we can reset the ball.
    x: f32,                         // x pixel coordinate of top left corner.
    y: f32,                         // y pixel coordinate of top left corner.
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
    x: f32,                 // x pixel coordinate of top left corner
    y: f32,                 // y pixel coordinate of top left corner
    width: f32,     
    height: f32,    
    speed: f32,             // Speed in pixels per second. Never changes. 
    speed_multiplier: f32   // Used to adjust the speed.

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
    fps: u32,
    net: Rc<RefCell<Net>>,
    ball: Rc<RefCell<Ball>>,
    lpaddle: Rc<RefCell<Paddle>>,
    rpaddle: Rc<RefCell<Paddle>>,
    lscore_card: Rc<RefCell<ScoreCard>>,
    rscore_card: Rc<RefCell<ScoreCard>>,
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
           fps: u32, 
           net: Net,
           ball: Ball, 
           lpaddle: Paddle, 
           rpaddle: Paddle,
           lscore_card: ScoreCard,
           rscore_card: ScoreCard) -> Game { 
        
        let mut game = Game { 
            ui: ui, 
            background_color: background_color,
            width: width,
            height: height,
            fps: fps, 
            net: Rc::new(RefCell::new(net)), 
            ball: Rc::new(RefCell::new(ball)), 
            lpaddle: Rc::new(RefCell::new(lpaddle)), 
            rpaddle: Rc::new(RefCell::new(rpaddle)), 
            lscore_card: Rc::new(RefCell::new(lscore_card)), 
            rscore_card: Rc::new(RefCell::new(rscore_card)), 
            time_ball_last_speedup_ms: Option::None,
            slow_motions_remaining: 3,
            time_slow_motion_started_ms: Option::None,
            running: false, 
            resettables: Vec::new()
        };
        
        game.resettables.push(game.ball.clone());
        game.resettables.push(game.lpaddle.clone());
        game.resettables.push(game.rpaddle.clone());
        game.resettables.push(game.lscore_card.clone());
        game.resettables.push(game.rscore_card.clone());
        game.reset();

        return game;

    }
    
    /// Display welcome screen containing title, game instructions and credits while playing
    /// funky music. The music stops when the game starts. 
    fn show_welcome_screen(&mut self) -> bool {

        // Play music in the background.
        let music_path = Path::new("assets/sounds/more_monkey_island_band.wav");
        let music = sdl2_mixer::Music::from_file(music_path).unwrap();
        music.play(-1);
        
        // Draw background.
        self.ui.renderer.set_draw_color(self.background_color);
        self.ui.renderer.clear();
        
        // Draw game title.
        let title_font_path = Path::new("assets/fonts/djb_pokey_dots.ttf");
        let title_font = sdl2_ttf::Font::from_file(title_font_path, 72).unwrap();
        let mut title_x = self.width / 2. - 95. - 95. -47.;
        let title_y = 100.;
        self.draw_text(&title_font, "P", Color::RGB(0x03, 0x91, 0xcf), title_x, title_y);
        title_x += 95.;
        self.draw_text(&title_font, "O", Color::RGB(0xf6, 0x77, 0x34), title_x, title_y);
        title_x += 95.;
        self.draw_text(&title_font, "N", Color::RGB(0xfc, 0xef, 0x6d), title_x, title_y);
        title_x += 95.;
        self.draw_text(&title_font, "G", Color::RGB(0x6f, 0xc3, 0x2d), title_x, title_y);
        title_x += 95.;
        self.draw_text(&title_font, "O", Color::RGB(0xf0, 0x3b, 0x32), title_x, title_y);
       
        // Draw instructions. 
        let instruction_font_path = Path::new("assets/fonts/coffee_time.ttf");
        let instruction_font = sdl2_ttf::Font::from_file(instruction_font_path, 26).unwrap();
        self.draw_centered_text(&instruction_font, "Move the left paddle with the mouse...", 
                                Color::RGB(0xff, 0xff, 0xff), 250.);
        self.draw_centered_text(&instruction_font, "Left click the mouse to slow down time...", 
                       Color::RGB(0xff, 0xff, 0xff), 300.);

        // Press any key to start. 
        let start_font_path = Path::new("assets/fonts/kghappysolid.ttf");
        let start_font = sdl2_ttf::Font::from_file(start_font_path, 39).unwrap();
        self.draw_centered_text(&start_font, "PRESS ANY KEY TO START!", 
                                Color::RGB(0xec, 0x42, 0x35), 380.);
        
        // Draw credits. 
        let credit_font_path = Path::new("assets/fonts/kg_cold_coffee.ttf");
        let credit_font = sdl2_ttf::Font::from_file(credit_font_path, 12).unwrap();
        self.draw_centered_text(&credit_font, "Programming by Wickus Martin", 
                                Color::RGB(0xff, 0xff, 0xff), 500.);
        self.draw_centered_text(&credit_font, "Music by Eric Matyas", 
                                Color::RGB(0xff, 0xff, 0xff), 530.);

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
        sdl2_mixer::Music::halt();
        return start_game.unwrap();
    }

    /// Draw text to the screen. The width and height are calculated from the font supplied.
    /// The position is specified as a top y location only. The x location is calculated
    /// so that the text centers on the screen.
    fn draw_centered_text(&mut self, font: &Font, text: &str, color: Color, y: f32) {
        let (width, _) = font.size(text).unwrap(); 
        let x = self.width / 2. - (width as f32) / 2.;
        self.draw_text(font, text, color, x, y);
    }


    /// Draw text to the screen. The width and height are calculated from the font supplied.
    fn draw_text(&mut self, font: &Font, text: &str, color: Color, x: f32, y: f32) {
        let surface = font.render(text, sdl2_ttf::blended(color)).unwrap();
        let texture = self.ui.renderer.create_texture_from_surface(&surface).unwrap();
        let (width, height) = font.size(text).unwrap(); 
        let target = Rect::new_unwrap(x as i32, y as i32, width as u32, height as u32);
        self.ui.renderer.copy(&texture, None, Some(target));
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

        // The running flag is stored as a game wide field. This allows the flag to be changed
        // at any point in the game. A method checking for user input can thus set the flag to
        // false when the user indicates they want to quit. The same goes for a method that checks
        // whether someone has won.
        self.running = true;
        let mut time_last_invocation = clock_ticks::precise_time_ms();

        while self.running {
            let time_this_invocation = clock_ticks::precise_time_ms();

            // The delta time in millis is used to measure the duration of frame execution
            // so that we can update the screen based on the time that has elapsed since the last
            // frame was rendered. It is also used to cap the frame rate.
            let dt_ms = time_this_invocation - time_last_invocation;
            let mut ctx = GameLoopContext::new(dt_ms as f32 / 1000.);
            self.execute_game_loop_iteration_per_frame(&mut ctx); 
            self.cap_frames_per_second(dt_ms);
            time_last_invocation = time_this_invocation;
        } 

    }
    
    /// Called once per frame. Essentially, an iteration of the game loop. 
    fn execute_game_loop_iteration_per_frame(&mut self, ctx: &mut GameLoopContext) {

        
        // Move objects. The left paddle is moved based on user input. 
        self.move_ball(ctx);
        self.move_left_paddle(ctx);
        self.move_right_paddle(ctx);

        
        // Draw objects.
        self.draw(ctx);

        // Play audio.
        self.play_audio(ctx);

        // End slow motion mode if duration has elapsed.
        if let Some(time_slow_motion_started_ms) = self.time_slow_motion_started_ms {
            if clock_ticks::precise_time_ms() - time_slow_motion_started_ms >= 5000 {
                self.time_slow_motion_started_ms = None;
            }
        }

        // Check to see if either the human (left paddle) or computer (right paddle) has won.
        // todo. fix check for win.
        self.check_for_win(ctx);
    }
    
    /// Move the left paddle based on user input. 
    fn move_left_paddle(&mut self, ctx: &mut GameLoopContext) {
        match self.ui.poll_event() {
            Some(event) => {
                match event {
                    // Quit the game and return back to the welcome screen.
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
                        // Guard against moving up or down beyond the screen bounds.
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

    /// The computer player moves the right paddle. 
    fn move_right_paddle(&mut self, ctx: &mut GameLoopContext) {
       
        let ball = self.ball.borrow();
        let mut rpaddle = self.rpaddle.borrow_mut(); 

        // If ball is moving toward the paddle, then track the ball. If the ball is moving away
        // from the paddle, then move toward the home position.
        let tracking_y = if ball.vx > 0. {ball.y + ball.diameter / 2.} else {self.height / 2.}; 

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
        
    /// Move the ball and deal with collisions. 
    fn move_ball(&mut self, ctx: &mut GameLoopContext) {

        let mut ball = self.ball.borrow_mut(); 
        let mut lpaddle = self.lpaddle.borrow_mut();
        let mut rpaddle = self.rpaddle.borrow_mut();
        
        // Calculate the tentative new ball coordinates based on the time since the last movement
        // and the current velocity of the ball. The new position is tentative since we still need
        // to account for collisions... hitting a wall or paddle.
        let mut new_ball_x = ball.x + self.mod_speed(ball.vx, ball.speed_multiplier) * ctx.dt_sec;
        let mut new_ball_y = ball.y + self.mod_speed(ball.vy, ball.speed_multiplier) * ctx.dt_sec;

        // If the ball hit the top or bottom wall, then the angle of deflection will be equal
        // to the angle of incidence. Instead of calculating the angle and new x and y coordinates,
        // we keep the x coordinate unchanged, but reverse the horizontal direction of the distance 
        // travelled beyond the wall bounds. Next, we also reverse the horizontal velocity.
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
            
            // The logic around hitting the right paddle is essentially the same as that for 
            // hitting the left paddle.

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
            self.rscore_card.borrow_mut().score += 1;
            ctx.audible_queue.push(self.ui.ping_sound.clone()); 
            bounce_that_allows_speedup = true;
        } else if new_ball_x + ball.diameter > self.width { 
            new_ball_x = self.width - (new_ball_x + ball.diameter - self.width) - ball.diameter;
            ball.vx = -ball.vx;
            // Left player scored.
            self.lscore_card.borrow_mut().score += 1;
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
        
        ctx.layered_draw_queue[0].push(self.net.clone());
        ctx.layered_draw_queue[0].push(self.lscore_card.clone());
        ctx.layered_draw_queue[0].push(self.rscore_card.clone());

        // Higher layers are drawn on top of lower layers. Allows us to for instance, ensure the 
        // ball passes over the top of the net instead of underneath it.
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
                match Color::RGB(0x6f, 0xc3, 0x2d) {
                    Color::RGB(r,g,b) => png_texture.set_color_mod(r,g,b),
                    _ => {}
                }
            } else {
                png_texture.set_color_mod(0x69, 0x69, 0x69);
            }
            let target = Rect::new_unwrap(x, y, w, 20);
            self.ui.renderer.copy(&png_texture, None, Some(target));
            x += w as i32 + 5;
        }
        self.ui.renderer.present();
    }

    fn play_audio(&mut self, ctx: &mut GameLoopContext) {
        for a in ctx.audible_queue.iter() {
            a.play(1);
        }
    }

    fn check_for_win(&mut self, ctx: &mut GameLoopContext) {

        let mut msg: Option<&str> = Option::None;
        let points_to_win = 5;

        if self.lscore_card.borrow().score >= points_to_win {
            msg = Option::Some("You win!");
        } else if self.rscore_card.borrow().score >= points_to_win {
            msg = Option::Some("I win!");
        }
        
        // There is a win message, therefore there is a winner. 
        if let Some(msg) = msg {
            self.running = false;
            self.ui.renderer.set_draw_color(self.background_color);
            self.ui.renderer.clear();
            let font_path = Path::new("assets/fonts/kghappysolid.ttf");
            let font = sdl2_ttf::Font::from_file(font_path, 60).unwrap();
            let (_, height) = font.size(msg).unwrap();
            let y = self.height / 2. - (height as f32) / 2.;
            self.draw_centered_text(&font, msg, Color::RGB(0xfc, 0xef, 0x6d), y);
            self.ui.renderer.present();
            thread::sleep_ms(1000);
        }

    }

    /// Modify speed by applying indicated multiplier. Additionally, if a slow motion turn is
    /// active, then halve the resulting speed.
    fn mod_speed(&self, speed: f32, speed_multiplier: f32) -> f32 {

        let mut modified_speed = speed * speed_multiplier;

        match self.time_slow_motion_started_ms {
            Some(_) => {
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

        // Reset slow motion status.
        self.time_ball_last_speedup_ms = Option::None;
        self.slow_motions_remaining = 3;
        self.time_slow_motion_started_ms = Option::None;

        // Reset objects.
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
    let window = video_subsystem.window("pongo", screen_width as u32, screen_height as u32)
        .position_centered()
        .build()
        .unwrap();
    let renderer = window.renderer().build().unwrap();
    
    //sdl_ctx.mouse().set_relative_mouse_mode(true);
    //sdl_ctx.mouse().show_cursor(false);

    // Initialize sdl_image for PNG image rendering. 
    sdl2_image::init(INIT_PNG);
    
    // Initialize sdl_ttf for true type font rendering.
    let ttf_ctx = sdl2_ttf::init().unwrap();

    // Initialize sdl_mixer for audio playback, then load and store the sounds we will use
    // in the game.
    let sdl_audio = sdl_ctx.audio().unwrap();
    sdl2_mixer::open_audio(DEFAULT_FREQUENCY, sdl2_mixer::AUDIO_S16LSB, 2, 1024);
    let ping_sound_path = Path::new("assets/sounds/ping.wav");
    let ping_sound = sdl2_mixer::Music::from_file(ping_sound_path).unwrap();
    let pong_sound_path = Path::new("assets/sounds/pong.wav");
    let pong_sound = sdl2_mixer::Music::from_file(pong_sound_path).unwrap();

    // Package the media we will use later on in the UI type. 
    let ui = Ui::new(sdl_ctx, renderer, ttf_ctx, sdl_audio, ping_sound, pong_sound);

    // The net will run vertically across the center of the screen.
    let net = Net::new(Color::RGB(0xff, 0xff, 0xff),
                       screen_width / 2. - 10. / 2.,
                       10.,
                       screen_height / (2 * 20 - 1) as f32,
                       20);

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
    
    // The left paddle starts in the left center of the screen and is controlled by the human
    // player.
    let left_paddle = Paddle::new(Color::RGB(0x03, 0x91, 0xcf), 
                                  paddle_x_offset, 
                                  paddle_initial_y,
                                  paddle_width,
                                  paddle_height,
                                  0.); // There is no restriction on how fast the human player may
                                       // move the paddle.

    // The right paddle start in the right center of the screen and is controlled by the computer
    // palyer.
    let right_paddle = Paddle::new(Color::RGB(0xeb, 0x4e, 0x3d), 
                                  screen_width - (paddle_x_offset + paddle_width), 
                                  paddle_initial_y,
                                  paddle_width,
                                  paddle_height,
                                  300.);
  
    let font_path = Path::new("assets/fonts/pixel.ttf");
    let font = Rc::new(sdl2_ttf::Font::from_file(font_path, 128).unwrap());
    let score_board_width = screen_width / 2. - 100.;
    let score_board_height = 65.;
    let score_board_x = screen_width / 2. - score_board_width / 2.;
    let score_board_y = 5.;
    let score_card_width = 80.;
    let score_card_height = 60.;

    let lscore_card = ScoreCard::new(Color::RGB(0x03, 0x91, 0xcf),
                                     score_board_x + 5.,
                                     score_board_y + 5.,
                                     score_card_width,
                                     score_card_height,
                                     font.clone());

    let rscore_card = ScoreCard::new(Color::RGB(0xeb, 0x4e, 0x3d),
                                     score_board_x + score_board_width - 5. - score_card_width,
                                     score_board_y + 5.,
                                     score_card_width,
                                     score_card_height,
                                     font.clone());

    // Assemble and return the game. We're ready to play!
    return Game::new(ui,
                     screen_background_color,
                     screen_width,
                     screen_height,
                     40,
                     net,
                     ball,
                     left_paddle,
                     right_paddle,
                     lscore_card,
                     rscore_card);

}
    
fn main() {
    build().launch_then_block_until_exit();
}

