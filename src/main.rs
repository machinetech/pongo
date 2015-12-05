extern crate clock_ticks;
extern crate rand;
extern crate sdl2;
extern crate sdl2_gfx;
extern crate sdl2_mixer;
extern crate sdl2_ttf;

use rand::distributions::{IndependentSample, Range};

use sdl2::{AudioSubsystem, Sdl};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

use std::cell::RefCell;
use std::f32;
use std::path::Path;
use std::rc::Rc;
use std::thread;
use std::vec::Vec;

use sdl2_gfx::primitives::DrawRenderer;
use sdl2_mixer::{AUDIO_S16LSB, DEFAULT_FREQUENCY, Music}; 
use sdl2_ttf::{Font, Sdl2TtfContext}; 

struct Ui {
    sdl_ctx: Sdl,
    renderer: Renderer<'static>,
    ttf_ctx: Sdl2TtfContext,
    font: Font,
    sdl_audio: AudioSubsystem, 
    ping_sound: Rc<Music>,
    pong_sound: Rc<Music>
}

impl Ui {
    fn new(sdl_ctx: Sdl, renderer: Renderer<'static>, ttf_ctx: Sdl2TtfContext, font: Font,
           sdl_audio: AudioSubsystem, ping_sound: Music, pong_sound: Music) -> Ui {
        Ui { 
            sdl_ctx: sdl_ctx, 
            renderer: renderer,
            ttf_ctx: ttf_ctx,
            font: font,
            sdl_audio: sdl_audio,
            ping_sound: Rc::new(ping_sound),
            pong_sound: Rc::new(pong_sound)
        }  
    } 

    fn poll_event(&self) -> Option<Event> {
        let mut event_pump = self.sdl_ctx.event_pump().unwrap();
        return event_pump.poll_event();
    }
}

trait Drawable {
    fn draw(&self, ui: &mut Ui); 
}

struct Table {
    color: Color,
    width: f32,
    height: f32,
    net_color: Color,
    lscore: u32,
    lscore_color: Color,
    rscore: u32,
    rscore_color: Color
}

impl Table {
    fn new(color: Color, width: f32, height: f32, net_color: Color, 
           lscore_color: Color, rscore_color: Color) -> Table {
        Table {
            color: color,
            width: width,
            height: height,
            net_color: net_color,
            lscore: 0,
            rscore: 0,
            lscore_color: lscore_color,
            rscore_color: rscore_color
        }
    }

    fn draw_score(&self, ui: &mut Ui, score: u32, color: Color, x: f32, y: f32) {
        let formatted_score = format!("{:^3}", score);
        let formatted_score_ref: &str = formatted_score.as_ref();
        let surface = ui.font.render(formatted_score_ref, sdl2_ttf::blended(color)).unwrap();
        let texture = ui.renderer.create_texture_from_surface(&surface).unwrap();
        let target = Rect::new_unwrap(x as i32, y as i32, 80, 60);
        ui.renderer.copy(&texture, None, Some(target));
    }

    fn draw_net(&self, ui: &mut Ui) {
        let num_net_dots = 20;
        let num_net_gaps = num_net_dots - 1;
        let net_dot_width = 10.;
        let net_dot_height = self.height / (num_net_dots + num_net_gaps) as f32;
        for i in 0..num_net_dots + num_net_gaps + 1 {
            let net_dot_x = self.width / 2. - net_dot_width / 2.;
            let net_dot_y = i as f32 * net_dot_height; 
            ui.renderer.set_draw_color(if i % 2 == 0 {self.net_color} else {self.color});
            let net_dot_rect = Rect::new_unwrap(net_dot_x as i32, net_dot_y as i32, 
                                                net_dot_width as u32, net_dot_height as u32);
            ui.renderer.fill_rect(net_dot_rect);
        }
    }
}

impl Drawable for Table {
    
    fn draw(&self, ui: &mut Ui) {
        ui.renderer.set_draw_color(self.color);
        ui.renderer.clear();
        self.draw_score(ui, self.lscore, self.lscore_color, self.width / 2. - 100., 20.);
        self.draw_score(ui, self.rscore, self.rscore_color, self.width / 2. + 20., 20.);
        self.draw_net(ui);
    }

}

