-- migration for adding team members table
CREATE TABLE team_members (
    id SERIAL PRIMARY KEY,
    team_id INTEGER NOT NULL,
    user_id VARCHAR(255) NOT NULL
);

-- Add index for faster lookups by team
CREATE INDEX idx_team_members_team_id ON team_members(team_id);