INSERT INTO match_config(
    match_id,
    player_count,
    preceding_match_id,
    created_at
)
VALUES (?, ?, ?, ?)
ON CONFLICT ("match_id")
DO UPDATE
SET
    player_count       = EXCLUDED.player_count,
    preceding_match_id = EXCLUDED.preceding_match_id,
    created_at         = EXCLUDED.created_at;