struct Ball {
    color: Color,
    x: f32,                         // x pixel co-ordinate of top left corner
    y: f32,                         // y pixel co-ordinate of top left corner
    diameter: f32,    
    speed: f32,                     // pixels per second 
    speed_multiplier: f32,
    vx: f32,                        // pixels per second
    vy: f32,                        // pixels per second
    max_paddle_bounce_angle: f32    // Angle up or down from imaginary horizontal line running 
                                    // perpendicular to the paddle. 
}

impl Ball {
    fn new(color: Color, x: f32, y: f32, diameter: f32, speed: f32, vx: f32, 
               vy: f32, max_paddle_bounce_angle: f32) -> Ball {
        Ball { 
            color: color, 
            x: x, 
            y: y, 
            diameter: diameter, 
            speed: speed, 
            speed_multiplier: 1.0,
            vx: vx, 
            vy: vy,
            max_paddle_bounce_angle: max_paddle_bounce_angle
        }
    }
}

impl Drawable for Ball {
    fn draw(&self, ui: &mut Ui) {
        ui.renderer.filled_circle((self.x + self.diameter/2.) as i16, 
                                  (self.y + self.diameter/2.) as i16, 
                                  (self.diameter/2.) as i16, self.color);
    }
}

struct Paddle {
    color: Color,
    x: f32,         // x pixel co-ordinate of top left corner
    y: f32,         // y pixel co-ordinate of top left corner
    width: f32,     
    height: f32,    
    speed: f32,     // pixels per second
    speed_multiplier: f32
}

impl Paddle {
    fn new(color: Color, x: f32, y: f32, width: f32, height: f32, speed: f32) -> Paddle {
        Paddle { 
            color: color,
            x: x, 
            y: y, 
            width: width, 
            height: height, 
            speed: speed,
            speed_multiplier: 1.0,
        }
    }
}

impl Drawable for Paddle {
    fn draw(&self, ui: &mut Ui) {
        ui.renderer.set_draw_color(self.color);
        let rect = Rect::new_unwrap(self.x as i32, self.y as i32, self.width as u32, 
                                    self.height as u32);
        ui.renderer.fill_rect(rect);
    }
}

struct GameLoopContext {
    dt_sec: f32,
    drawables: Vec<Rc<RefCell<Drawable>>>,
    audibles: Vec<Rc<Music>>
}

impl GameLoopContext {
    fn new(dt_sec: f32) -> GameLoopContext {
        GameLoopContext {
            dt_sec: dt_sec,
            drawables: Vec::new(),
            audibles: Vec::new()
        }
    }
}

struct Game {
    ui: Ui,
    fps: u32,
    table: Rc<RefCell<Table>>,
    ball: Rc<RefCell<Ball>>,
    lpaddle: Rc<RefCell<Paddle>>,
    rpaddle: Rc<RefCell<Paddle>>,
    time_ball_last_speedup_ms: Option<u64>,
    slow_motions_remaining: u32,
    time_slow_motion_started_ms: Option<u64>,
    running: bool
}

impl Game {

    /// Create initial game state. 
    fn new(ui: Ui, fps: u32, table: Table, ball: Ball, lpaddle: Paddle, 
           rpaddle: Paddle) -> Game { 
        Game { 
            ui: ui, 
            fps: fps, 
            table: Rc::new(RefCell::new(table)),
            ball: Rc::new(RefCell::new(ball)), 
            lpaddle: Rc::new(RefCell::new(lpaddle)), 
            rpaddle: Rc::new(RefCell::new(rpaddle)), 
            time_ball_last_speedup_ms: Option::None,
            slow_motions_remaining: 3,
            time_slow_motion_started_ms: Option::None,
            running: false 
        }
    }
    
    /// Display welcome screen
    fn show_welcome_screen(&mut self) {
        self.ui.poll_event();
        let color = Color::RGB(0xff, 0xff, 0xff);
        let surface = self.ui.font.render("Get Ready Player 1!", sdl2_ttf::blended(color)).unwrap();
        let texture = self.ui.renderer.create_texture_from_surface(&surface).unwrap();
        let target = Rect::new_unwrap(self.table.borrow().width  as i32 / 2 - 200, 20, 400, 50);
        self.ui.renderer.set_draw_color(Color::RGB(0x00, 0x00, 0x00));
        self.ui.renderer.clear();
        self.ui.renderer.set_draw_color(Color::RGB(0xff, 0xff, 0xff));
        self.ui.renderer.copy(&texture, None, Some(target));
        self.ui.renderer.present();
        thread::sleep_ms(2000);
    }

