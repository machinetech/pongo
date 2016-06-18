mod pongo;

extern crate rand;
extern crate sdl2;
extern crate sdl2_gfx;
extern crate sdl2_image;
extern crate sdl2_mixer;
extern crate sdl2_ttf;

use sdl2::pixels::Color;

use sdl2_image::{INIT_PNG};
use sdl2_mixer::{AUDIO_S16LSB, DEFAULT_FREQUENCY, Music}; 
use sdl2_ttf::{Font}; 

use std::f32;
use std::path::Path;
use std::rc::Rc;

use pongo::ball::Ball;
use pongo::game::Game;
use pongo::net::Net;
use pongo::paddle::Paddle;
use pongo::score_card::ScoreCard;
use pongo::ui::Ui;

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
    sdl_ctx.mouse().show_cursor(false);

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

