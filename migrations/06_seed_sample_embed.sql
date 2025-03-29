-- Seed data for team_embeds table

-- -- Add sample embeds for team 1 (assuming it exists from previous migrations)
-- INSERT INTO team_embeds (team_id, channel_id, variant, message_id) 
-- VALUES 
--     (0, 0000000000000000000, 'resources', NULL);

-- Verify inserts with a query
-- SELECT t.name AS team_name, te.variant, te.channel_id, te.message_id
-- FROM teams t
-- JOIN team_embeds te ON t.id = te.team_id
-- ORDER BY t.id, te.variant;