    /// Start the game and block until finished. 
    fn start(&mut self) {
        self.running = true;
        let mut time_last_invocation = clock_ticks::precise_time_ms();
        while self.running {
            let time_this_invocation = clock_ticks::precise_time_ms();
            let dt_ms = time_this_invocation - time_last_invocation;
            let mut ctx = GameLoopContext::new(dt_ms as f32/ 1000.);
            self.update(&mut ctx); 
            self.cap_fps(dt_ms);
            time_last_invocation = time_this_invocation;
        } 
    }
    
    // Called once per frame. 
    fn update(&mut self, ctx: &mut GameLoopContext) {
        ctx.drawables.push(self.table.clone());
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
                    // Move the left player paddle based on mouse input.
                    Event::MouseMotion{x,y, ..} => {
                        let y = y as f32;
                        let table = self.table.borrow();
                        let mut lpaddle = self.lpaddle.borrow_mut();
                        lpaddle.y = y; 
                        if lpaddle.y < 0. { 
                            lpaddle.y = 0.; 
                        } else if lpaddle.y + lpaddle.height > table.height {
                            lpaddle.y = table.height - lpaddle.height; 
                        }
                    }
                    _ => {}
                }
            },
            None => {}
        }
        ctx.drawables.push(self.lpaddle.clone());
    }

    // The game moves the right paddle. 
    fn move_right_paddle(&mut self, ctx: &mut GameLoopContext) {
        let table = self.table.borrow();
        let mut ball = self.ball.borrow();
        let mut rpaddle = self.rpaddle.borrow_mut(); 
        // Move toward oncoming ball. If the ball is moving away, head for the home position.
        let tracking_y = if ball.vx > 0. { ball.y + ball.diameter / 2. } else { table.height / 2. }; 
        // We use non-overlapping segments of the paddle (3/4 vs 1/4) when deciding whether to move
        // the paddle up or down. Using the center of the paddle against the center of the ball is
        // very precise and will result in overshoots. Then in the next frame the paddle jumps up
        // to compensate. Using different segments, we stabilize the movement.
        if tracking_y > rpaddle.y + rpaddle.height * (3./4.) {
            rpaddle.y += self.get_modified_speed(rpaddle.speed, rpaddle.speed_multiplier) * ctx.dt_sec;
            // Guard against overshooting the ball.
            if rpaddle.y > tracking_y {
                rpaddle.y = tracking_y - rpaddle.height / 2.;
            }
        } else if tracking_y < rpaddle.y + rpaddle.height * (1./4.) {
            rpaddle.y -= self.get_modified_speed(rpaddle.speed, rpaddle.speed_multiplier) * ctx.dt_sec;
            // Guard against overshooting the ball.
            if rpaddle.y + rpaddle.height < tracking_y {
                rpaddle.y = tracking_y - rpaddle.height / 2.;
            }
        }
        if rpaddle.y < 0. { 
            rpaddle.y = 0.; 
        } else if rpaddle.y + rpaddle.height > table.height {
            rpaddle.y = table.height - rpaddle.height; 
        }
        ctx.drawables.push(self.rpaddle.clone());
    }
        
    // Move the ball and deal with collisions. 
    fn move_ball(&mut self, ctx: &mut GameLoopContext) {
        let mut table = self.table.borrow_mut(); 
        let mut ball = self.ball.borrow_mut(); 
        let lpaddle = self.lpaddle.borrow();
        let mut rpaddle = self.rpaddle.borrow_mut();
        
        let mut new_ball_x = ball.x + self.get_modified_speed(ball.vx, ball.speed_multiplier) * ctx.dt_sec;
        let mut new_ball_y = ball.y + self.get_modified_speed(ball.vy, ball.speed_multiplier) * ctx.dt_sec;

        // Speedup the ball periodically until max speed reached. 
        let time_now_ms = clock_ticks::precise_time_ms();
        match self.time_ball_last_speedup_ms {
            None => {   
                self.time_ball_last_speedup_ms = Option::Some(time_now_ms);
            },
            Some(time_ball_last_speedup_ms) => {
                if time_now_ms - time_ball_last_speedup_ms > 15000 && 
                    ball.speed_multiplier < 2. && self.time_slow_motion_started_ms.is_none() {
                    ball.speed_multiplier += 0.1;
                    rpaddle.speed_multiplier += 0.1;
                    self.time_ball_last_speedup_ms = Option::Some(time_now_ms);
                }
            }
        }

        // Top or bottom wall.
        if new_ball_y < 0. {
            new_ball_y = -new_ball_y;
            ball.vy = -ball.vy;
            ctx.audibles.push(self.ui.ping_sound.clone());
        } else if new_ball_y + ball.diameter >= table.height { 
            new_ball_y = table.height - (new_ball_y + ball.diameter - table.height) - ball.diameter;
            ball.vy = -ball.vy;
            ctx.audibles.push(self.ui.ping_sound.clone());
        } 

        // Left or right paddle.
        if new_ball_x < lpaddle.x + lpaddle.width && ball.x >= lpaddle.x + lpaddle.width {
            let bounce_x = lpaddle.x + lpaddle.width; 
            // The gradient of the straight line from (ball.x,ball.y) to (bounce_x,bounce_y) to
            // (new_ball_x,new_ball_y) stays constant, so we can use that to find the value of the
            // top left corner of the ball when it bounces.  
            let bounce_y = (new_ball_y - ball.y) / (new_ball_x - ball.x) * (bounce_x - ball.x) + ball.y;
            if bounce_y + ball.diameter >= lpaddle.y && bounce_y <= lpaddle.y + lpaddle.height {
                // Calculate where the center of the ball hit relative to center of the paddle.
                let relative_y = (lpaddle.y + lpaddle.height / 2.) - (bounce_y + ball.diameter / 2.);
                // Use the ratio of the bounce position to half the height of the paddle as an
                // angle multiplier.
                let bounce_angle_multiplier = (relative_y / (lpaddle.height / 2.)).abs();
                let bounce_angle = bounce_angle_multiplier * ball.max_paddle_bounce_angle;
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
                ctx.audibles.push(self.ui.pong_sound.clone()); 
            }
        } else if new_ball_x + ball.diameter > rpaddle.x && ball.x + ball.diameter <= rpaddle.x {
            let bounce_x = rpaddle.x - ball.diameter; 
            let bounce_y = (new_ball_y - ball.y) / (new_ball_x - ball.x) * (bounce_x - ball.x) + ball.y;
            if bounce_y + ball.diameter  >= rpaddle.y && bounce_y <= rpaddle.y + rpaddle.height {
                let relative_y = (rpaddle.y + rpaddle.height / 2.) - (bounce_y + ball.diameter / 2.);
                let bounce_angle_multiplier = (relative_y / (rpaddle.height / 2.)).abs();
                let bounce_angle = bounce_angle_multiplier * ball.max_paddle_bounce_angle;
                ball.vx = ball.speed * bounce_angle.cos() * -1.;
                ball.vy = ball.speed * bounce_angle.sin() * if ball.vy < 0. {-1.} else {1.}; 
                let bounce_dt_sec = ctx.dt_sec * (new_ball_y - bounce_y) / (new_ball_y - ball.y);
                new_ball_x = bounce_x + ball.vx * bounce_dt_sec;
                new_ball_y = bounce_y + ball.vy * bounce_dt_sec;
                ctx.audibles.push(self.ui.ping_sound.clone()); 
            }
        } 

        // Left or right wall.
        if new_ball_x < 0. { 
            new_ball_x = -new_ball_x;
            ball.vx = -ball.vx;
            // Right player scored.
            table.rscore += 1;    
            ctx.audibles.push(self.ui.ping_sound.clone()); 
        } else if new_ball_x + ball.diameter > table.width { 
            new_ball_x = table.width - (new_ball_x + ball.diameter - table.width) - ball.diameter;
            ball.vx = -ball.vx;
            // Left player scored.
            table.lscore += 1;    
            ctx.audibles.push(self.ui.ping_sound.clone()); 
        } 

        ball.x = new_ball_x;
        ball.y = new_ball_y;
        ctx.drawables.push(self.ball.clone());
    }
    
    fn draw(&mut self, ctx: &mut GameLoopContext) {
        for d in ctx.drawables.iter() {
            d.borrow().draw(&mut self.ui);
        }
        self.ui.renderer.present();
    }

    fn play_audio(&mut self, ctx: &mut GameLoopContext) {
        for a in ctx.audibles.iter() {
            a.play(1);
        }
    }

    fn get_modified_speed(&self, speed: f32, speed_multiplier: f32) -> f32 {
        match self.time_slow_motion_started_ms {
            None => {
                speed * speed_multiplier
            },
            Some(time_slow_motion_started_ms) => {
                speed * speed_multiplier * 0.5
            }
        }
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
    table_color: Color,
    table_width: f32,
    table_height: f32,
    net_color: Color,
    fps: u32,
    ball_color: Color,
    ball_speed: f32,
    ball_diameter: f32,
    lpaddle_color: Color,
    rpaddle_color: Color,
    paddle_offset: f32,
    paddle_width: f32,
    paddle_height: f32,
    paddle_speed: f32,
    max_launch_angle: f32,
    max_bounce_angle: f32
}

