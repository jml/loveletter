/// A whole game of Love Letter.
///
/// A game of Love Letter consists of multiple rounds. The winners of each
/// round receive a token of affection from the princess. The first players to
/// receive four tokens of affection are declared to have won her heart, and
/// thus, the game.

use config;
use round;

pub struct Game {
    // XXX: Possibly Game should not own Config. In the only current non-test
    // use case, Config can easily last longer than Game. The only reason we
    // want to own this is for the helper `make_game` function.
    _config: config::Config,
}


impl Game {
    fn new(config: config::Config) -> Game {
        Game { _config: config }
    }

    fn num_players(&self) -> uint {
        self._config.num_players()
    }

    pub fn new_round(&self) -> round::Round {
        round::Round::new(&self._config)
    }
}


pub fn new_game(num_players: uint) -> Result<Game, config::Error> {
    config::Config::new(num_players).map(|cfg| Game::new(cfg))
}



#[cfg(test)]
mod test {

    use config;
    use super::Game;

    fn make_game(num_players: uint) -> Game {
        let cfg = config::Config::new(num_players).ok().unwrap();
        Game::new(cfg)
    }

    #[test]
    fn test_num_players() {
        let num_players = 4;
        let cfg = config::Config::new(num_players).ok().unwrap();
        let g = Game::new(cfg);
        assert_eq!(g.num_players(), num_players);
    }

    #[test]
    fn test_make_round() {
        let g = make_game(4);
        let r = g.new_round();
        assert_eq!(r.num_players(), g.num_players());
    }

}
