/// Simple opaque type used to identify players.

use std::fmt;

#[deriving(PartialEq, Eq, Clone)]
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
