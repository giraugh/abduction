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
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Link to match config
    FOREIGN KEY (match_id) REFERENCES match_config(match_id)
);

-- CREATE AN INDEX FOR THE MUTATIONS IN A GIVEN MATCH
CREATE INDEX entity_mutation_match ON entity_mutation(match_id);
