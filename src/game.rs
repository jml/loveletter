/// A whole game of Love Letter.
///
/// A game of Love Letter consists of multiple rounds. The winners of each
/// round receive a token of affection from the princess. The first players to
/// receive four tokens of affection are declared to have won her heart, and
/// thus, the game.

use config;
use round;

struct Game {
    _num_players: uint,
}


impl Game {
    fn new(config: &config::Config) -> Game {
        Game { _num_players: config.num_players() }
    }

    fn num_players(&self) -> uint {
        self._num_players
    }

    fn new_round(&self) -> round::Round {
        round::Round::new(self.num_players()).unwrap()
    }
}


#[cfg(test)]
mod test {

    use config;
    use super::Game;

    fn make_game(num_players: uint) -> Game {
        let cfg = config::Config::new(num_players).ok().unwrap();
        Game::new(&cfg)
    }

    #[test]
    fn test_num_players() {
        let num_players = 4;
        let cfg = config::Config::new(num_players).ok().unwrap();
        let g = Game::new(&cfg);
        assert_eq!(g.num_players(), num_players);
    }

    #[test]
    fn test_make_round() {
        let g = make_game(4);
        let r = g.new_round();
        assert_eq!(r.num_players(), g.num_players());
    }

}
