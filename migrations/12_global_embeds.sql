-- Migration to create global_embeds table for tracking global embeds in Discord channels

-- Create the global_embeds table
CREATE TABLE global_embeds (
    id SERIAL PRIMARY KEY,
    channel_id BIGINT NOT NULL,  -- Discord channel IDs are large numbers
    variant VARCHAR(50) NOT NULL, -- Type of embed (e.g., "townhall_ranking", "global_stats", "leaderboard")
    message_id BIGINT,           -- Optional: To track the actual message ID for updates
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure unique combinations of channel and variant
    -- (one type of global embed per channel)
    UNIQUE (channel_id, variant)
);

-- Add indices for faster lookups
CREATE INDEX idx_global_embeds_channel_id ON global_embeds(channel_id);
CREATE INDEX idx_global_embeds_variant ON global_embeds(variant);

-- Add a view to get global embed information
CREATE VIEW global_embed_info AS
SELECT 
    ge.id AS embed_id,
    ge.channel_id,
    ge.variant,
    ge.message_id,
    ge.last_updated_at
FROM global_embeds ge
ORDER BY ge.variant;

-- Optional: Add a trigger to update last_updated_at on changes
CREATE TRIGGER update_global_embeds_timestamp
BEFORE UPDATE ON global_embeds
FOR EACH ROW
BEGIN
    UPDATE global_embeds 
    SET last_updated_at = CURRENT_TIMESTAMP
    WHERE id = OLD.id;
END;