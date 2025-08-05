SELECT
    match_id,
    player_count,
    preceding_match_id,
    created_at
FROM
    match_config
WHERE
    match_id = ?
