# Abduction

Plan is that the server simulates a simple mostly text-based survival game and streams it to the site. The sites show whats happening
on a hex grid w/ event logs from each hex.

## TODO

Websocket Death
- Resuming websocket when dead
- Backport this to qubit

Environment Generation


Scheduling games
  - Detect when a game is finished (one player remains)
  - Stop that game (emit MatchEnd TickEvent), and somehow schedule the next one
    - When a game finishes, setup an async task to wake up next day at a given time and start the next match
    - TODO: add persistence to setup this task when starting the program

Non-game part of client 
  - Must be able to check if a match is currently running
  - If not, show when the next game is scheduled for
  - See the outcome of the previous game
  - (Later) see the replay (and outcome) of the previous game

Game linking design
  - Do we have one huge week-long game? or roughly one game a day?
  - I think im leaning towards one game a day, with the expectation that they'll take most of the day
    - Gives more opportunities to see the start of the game, and down-time between games
