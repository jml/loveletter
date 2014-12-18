/// loveletter: implementation of [Love Letter](http://boardgamegeek.com/boardgame/129622/love-letter)

pub use action::{Event, Play, PlayError};
pub use deck::Card;
pub use round::{Round, Turn, TurnOutcome};
pub use round::Error as RoundError;

pub mod deck;
pub mod prompt;

mod action;
mod config;
mod game;
mod round;
mod player;
mod util;


#[cfg(test)]
mod tests;