impl GameBuilder {

    fn new() -> GameBuilder {
        GameBuilder {
            table_color: Color::RGB(0x00, 0x00, 0x00),
            table_width: 480.,
            table_height: 320.,
            net_color: Color::RGB(0xff, 0xff, 0xff),
            fps: 40,
            ball_color: Color::RGB(0xff, 0xff, 0xff),
            ball_speed: 320.,
            ball_diameter: 10.,
            lpaddle_color: Color::RGB(0xff, 0xff, 0xff),
            rpaddle_color: Color::RGB(0xff, 0xff, 0xff),
            paddle_offset: 4.,
            paddle_width: 10.,
            paddle_height: 80.,
            paddle_speed: 640.,
            max_launch_angle: f32::consts::PI/4.,
            max_bounce_angle: f32::consts::PI/12.
        }
    }

    fn with_table_dimensions(mut self, width: f32, height: f32) -> GameBuilder {
        self.table_width = width;
        self.table_height = height;
        self
    }

    fn with_table_color(mut self, r: u8, g: u8, b: u8) -> GameBuilder {
        self.table_color = Color::RGB(r,g,b);
        self
    }

    fn with_net_color(mut self, r: u8, g: u8, b: u8) -> GameBuilder {
        self.net_color = Color::RGB(r,g,b);
        self
    }
    
