use deck;

#[deriving(Show, PartialEq, Eq, Clone)]
pub struct Player {
    _hand: Option<deck::Card>,
    _protected: bool,
    _discard: Vec<deck::Card>,
}


#[deriving(Show, PartialEq, Eq)]
pub enum Error {
    Inactive,
    BadGuess,
    NoSuchCard(deck::Card, (deck::Card, deck::Card)),
}


impl Player {
    pub fn new(hand: Option<deck::Card>) -> Player {
        Player { _hand: hand, _protected: false, _discard: vec![] }
    }

    pub fn active(&self) -> bool {
        self._hand.is_some()
    }

    pub fn discards(&self) -> &[deck::Card] {
        self._discard.as_slice()
    }

    pub fn get_hand(&self) -> Option<deck::Card> {
        self._hand
    }

    pub fn protect(&self, protected: bool) -> Result<Player, Error> {
        if self.active() {
            Ok(Player {
                _hand: self._hand,
                _protected: protected,
                _discard: self._discard.clone(),
            })
        } else {
            Err(Inactive)
        }
    }

    pub fn eliminate(&self) -> Result<Player, Error> {
        match self._hand {
            None => Err(Inactive),
            Some(hand) =>
                Ok(if self._protected {
                    self.clone()
                } else {
                    self.replace(None).discard(hand)
                }),
        }
    }

    pub fn eliminate_if_guessed(&self, guess: deck::Card) -> Result<Player, Error> {
        if guess == deck::Soldier {
            return Err(BadGuess)
        }
        match self._hand {
            None => Err(Inactive),
            Some(card) =>
                if card == guess {
                    self.eliminate()
                } else {
                    Ok(self.clone())
                },
        }
    }

    pub fn eliminate_if_weaker(&self, other: &Player) -> Result<(Player, Player), Error> {
        match (self._hand, other._hand) {
            (Some(my_card), Some(their_card)) => {
                if self._protected {
                    Ok((self.clone(), other.clone()))
                } else {
                    Ok(match my_card.cmp(&their_card) {
                        Less => (self.replace(None), other.clone()),
                        Greater => (self.clone(), other.replace(None)),
                        Equal => (self.clone(), other.clone())
                    })
                }
            }
            _ => Err(Inactive),
        }
    }

    pub fn swap_hands(&self, other: &Player) -> Result<(Player, Player), Error> {
        if !self.active() {
            Err(Inactive)
        } else if self._protected {
            Ok((self.clone(), other.clone()))
        } else {
            Ok((self.replace(other._hand), other.replace(self._hand)))
        }
    }

    pub fn play_card(&self, dealt: deck::Card, chosen: deck::Card) -> Result<Player, Error> {
        match self._hand {
            None => Err(Inactive),
            Some(hand) =>
                if chosen == hand {
                    Ok(self.discard(chosen).replace(Some(dealt)))
                } else if chosen == dealt {
                    Ok(self.discard(chosen))
                } else {
                    Err(NoSuchCard(chosen, (hand, dealt)))
                },
        }
    }

    pub fn discard_and_draw(&self, new_card: Option<deck::Card>) -> Result<Player, Error> {
        if !self.active() {
            Err(Inactive)
        } else if self._protected {
            Ok(self.clone())
        } else {
            match self._hand {
                // XXX: Also need to make sure that when we eliminate in this
                // case, that we return the new_card to the stack.
                Some(deck::Princess) => self.eliminate(),
                Some(hand) => Ok(self.discard(hand).replace(new_card)),
                _ => panic!("{} not be active", self)
            }
        }
    }

    fn discard(&self, card: deck::Card) -> Player {
        let mut new_discard = self._discard.clone();
        new_discard.push(card);
        Player {
            _hand: self._hand,
            _protected: self._protected,
            _discard: new_discard,
        }
    }

    fn replace(&self, card: Option<deck::Card>) -> Player {
        Player {
            _hand: card,
            _protected: self._protected,
            _discard: self._discard.clone(),
        }
    }
}


#[cfg(test)]
mod test {
    use deck;
    use super::{Inactive, Player};

    #[test]
    fn test_eliminate_gone_player() {
        let p = Player::new(None);
        let error = p.eliminate().unwrap_err();
        assert_eq!(Inactive, error);
    }

    #[test]
    fn test_discards_empty() {
        let p = Player::new(Some(deck::Wizard));
        let expected: &[deck::Card] = [];
        assert_eq!(expected, p.discards());
    }

    #[test]
    fn test_play_hand_updates_hand() {
        let p = Player::new(Some(deck::Wizard));
        let new_p = p.play_card(deck::Priestess, deck::Wizard).unwrap();
        assert_eq!(Some(deck::Priestess), new_p.get_hand());
    }

    #[test]
    fn test_play_hand_discards() {
        let p = Player::new(Some(deck::Wizard));
        let new_p = p.play_card(deck::Priestess, deck::Wizard).unwrap();
        let expected: &[deck::Card] = [deck::Wizard];
        assert_eq!(expected, new_p.discards());
    }

    #[test]
    fn test_play_dealt_leaves_hand() {
        let p = Player::new(Some(deck::Wizard));
        let new_p = p.play_card(deck::Priestess, deck::Priestess).unwrap();
        assert_eq!(Some(deck::Wizard), new_p.get_hand());
    }

    #[test]
    fn test_play_dealt_discards() {
        let p = Player::new(Some(deck::Wizard));
        let new_p = p.play_card(deck::Priestess, deck::Priestess).unwrap();
        let expected: &[deck::Card] = [deck::Priestess];
        assert_eq!(expected, new_p.discards());
    }

    #[test]
    fn test_force_discard_updates_discard() {
        let p = Player::new(Some(deck::Knight));
        let new_p = p.discard_and_draw(Some(deck::Clown)).unwrap();
        let expected: &[deck::Card] = [deck::Knight];
        assert_eq!(expected, new_p.discards());
    }

    #[test]
    fn test_force_discard_princess_updates_discard() {
        let p = Player::new(Some(deck::Princess));
        let new_p = p.discard_and_draw(Some(deck::Clown)).unwrap();
        let expected: &[deck::Card] = [deck::Princess];
        assert_eq!(expected, new_p.discards());
    }

    #[test]
    fn test_eliminate_discards() {
        let p = Player::new(Some(deck::Princess));
        let new_p = p.eliminate().unwrap();
        let expected: &[deck::Card] = [deck::Princess];
        assert_eq!(expected, new_p.discards());
    }

}
