extern crate sdl2;

use sdl2::keycode::KeyCode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

struct Ball {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    dx: i32,
    dy: i32,
}

struct Paddle {
    w: i32,
    h: i32,
    x: i32,
    y: i32,
}

fn main() {
    let arena_w: i32 = 800;
    let arena_h: i32 = 600;
    
    let mut sdl_context = sdl2::init().video().unwrap();

    let window = sdl_context.window("pong", arena_w as u32, arena_h as u32)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer().build().unwrap();

    let mut drawer = renderer.drawer();
    drawer.set_draw_color(Color::RGB(0,0,0));
    drawer.clear();
    drawer.present();

    let mut ball = Ball{x: arena_w/2, 
                        y: arena_h/2,
                        w: 20,
                        h: 20,
                        dx: 1,
                        dy: 0,
                   };

    let mut running = true;

    while running {
        // Game loop should look like:
        // check for user input
        // run AI
        // move enemies
        // resolve collisions
        // draw graphics
        // play sounds
        
        for event in sdl_context.event_pump().poll_iter() {
            use sdl2::event::Event;

            match event {
                Event::Quit {..} | 
                Event::KeyDown { keycode: KeyCode::Escape, .. } => {
                    running = false
                },
                _ => {}
            }
        }
        ball.x += ball.dx;
        ball.x %= arena_w;
        ball.y += ball.dy;
        ball.y %= arena_h;
        drawer.set_draw_color(Color::RGB(0, 0, 0));
        drawer.clear();
        // The rest of the game loop goes
        // here...
        let ball_rect = Rect::new(ball.x, ball.y, ball.w, ball.h);
        drawer.set_draw_color(Color::RGB(255, 255, 255));
        drawer.fill_rect(ball_rect);
        drawer.present();
    }
}

//TODO:
//Collision detection as per http://lazyfoo.net/tutorials/SDL/27_collision_detection/index.php
