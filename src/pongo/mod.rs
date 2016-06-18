/// Trait for types that can be set back to an initial state. 
pub trait Resettable {
    fn reset(&mut self);
}

pub mod ball;
pub mod game;
pub mod net;
pub mod paddle;
pub mod score_card;
pub mod ui;

