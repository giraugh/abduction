CREATE TABLE entity_mutation (
    -- Unique mutation id, not referenced
    mutation_id INTEGER PRIMARY KEY NOT NULL,

    -- A uuid field identifying the entity
    entity_id TEXT NOT NULL,

    -- A uuid field identifying which match its in
    match_id TEXT NOT NULL,

    -- Mutation type
    -- Either (S)et or (D)elete
    mutation_type TEXT CHECK( mutation_type IN ('S','D') ) NOT NULL,

    -- payload for mutation
    -- (Required for C and U mutations but not for D)
    payload JSONB,

    -- Created at (TODO: should this be more precise?)
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
