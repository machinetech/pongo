extern crate clock_ticks;
extern crate rand;
extern crate sdl2;
extern crate sdl2_gfx;
extern crate sdl2_ttf;

use rand::distributions::{IndependentSample, Range};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;
use sdl2::Sdl;
use sdl2::TimerSubsystem;
use sdl2::VideoSubsystem;
use std::default::Default;
use std::f32;
use std::path::Path;
use std::thread;

use sdl2_gfx::primitives::DrawRenderer;

use sdl2_ttf::Sdl2TtfContext;

struct Ui {
    sdl_ctx: Sdl,
    renderer: Renderer<'static>,
    ttf_ctx: Sdl2TtfContext
}

impl Ui {
    fn new(sdl_ctx: Sdl, renderer: Renderer<'static>, ttf_ctx: Sdl2TtfContext) -> Ui {
        Ui { 
            sdl_ctx: sdl_ctx, 
            renderer: renderer,
            ttf_ctx: ttf_ctx
        }  
    } 

    fn poll_event(&self) -> Option<Event> {
        let mut event_pump = self.sdl_ctx.event_pump().unwrap();
        return event_pump.poll_event();
    }
}

struct Table {
    color: Color,
    width: f32,
    height: f32
}

impl Table {
    fn new(color: Color, width: f32, height: f32) -> Table {
        Table {
            color: color,
            width: width,
            height: height
        }
    }

    fn draw(&self, ui: &mut Ui) {

        // Fill the table background.
        ui.renderer.set_draw_color(self.color);
        ui.renderer.clear();
        
        // Draw the net.
        let num_net_dots = 20;
        let num_net_gaps = num_net_dots - 1;
        let net_dot_width = 10.;
        let net_dot_height = self.height / (num_net_dots + num_net_gaps) as f32;
        for i in 0..num_net_dots + num_net_gaps + 1 {
            let net_dot_x = self.width / 2. - net_dot_width / 2.;
            let net_dot_y = i as f32 * net_dot_height; 
            // todo: Need separate net color in table.
            ui.renderer.set_draw_color(if i % 2 == 0 {Color::RGB(0xff, 0xff, 0xff)} else {self.color});
            let net_dot_rect = Rect::new_unwrap(net_dot_x as i32, 
                                        net_dot_y as i32, 
                                        net_dot_width as u32,
                                        net_dot_height as u32);
            ui.renderer.fill_rect(net_dot_rect);
        }

    }
}

struct Ball {
    color: Color,
    x: f32,                         // x pixel co-ordinate of top left corner
    y: f32,                         // y pixel co-ordinate of top left corner
    diameter: f32,    
    speed: f32,                     // pixels per second 
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
            vx: vx, 
            vy: vy,
            max_paddle_bounce_angle: max_paddle_bounce_angle
        }
    }
    
    fn draw(&self, ui: &mut Ui) {
        ui.renderer.filled_circle((self.x + self.diameter/2.) as i16, 
                                  (self.y + self.diameter/2.) as i16, 
                                  (self.diameter/2.) as i16, 
                                  self.color);
    }
}

struct Paddle {
    color: Color,
    x: f32,         // x pixel co-ordinate of top left corner
    y: f32,         // y pixel co-ordinate of top left corner
    width: f32,     
    height: f32,    
    speed: f32,     // pixels per second
    vy: f32,        // pixels per second
    score: u32
}

impl Paddle {
    fn new(color: Color, x: f32, y: f32, width: f32, height: f32, speed: f32, vy: f32, 
               score: u32) -> Paddle {
        Paddle { 
            color: color,
            x: x, 
            y: y, 
            width: width, 
            height: height, 
            speed: speed, 
            vy: vy, 
            score: score 
        }
    }

    fn draw(&self, ui: &mut Ui) {
        ui.renderer.set_draw_color(self.color);
        let rect = Rect::new_unwrap(self.x as i32, 
                                    self.y as i32, 
                                    self.width as u32,
                                    self.height as u32);
        ui.renderer.fill_rect(rect);
    }
}

struct Game {
    ui: Ui,
    fps: u32,
    table: Table,
    ball: Ball,
    lpaddle: Paddle,
    rpaddle: Paddle,
    running: bool
}

impl Game {

    /// Create initial game state. 
    fn new(ui: Ui, fps: u32, table: Table, ball: Ball, lpaddle: Paddle, 
               rpaddle: Paddle) -> Game { 
        Game { 
            ui: ui, 
            fps: fps, 
            table: table,
            ball: ball, 
            lpaddle: lpaddle, 
            rpaddle: rpaddle, 
            running: false 
        }
    }