    fn with_fps(mut self, fps: u32) -> GameBuilder {
        self.fps = fps; 
        self
    }
    
    fn with_ball_color(mut self, r: u8, g: u8, b: u8) -> GameBuilder {
        self.ball_color = Color::RGB(r,g,b);
        self
    }

    fn with_ball_speed_per_sec(mut self, speed: f32) -> GameBuilder {
        self.ball_speed = speed; 
        self
    }

    fn with_ball_diameter(mut self, diameter: f32) -> GameBuilder {
        self.ball_diameter = diameter;
        self
    }

    fn with_paddle_offset(mut self, offset: f32) -> GameBuilder {
        self.paddle_offset = offset;
        self
    }

    fn with_paddle_width(mut self, width: f32) -> GameBuilder {
        self.paddle_width = width;
        self
    }
    
    fn with_paddle_height(mut self, height: f32) -> GameBuilder {
        self.paddle_height = height;
        self
    }

    fn with_paddle_speed_per_sec(mut self, speed: f32) -> GameBuilder {
        self.paddle_speed = speed; 
        self
    }

    fn with_left_paddle_color(mut self, r: u8, g: u8, b: u8) -> GameBuilder {
        self.lpaddle_color = Color::RGB(r,g,b);
        self
    }

    fn with_right_paddle_color(mut self, r: u8, g: u8, b: u8) -> GameBuilder {
        self.rpaddle_color = Color::RGB(r,g,b);
        self
    }

    fn with_max_launch_angle_rads(mut self, max_launch_angle: f32) -> GameBuilder {
        self.max_launch_angle = max_launch_angle;
        self
    }
    
    fn with_max_bounce_angle_rads(mut self, max_bounce_angle: f32) -> GameBuilder {
        self.max_bounce_angle = max_bounce_angle;
        self
    }

