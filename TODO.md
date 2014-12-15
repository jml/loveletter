# TODO

## Enable external client

## Improve command-line client

### End of game

* Report on _all_ survivors at the end
* Announce _why_ game ended
* Have a different message for 'last player standing' vs 'highest remaining player'

## Code health

* Write tests for everything in `util`
* Create a state validator for `Game`
* Provide a way of getting the burnt card at the end
* Try using the State pattern for Game
* Improve speed of `subtract_vector`
* Improve speed of `maxima_by` to be one pass
* `GameState` is a bit weird: maybe try to have something better
* Split `handle_turn` into two states: waiting for player & next player

# Questions

* Is it possible to represent `play_to_action` as a data structure, rather
  than a function?