    /// Start the game and block until finished. 
    fn start(&mut self) {
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

    // Called once per frame. 
    fn update(&mut self, dt_sec: f32) {
        self.move_left_paddle(dt_sec);
        self.move_right_paddle(dt_sec);
        self.move_ball(dt_sec);
        self.draw()
    }

    // Move the left paddle based on user input. 
    fn move_left_paddle(&mut self, dt_sec: f32) {
        match self.ui.poll_event() {
            Some(event) => {
                match event {
                    Event::Quit{..} => {
                        self.running = false;
                    },
                    Event::MouseMotion{x,y, ..} => {
                        let y = y as f32;
                        let table = &self.table;
                        let lpaddle = &mut self.lpaddle;
                        lpaddle.y = y; 
                        if lpaddle.y < 0. { 
                            lpaddle.y = 0.; 
                        }
                        else if lpaddle.y + lpaddle.height > table.height {
                            lpaddle.y = table.height - lpaddle.height; 
                        }
                    }
                    _ => {}
                }
            },
            None => {}
        }
    }

    // The game moves the right paddle. 
    fn move_right_paddle(&mut self, dt_sec: f32) {
        let table = &self.table;
        let ball = &self.ball;
        let rpaddle = &mut self.rpaddle;
        let tracking_y = if ball.vx > 0. { ball.y + ball.diameter / 2. } else { table.height / 2. }; 
        if tracking_y > rpaddle.y + rpaddle.height / 2. {
            rpaddle.y += rpaddle.speed * dt_sec;
        } else if tracking_y < rpaddle.y + rpaddle.height / 2. {
            rpaddle.y -= rpaddle.speed * dt_sec;
        }
        if rpaddle.y < 0. { 
            rpaddle.y = 0.; 
        }
        else if rpaddle.y + rpaddle.height > table.height {
            rpaddle.y = table.height - rpaddle.height; 
        }
    }
        
    // Move the ball and deal with collisions. 
    fn move_ball(&mut self, dt_sec: f32) {
        let table = &mut self.table;
        let ball = &mut self.ball;
        let lpaddle = &mut self.lpaddle;
        let rpaddle = &mut self.rpaddle;
        
        let mut new_ball_x = ball.x + ball.vx * dt_sec;
        let mut new_ball_y = ball.y + ball.vy * dt_sec;

        // Top or bottom wall.
        if new_ball_y < 0. {
            new_ball_y = -new_ball_y;
            ball.vy = -ball.vy;
        } else if new_ball_y + ball.diameter >= table.height { 
            new_ball_y = table.height - (new_ball_y + ball.diameter - table.height) - ball.diameter;
            ball.vy = -ball.vy;
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
                let bounce_dt_sec = dt_sec * (new_ball_y - bounce_y) / (new_ball_y - ball.y);
                new_ball_x = bounce_x + ball.vx * bounce_dt_sec;
                new_ball_y = bounce_y + ball.vy * bounce_dt_sec;
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
                let bounce_dt_sec = dt_sec * (new_ball_y - bounce_y) / (new_ball_y - ball.y);
                new_ball_x = bounce_x + ball.vx * bounce_dt_sec;
                new_ball_y = bounce_y + ball.vy * bounce_dt_sec;
            }
        } 

        // Left or right wall.
        if new_ball_x < 0. { 
            new_ball_x = -new_ball_x;
            ball.vx = -ball.vx;
            rpaddle.score += 1;
        } else if new_ball_x + ball.diameter > table.width { 
            new_ball_x = table.width - (new_ball_x + ball.diameter - table.width) - ball.diameter;
            ball.vx = -ball.vx;
            lpaddle.score += 1;
        } 

        ball.x = new_ball_x;
        ball.y = new_ball_y;
    }
    
    fn draw(&mut self) {
        self.table.draw(&mut self.ui);
        self.ball.draw(&mut self.ui);
        self.lpaddle.draw(&mut self.ui);
        self.rpaddle.draw(&mut self.ui);
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
    table_color: Color,
    table_width: f32,
    table_height: f32,
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
            table_color: Color::RGB(0xff, 0xff, 0xff),
            table_width: 480.,
            table_height: 320.,
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
        let mut renderer = window.renderer().build().unwrap();
        let ttf_ctx = sdl2_ttf::init().unwrap();
        let font_path = Path::new("assets/fonts/arcade/ARCADE.TTF");
        let font = sdl2_ttf::Font::from_file(font_path, 128).unwrap();
        let surface = font.render("10", sdl2_ttf::blended(Color::RGB(0xff, 0xff, 0xff))).unwrap();
        let texture = renderer.create_texture_from_surface(&surface).unwrap();
        let target = Rect::new_unwrap(10,20,100,100);
        renderer.set_draw_color(Color::RGB(0x00, 0x00, 0x00));
        renderer.clear();
        renderer.set_draw_color(Color::RGB(0xff, 0xff, 0xff));
        renderer.copy(&texture, None, Some(target));
        renderer.present();
        std::thread::sleep_ms(5000);
        Ui::new(sdl_ctx, renderer, ttf_ctx)
    }

    fn create_table(&self) -> Table {
        Table::new(self.table_color, self.table_width, self.table_height)
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
        let vy = 0.;
        let score = 0;
        Paddle::new(self.lpaddle_color, x, y, width, height, speed, vy, score)
    }

    fn create_right_paddle(&self) -> Paddle {
        let width = self.paddle_width;
        let height = self.paddle_height;
        let x = self.table_width - (self.paddle_offset + width);
        let y = (self.table_height - height)/2.;
        let speed = self.paddle_speed;
        let vy = 0.;
        let score = 0;
        Paddle::new(self.rpaddle_color, x, y, width, height, speed, vy, score)
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
    // Moved the todos to Trello. 
    let mut game = GameBuilder::new()
        .with_table_dimensions(800., 600.)
        .with_table_color(0x00, 0x00, 0x00)
        .with_fps(40)
        .with_ball_color(0xff, 0xff, 0xff)
        .with_ball_speed_per_sec(500.)
        .with_ball_diameter(11.)
        .with_paddle_offset(4.)
        .with_paddle_width(5.)
        .with_paddle_height(60.)
        .with_paddle_speed_per_sec(300.)
        .with_left_paddle_color(0xff, 0xff, 0xff)
        .with_right_paddle_color(0xff, 0xff, 0xff)
        .with_max_launch_angle_rads(f32::consts::PI/4.)
        .with_max_bounce_angle_rads(f32::consts::PI/3.)
        .build();
    game.start();
}
