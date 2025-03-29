-- Add sample data to teams table if it doesn't already exist
INSERT INTO teams (id, name) 
SELECT 0, 'sample_team'
WHERE NOT EXISTS (SELECT 1 FROM teams WHERE name = 'sample_team');

-- -- Get the ID of the sample team
-- INSERT INTO team_members (id, team_id, username)
-- SELECT 0, 0, 'sampleuser123'
-- WHERE NOT EXISTS (SELECT 1 FROM team_members WHERE username = 'sampleuser123');