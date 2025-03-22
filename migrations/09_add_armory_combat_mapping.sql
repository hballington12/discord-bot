-- filepath: /home/ixguard/Documents/work/rust/discord-bot/migrations/09_add_armory_combat_mapping.sql
-- Create a table to map armory levels to combat level access
CREATE TABLE armory_combat_mapping (
    armory_level INTEGER PRIMARY KEY,
    max_combat_level INTEGER NOT NULL
);

-- Populate with sample data
INSERT INTO armory_combat_mapping (armory_level, max_combat_level) VALUES
(1, 10),
(2, 100),
(3, 200),
(4, 300),
(5, 400),
(6, 500),
(7, 600),
(8, 700),
(9, 9999);