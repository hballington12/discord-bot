-- Add sample data to teams table if it doesn't already exist
INSERT INTO teams (id, name) 
SELECT 0, 'Sample Team'
WHERE NOT EXISTS (SELECT 1 FROM teams WHERE name = 'Sample Team');

-- Get the ID of the sample team
INSERT INTO team_members (id, team_id, username)
SELECT 0, 0, 'sampleuser123'
WHERE NOT EXISTS (SELECT 1 FROM team_members WHERE username = 'sampleuser123');