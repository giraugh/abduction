-- Create test match
INSERT INTO match_config(match_id, player_count) VALUES (
    'test',
    100
);

-- Create test entities

INSERT INTO entity_mutation(entity_id, match_id, mutation_type, payload) VALUES (
    'a',
    'test',
    'S',
    '{ "name": "Empty Entity", "markers": [], "attributes": {}, "relations": [] }'
);

INSERT INTO entity_mutation(entity_id, match_id, mutation_type, payload) VALUES (
    'b',
    'test',
    'S',
    '{ "name": "Full Entity", "markers": ["player"], "attributes": { "health": [3, 3] }, "relations": [["friend", "a"]] }'
);

INSERT INTO entity_mutation(entity_id, match_id, mutation_type, payload) VALUES (
    'c',
    'test',
    'S',
    '{ "name": "Gonna be deleted Entity", "markers": [], "attributes": {}, "relations": [] }'
);

INSERT INTO entity_mutation(entity_id, match_id, mutation_type) VALUES (
    'c',
    'test',
    'D'
);
