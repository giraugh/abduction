-- Create test match
INSERT INTO match_config(match_id, player_count) VALUES (
    1,
    100
);

-- Create test entities

INSERT INTO entity_mutation(entity_id, match_id, mutation_type, payload) VALUES (
    'a',
    1,
    'S',
    '{ "name": "Empty Entity", "markers": [], "attributes": {}, "relations": [] }'
);

INSERT INTO entity_mutation(entity_id, match_id, mutation_type, payload) VALUES (
    'b',
    1,
    'S',
    '{ "name": "Full Entity", "markers": ["player"], "attributes": { "health": [3, 3] }, "relations": [["friend", "a"]] }'
);

INSERT INTO entity_mutation(entity_id, match_id, mutation_type, payload) VALUES (
    'c',
    1,
    'S',
    '{ "name": "Gonna be deleted Entity", "markers": [], "attributes": {}, "relations": [] }'
);

INSERT INTO entity_mutation(entity_id, match_id, mutation_type) VALUES (
    'c',
    1,
    'D'
);
