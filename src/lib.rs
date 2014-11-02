/// loveletter: implementation of [Love Letter](http://boardgamegeek.com/boardgame/129622/love-letter)

pub use action::{Play, PlayError, NoEffect, Attack, Guess, InvalidPlayer, InactivePlayer, BadGuess};
pub use deck::{Card, Soldier, Clown, Knight, Priestess, Wizard, General, Minister, Princess};
pub use game::{BadDeck, Game, InvalidPlayers, Turn};

pub mod deck;
pub mod prompt;

mod action;
mod game;
mod player;
mod util;


#[cfg(test)]
mod tests;
