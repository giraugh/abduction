SELECT
    match_id,
    player_count as "player_count: i32",
    preceding_match_id,
    world_radius as "world_radius: i32"
FROM
    match_config
WHERE
    match_id = ?
