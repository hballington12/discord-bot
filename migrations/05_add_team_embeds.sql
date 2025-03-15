-- Migration to create team_embeds table for tracking team embeds in Discord channels

-- Create the team_embeds table
CREATE TABLE team_embeds (
    id SERIAL PRIMARY KEY,
    team_id INTEGER NOT NULL,
    channel_id BIGINT NOT NULL,  -- Discord channel IDs are large numbers
    variant VARCHAR(50) NOT NULL, -- Type of embed (e.g., "resources", "stats", "dashboard")
    message_id BIGINT,           -- Optional: To track the actual message ID for updates
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key to the teams table
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    
    -- Ensure unique combinations of team, channel and variant
    -- (one type of embed per team per channel)
    UNIQUE (team_id, channel_id, variant)
);

-- Add indices for faster lookups
CREATE INDEX idx_team_embeds_team_id ON team_embeds(team_id);
CREATE INDEX idx_team_embeds_channel_id ON team_embeds(channel_id);

-- Add a view to join team information with their embeds
CREATE VIEW team_embed_info AS
SELECT 
    te.id AS embed_id,
    t.id AS team_id,
    t.name AS team_name,
    te.channel_id,
    te.variant,
    te.message_id,
    te.last_updated_at
FROM teams t
JOIN team_embeds te ON t.id = te.team_id
ORDER BY t.name, te.variant;

-- Optional: Add a trigger to update last_updated_at on changes
CREATE TRIGGER update_team_embeds_timestamp
BEFORE UPDATE ON team_embeds
FOR EACH ROW
BEGIN
    UPDATE team_embeds 
    SET last_updated_at = CURRENT_TIMESTAMP
    WHERE id = OLD.id;
END;