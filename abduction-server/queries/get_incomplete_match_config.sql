WITH latest_match as (
    SELECT * FROM match_config
    ORDER BY created_at DESC
    LIMIT 1
)
SELECT match_id,
    player_count as "player_count: i32",
    preceding_match_id,
    world_radius as "world_radius: i32",
    complete
FROM latest_match WHERE complete = false
