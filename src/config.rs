/// Configuration for a game of Love Letter.
///
/// Love Letter is a very simple game, so there's very little configuration:
/// just the number of players and the score you need to win.


#[deriving(Clone)]
pub struct Config {
    _num_players: uint,
}


pub enum Error {
    /// Specified an invalid number of players.
    InvalidPlayers(uint),
}


impl Config {
    pub fn new(num_players: uint) -> Result<Config, Error> {
        if valid_player_count(num_players) {
            Ok(Config { _num_players: num_players })
        } else {
            Err(Error::InvalidPlayers(num_players))
        }
    }

    pub fn num_players(&self) -> uint {
        self._num_players
    }
}


fn valid_player_count(num_players: uint) -> bool {
    2 <= num_players && num_players <= 4
}
