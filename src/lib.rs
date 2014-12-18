/// loveletter: implementation of [Love Letter](http://boardgamegeek.com/boardgame/129622/love-letter)

pub use action::{Event, Play, PlayError};
pub use deck::Card;
pub use round::{Round, GameError, Turn, TurnOutcome};

pub mod deck;
pub mod prompt;

mod action;
mod round;
mod player;
mod util;


#[cfg(test)]
mod tests;
