use sdl2::{AudioSubsystem, Sdl};
use sdl2::event::Event;
use sdl2::render::Renderer;
use sdl2_mixer::Music; 
use sdl2_ttf::Sdl2TtfContext; 
use std::rc::Rc;

/// Interface for interacting with the user. For example, obtaining user input, drawing to the
/// screen and playing audio.
pub struct Ui {
    pub sdl_ctx: Sdl,
    pub renderer: Renderer<'static>,
    pub ttf_ctx: Sdl2TtfContext,
    pub sdl_audio: AudioSubsystem, 
    pub ping_sound: Rc<Music>,
    pub pong_sound: Rc<Music>
}

impl Ui {
    
    pub fn new(sdl_ctx: Sdl, 
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
    pub fn poll_event(&self) -> Option<Event> {
        return self.sdl_ctx.event_pump().unwrap().poll_event();
    }

}

/// Trait for types that can be drawn to the screen. 
pub trait Drawable {
    fn draw(&self, ui: &mut Ui); 
}

