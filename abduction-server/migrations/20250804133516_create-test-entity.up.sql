INSERT INTO entity_mutation(entity_id, match_id, mutation_type, payload) VALUES (
    'a',
    'TESTING',
    'S',
    '{ "name": "Empty Entity", "markers": [], "attributes": {}, "relations": [] }'
);

INSERT INTO entity_mutation(entity_id, match_id, mutation_type, payload) VALUES (
    'b',
    'TESTING',
    'S',
    '{ "name": "Full Entity", "markers": ["player"], "attributes": { "health": [3, 3] }, "relations": [["friend", "a"]] }'
);

INSERT INTO entity_mutation(entity_id, match_id, mutation_type, payload) VALUES (
    'c',
    'TESTING',
    'S',
    '{ "name": "Gonna be deleted Entity", "markers": [], "attributes": {}, "relations": [] }'
);

INSERT INTO entity_mutation(entity_id, match_id, mutation_type) VALUES (
    'c',
    'TESTING',
    'D'
);
