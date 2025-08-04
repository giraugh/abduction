WITH match_mutations AS (
    SELECT * from entity_mutation WHERE match_id = ?
),
latest_mutations AS (
    SELECT
        entity_id,
        payload,
        mutation_type,
        ROW_NUMBER() OVER (PARTITION BY entity_id ORDER BY mutation_id DESC) AS row_num
    FROM
        match_mutations
)
SELECT
    entity_id,
    payload as "entity: Json<Entity>"
FROM
    latest_mutations
WHERE
    row_num = 1
    AND mutation_type = 'S';
