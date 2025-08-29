CREATE TABLE match_config (
    -- A unique id identifying the match
    match_id TEXT NOT NULL PRIMARY KEY,

    -- The total number of players in this match
    player_count INTEGER NOT NULL,

    -- Optionally, the id of the match which precedes this
    -- if set, the surviving players from that match will
    -- be cloned into this match
    preceding_match_id TEXT,

    --- How many hexagons in each direction the world extents to
    --- Sort of like a hexagon radius
    world_radius INTEGER NOT NULL,

    --- Was this match completed? If not, it will be resumed on startup
    complete BOOLEAN NOT NULL DEFAULT FALSE,

    -- When the configuration was created
    -- For now, this is used to identity the most current match
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
