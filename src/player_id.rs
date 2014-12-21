/// Simple opaque type used to identify players.

use std::fmt;
use std::slice;
use std::vec;

#[deriving(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct PlayerId(uint);


impl fmt::Show for PlayerId {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let &PlayerId(i) = self;
        formatter.write(format!("Player #{}", i + 1).as_bytes())
    }
}


pub struct PlayerIdGenerator {
    last: uint,
}

impl PlayerIdGenerator {
    fn new() -> PlayerIdGenerator {
        PlayerIdGenerator { last: 0 }
    }
}

impl Iterator<PlayerId> for PlayerIdGenerator {
    fn next(&mut self) -> Option<PlayerId> {
        let result = PlayerId(self.last);
        self.last += 1;
        Some(result)
    }
}


pub fn player_id_generator() -> PlayerIdGenerator {
    PlayerIdGenerator::new()
}


pub fn make_players(n: uint) -> Option<Players> {
    let player_id_gen = player_id_generator();
    let players: Vec<PlayerId> = player_id_gen.take(n).collect();
    match Players::new(players.as_slice()) {
        Ok(p) => Some(p),
        Err(Error::InvalidNumPlayers(..)) => None,
        _ => panic!("player_id_generator generated duplicate players"),
    }
}


pub enum Error {
    InvalidNumPlayers(uint),
    DuplicatePlayers,
}

pub struct Players {
    _players: Vec<PlayerId>,
}


impl Players {
    pub fn new(players: &[PlayerId]) -> Result<Players, Error> {
        if !valid_player_count(players.len()) {
            return Err(Error::InvalidNumPlayers(players.len()));
        }
        let mut ps = vec::as_vec(players).clone();
        ps.sort();
        let mut qs = ps.clone();
        qs.dedup();
        if ps == qs {
            Ok(Players { _players: ps })
        } else {
            Err(Error::DuplicatePlayers)
        }
    }

    pub fn iter(&self) -> slice::Items<PlayerId> {
        self._players.iter()
    }
}


fn valid_player_count(num_players: uint) -> bool {
    2 <= num_players && num_players <= 4
}
