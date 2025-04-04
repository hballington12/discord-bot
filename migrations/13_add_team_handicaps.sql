-- Migration to add handicap column to teams table
ALTER TABLE teams ADD COLUMN handicap INTEGER NOT NULL DEFAULT 1;

-- Update existing teams to have handicap 1
UPDATE teams SET handicap = 1 WHERE handicap IS NULL;

-- Add index on handicap for faster queries if needed
CREATE INDEX idx_teams_handicap ON teams (handicap);

PRAGMA user_version = 2;