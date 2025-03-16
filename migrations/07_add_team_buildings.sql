-- Migration to add buildings for teams
-- filepath: /home/ixguard/Documents/work/rust/discord-bot/migrations/07_add_team_buildings.sql

-- Create the buildings table
CREATE TABLE team_buildings (
    id SERIAL PRIMARY KEY,
    team_id INTEGER NOT NULL,
    building_name VARCHAR(100) NOT NULL,
    level INTEGER NOT NULL,
    last_upgraded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure unique combinations of team and building
    -- (each team can have only one of each building type)
    UNIQUE (team_id, building_name)
);

-- Add indices for faster lookups
CREATE INDEX idx_team_buildings_team_id ON team_buildings(team_id);

-- Add a view to join team information with their buildings
CREATE VIEW team_building_info AS
SELECT 
    tb.id AS building_id,
    t.id AS team_id,
    t.name AS team_name,
    tb.building_name,
    tb.level,
    tb.last_upgraded_at,
    (tb.level = tb.max_level) AS is_maxed
FROM teams t
JOIN team_buildings tb ON t.id = tb.team_id
ORDER BY t.name, tb.building_name;

-- Add some seed data for common building types
INSERT INTO team_buildings (team_id, building_name, level) VALUES
    -- Team 1 Buildings
    (0, 'Townhall', 1);

-- Create a trigger to update last_upgraded_at on changes
CREATE TRIGGER update_team_buildings_upgrade_timestamp
BEFORE UPDATE ON team_buildings
FOR EACH ROW
WHEN NEW.level <> OLD.level
BEGIN
    UPDATE team_buildings 
    SET last_upgraded_at = CURRENT_TIMESTAMP
    WHERE id = OLD.id;
END;
