# Abduction

> Aliens are abducting humans and running sick games. Humans are fighting to their death in space in brutal heart-renching games... want to see one?

Abduction is a text based story simulator for survival-games-esque matches featuring a revolving cast of simulated agent players.
Follow your favourite players as they battle to escape the alien system.

## How will this work?

Abduction is still work in progress. When the first season launches you can expect:

- Procedural text-based stories for each player
- A rotating cast of humans that are changed by every match
- Shocking unexpected world events, weather and twists
- A human story that focuses on making friends and persevering just as much as violence and depravity
- Tools for browsing and inspecting each player, so you can make an informed decision on who gets your favour
- Site-wide voting to provide advantages (at a cost) to certain players

The schedule is still a bit wip, but here's what I'm thinking so far
- Each season will run for about a month
- Pool of about 100 players, with ~15 in each match
- Games starting every few hours, about 2-3 a day


## Work In Progress

Here's my ideation / notes on current game progress.

-----

#### Technical (Engine Improvements)

Websocket Updates
- Resuming websocket when dead
- Backport this to qubit

Migration Safety
- When we make certain changes, I want to update some kind of version tag and then force a new game to be deployed when the CI build runs
- this could quite literally use the rust version and put something in the db

Entity ID Interning
- Checkout `rodeo` crate for interning entity IDs
- Reduce memory allocations
- Alternatively, I could create my own mechanism for this by just using actual integers for ids
  - Perhaps with a generation mechanism?
  - TODO: check out what bevy does for this


#### Game Design

Narrative Framing
 - Introduction by a host
 - Slowly spawn in each player along with a narrative announcement

Water Storage
 - Could potentially have a "bottle" item
 - Can fill it with water if held
  - Has two uses of water

New Motivators Ideas
 - **Disease**
  - Slowly converts into sickness
  - Can spread to others
 - **Loneliness**
  - Never see other people, then this happens
  - Causes talking to non-people

Agent Persistence
 - Want a way for the same "players" to appear in multiple games
 - Audience can get attached to their favourite characters
 - Death should still be a way of being eliminated
   - And it should be possible to be completely eliminated
 - IDEA:
   - There is some "entity" that wonders the map and collects corpses
   - Collected corpses are revived into the next game
   - Players with corpses that are not collected are lost forever
     - If they are disintegrated, eaten, exploded etc
     - If something happens to the collector?

Fun Misc Feature Ideas
- Forming Allies
- Owning Pets
- Collecting and Holding Resources
- Players greeting each other
- Memories
- Fire spreading

Game linking design
  - Do we have one huge week-long game? or roughly one game a day?
  - I think im leaning towards one game a day, with the expectation that they'll take most of the day
    - Gives more opportunities to see the start of the game, and down-time between games
