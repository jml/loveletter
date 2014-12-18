# TODO

## Enable external client

## Improve command-line client

### End of game

* Report on _all_ survivors at the end
* Announce _why_ game ended
* Have a different message for 'last player standing' vs 'highest remaining player'

## Code health

* Write tests for everything in `util`
* Create a state validator for `Round`
* Provide a way of getting the burnt card at the end
* Try using the State pattern for `Round`
* Improve speed of `subtract_vector`
* Improve speed of `maxima_by` to be one pass
* `State` is a bit weird: maybe try to have something better
* Split `handle_turn` into two states: waiting for player & next player

# Questions

* Is it possible to represent `play_to_action` as a data structure, rather
  than a function?
* Currently there's a bunch of punning around player id being the index of the
  vector in both `Game` and `Round`, with both objects using the same ids. It
  would be nice to make this explicit.
* What should the remote interface look like?
* `Game` is currently constructed from a `Config` object but maybe it should
  instead take a list of player IDs that are then used opaquely throughout the
  program?
* How do I cleanly separate the command-line program from the core code?
* What's the best way cleanly separate the (anticipated!) server?
