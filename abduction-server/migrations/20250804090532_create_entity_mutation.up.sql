CREATE TABLE entity_mutation (
    -- Unique mutation id, not referenced
    mutation_id INTEGER PRIMARY KEY NOT NULL,

    -- A uuid field identifying the entity
    entity_id TEXT NOT NULL,

    -- An identifying which match its for
    match_id INTEGER NOT NULL,

    -- Mutation type
    -- Either (S)et or (D)elete
    mutation_type TEXT CHECK( mutation_type IN ('S','D') ) NOT NULL,

    -- payload for mutation
    -- (Required for S mutations but not for D)
    payload JSONB,

    -- Created at
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- TODO FOREIGN KEY ON MATCH CONFIG
