/// loveletter: implementation of [Love Letter](http://boardgamegeek.com/boardgame/129622/love-letter)

pub use action::{Play, PlayError};
pub use deck::Card;
pub use game::{Game, GameError, Turn};

pub mod deck;
pub mod prompt;

mod action;
mod game;
mod player;
mod util;


#[cfg(test)]
mod tests;
