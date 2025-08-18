# Abduction Dev TODO

- Need to think about whether I am really happy storing complete entities inside diffs...
- Ideally, we have a nicer setup for mutating entities when implementing gamelogic, its a bit rough right now.. even w/ the .mutate() setup
  - Do we want a setup thats stores mutations like `SetAttr(name, field)` etc? If the db can merge JSON fields this might be doable..
- We need a nicer way to store motivators so that they can have unique behaviour... perhaps
  - What we want is to be able to define what "actions" a motivator of a given "kind" will "vote" on.
  - i.e I need a way to say that the `HealthMotivator` is distinct and votes for *these* actions.
  - but I guess it could just be a big function which takes `EntityAttributes` and outputs the action votes? feels a bit gross...
  - Ig what I really want is to be able to have a HashMap<MotivatorKind, Motivator>...? Doesn't seem too bad...