    fn create_ui(&self) -> Ui {
        let sdl_ctx = sdl2::init().unwrap();
        sdl_ctx.mouse().show_cursor(false);
        //let cursor = sdl2::mouse::Cursor::from_system(sdl2::mouse::SystemCursor::No).unwrap().set(); 
        let video_subsystem = sdl_ctx.video().unwrap();
        let window = video_subsystem.window("pong", 
                self.table_width as u32, self.table_height as u32)
                .position_centered()
                .build()
                .unwrap();
        let renderer = window.renderer().build().unwrap();
        let ttf_ctx = sdl2_ttf::init().unwrap();
        let font_path = Path::new("assets/fonts/absender.ttf");
        let font = sdl2_ttf::Font::from_file(font_path, 128).unwrap();
        let sdl_audio = sdl_ctx.audio().unwrap();
        sdl2_mixer::open_audio(DEFAULT_FREQUENCY, sdl2_mixer::AUDIO_S16LSB, 2, 1024);
        let ping_sound_path = Path::new("assets/sounds/ping.wav");
        let ping_sound = sdl2_mixer::Music::from_file(ping_sound_path).unwrap();
        let pong_sound_path = Path::new("assets/sounds/pong.wav");
        let pong_sound = sdl2_mixer::Music::from_file(pong_sound_path).unwrap();
        Ui::new(sdl_ctx, renderer, ttf_ctx, font, sdl_audio, ping_sound, pong_sound)
    }

    fn create_table(&self) -> Table {
        Table::new(self.table_color, self.table_width, self.table_height, self.net_color,
                   self.lpaddle_color, self.rpaddle_color)
    }

    fn create_ball(&self) -> Ball {
        
        // Place ball at center of screen. 
        let diameter = self.ball_diameter;
        let x = self.table_width/2.;
        let y = self.table_height/2.;

        let speed = self.ball_speed;
        let mut rng = rand::thread_rng();

        let launch_angle = Range::new(0., self.max_launch_angle).ind_sample(&mut rng);
        let dir = [-1., 1.];

        // Use the sine of the angle to determine the vertical speed. Then, 
        // choose a direction (up or down) to select a vertical velocity.
        let up_or_down = rand::sample(&mut rng, dir.into_iter(),1)[0]; 
        let vy = launch_angle.sin() * speed * up_or_down; 
        let left_or_right = rand::sample(&mut rng, dir.into_iter(),1)[0]; 
        
        // Use Pythagoras to determine the horizontal speed. Then, choose a
        // direction (left or right) to select a horizontal velocity.
        let vx = ((speed * speed) - (vy * vy)).sqrt() * left_or_right;
        Ball::new(self.ball_color, x, y, diameter, speed, vx, vy, self.max_bounce_angle)
    }    

    fn create_left_paddle(&self) -> Paddle {
        let width = self.paddle_width;
        let height = self.paddle_height;
        let x = self.paddle_offset;
        let y = (self.table_height - height)/2.;
        let speed = self.paddle_speed;
        Paddle::new(self.lpaddle_color, x, y, width, height, speed)
    }

    fn create_right_paddle(&self) -> Paddle {
        let width = self.paddle_width;
        let height = self.paddle_height;
        let x = self.table_width - (self.paddle_offset + width);
        let y = (self.table_height - height)/2.;
        let speed = self.paddle_speed;
        Paddle::new(self.rpaddle_color, x, y, width, height, speed)
    }

    fn build(&self) -> Game {
        Game::new(self.create_ui(), 
                  self.fps, 
                  self.create_table(),
                  self.create_ball(),
                  self.create_left_paddle(),
                  self.create_right_paddle())
    }
}

fn main() {
    let mut game = GameBuilder::new()
        .with_table_dimensions(800., 600.)
        .with_table_color(0x25, 0x25, 0x25)
        .with_net_color(0xf4, 0xf3, 0xee)
        .with_ball_color(0xff, 0xcc, 0x00)
        .with_ball_speed_per_sec(500.)
        .with_ball_diameter(11.)
        .with_paddle_offset(4.)
        .with_paddle_width(5.)
        .with_paddle_height(60.)
        .with_paddle_speed_per_sec(300.)
        .with_left_paddle_color(0xf6, 0xf4, 0xda)
        .with_right_paddle_color(0xd9, 0xe2, 0xe1)
        .with_max_launch_angle_rads(f32::consts::PI*50./180.)
        .with_max_bounce_angle_rads(f32::consts::PI*45./180.)
        .with_fps(40)
        .build();
    game.show_welcome_screen();
    game.start();
}
