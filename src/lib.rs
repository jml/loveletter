/// loveletter: implementation of [Love Letter](http://boardgamegeek.com/boardgame/129622/love-letter)

pub use action::{Event, Play, PlayError, PlayerId};
pub use config::Error as Error;
pub use deck::Card;
pub use round::{Round, Turn, TurnOutcome};
pub use round::Error as RoundError;

pub mod deck;
pub mod game;
pub mod prompt;

mod action;
mod config;
mod round;
mod player;
mod util;


#[cfg(test)]
mod tests;
