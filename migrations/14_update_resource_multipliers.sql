-- Migration script to standardize resource multipliers across all building types
-- Using the Woodcutter's Lodge values as the base template

-- First, delete existing multipliers for buildings we want to update
DELETE FROM resource_multiplier_mapping WHERE building_name IN (
    'prospector_shop', 
    'fishmonger_shop', 
    'herbalist_hut', 
    'farming_guild', 
    'rune_shop', 
    'treasury', 
    'crafting_guild'
);

-- Re-insert with standardized values based on Woodcutter's Lodge
-- Prospector's Shop (ores)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('prospector_shop', 'ores', 1, 1.0, 0),    -- Base rate
('prospector_shop', 'ores', 2, 1.2, 2),    -- Level 2: 20% bonus + 2 extra
('prospector_shop', 'ores', 3, 1.5, 4),    -- Level 3: 50% bonus + 4 extra
('prospector_shop', 'ores', 4, 1.8, 7),    -- Level 4: 80% bonus + 7 extra
('prospector_shop', 'ores', 5, 2.2, 12),   -- Level 5: 120% bonus + 12 extra
('prospector_shop', 'ores', 6, 2.7, 18),   -- Level 6: 170% bonus + 18 extra
('prospector_shop', 'ores', 7, 3.3, 25),   -- Level 7: 230% bonus + 25 extra
('prospector_shop', 'ores', 8, 4.0, 35),   -- Level 8: 300% bonus + 35 extra
('prospector_shop', 'ores', 9, 5.0, 50);   -- Level 9: 400% bonus + 50 extra

-- Fishmonger's Shop (fish)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('fishmonger_shop', 'fish', 1, 1.0, 0),    -- Base rate
('fishmonger_shop', 'fish', 2, 1.2, 2),    -- Level 2: 20% bonus + 2 extra
('fishmonger_shop', 'fish', 3, 1.5, 4),    -- Level 3: 50% bonus + 4 extra
('fishmonger_shop', 'fish', 4, 1.8, 7),    -- Level 4: 80% bonus + 7 extra
('fishmonger_shop', 'fish', 5, 2.2, 12),   -- Level 5: 120% bonus + 12 extra
('fishmonger_shop', 'fish', 6, 2.7, 18),   -- Level 6: 170% bonus + 18 extra
('fishmonger_shop', 'fish', 7, 3.3, 25),   -- Level 7: 230% bonus + 25 extra
('fishmonger_shop', 'fish', 8, 4.0, 35),   -- Level 8: 300% bonus + 35 extra
('fishmonger_shop', 'fish', 9, 5.0, 50);   -- Level 9: 400% bonus + 50 extra

-- Herbalist's Hut (herbs)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('herbalist_hut', 'herbs', 1, 1.0, 0),     -- Base rate
('herbalist_hut', 'herbs', 2, 1.2, 2),     -- Level 2: 20% bonus + 2 extra
('herbalist_hut', 'herbs', 3, 1.5, 4),     -- Level 3: 50% bonus + 4 extra
('herbalist_hut', 'herbs', 4, 1.8, 7),     -- Level 4: 80% bonus + 7 extra
('herbalist_hut', 'herbs', 5, 2.2, 12),    -- Level 5: 120% bonus + 12 extra
('herbalist_hut', 'herbs', 6, 2.7, 18),    -- Level 6: 170% bonus + 18 extra
('herbalist_hut', 'herbs', 7, 3.3, 25),    -- Level 7: 230% bonus + 25 extra
('herbalist_hut', 'herbs', 8, 4.0, 35),    -- Level 8: 300% bonus + 35 extra
('herbalist_hut', 'herbs', 9, 5.0, 50);    -- Level 9: 400% bonus + 50 extra

-- Farming Guild (seeds)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('farming_guild', 'seeds', 1, 1.0, 0),     -- Base rate
('farming_guild', 'seeds', 2, 1.2, 2),     -- Level 2: 20% bonus + 2 extra
('farming_guild', 'seeds', 3, 1.5, 4),     -- Level 3: 50% bonus + 4 extra
('farming_guild', 'seeds', 4, 1.8, 7),     -- Level 4: 80% bonus + 7 extra
('farming_guild', 'seeds', 5, 2.2, 12),    -- Level 5: 120% bonus + 12 extra
('farming_guild', 'seeds', 6, 2.7, 18),    -- Level 6: 170% bonus + 18 extra
('farming_guild', 'seeds', 7, 3.3, 25),    -- Level 7: 230% bonus + 25 extra
('farming_guild', 'seeds', 8, 4.0, 35),    -- Level 8: 300% bonus + 35 extra
('farming_guild', 'seeds', 9, 5.0, 50);    -- Level 9: 400% bonus + 50 extra

-- Rune Shop (runes)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('rune_shop', 'runes', 1, 1.0, 0),         -- Base rate
('rune_shop', 'runes', 2, 1.2, 2),         -- Level 2: 20% bonus + 2 extra
('rune_shop', 'runes', 3, 1.5, 4),         -- Level 3: 50% bonus + 4 extra
('rune_shop', 'runes', 4, 1.8, 7),         -- Level 4: 80% bonus + 7 extra
('rune_shop', 'runes', 5, 2.2, 12),        -- Level 5: 120% bonus + 12 extra
('rune_shop', 'runes', 6, 2.7, 18),        -- Level 6: 170% bonus + 18 extra
('rune_shop', 'runes', 7, 3.3, 25),        -- Level 7: 230% bonus + 25 extra
('rune_shop', 'runes', 8, 4.0, 35),        -- Level 8: 300% bonus + 35 extra
('rune_shop', 'runes', 9, 5.0, 50);        -- Level 9: 400% bonus + 50 extra

-- Treasury (coins)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('treasury', 'coins', 1, 1.0, 0),          -- Base rate
('treasury', 'coins', 2, 1.2, 2),          -- Level 2: 20% bonus + 2 extra
('treasury', 'coins', 3, 1.5, 4),          -- Level 3: 50% bonus + 4 extra
('treasury', 'coins', 4, 1.8, 7),          -- Level 4: 80% bonus + 7 extra
('treasury', 'coins', 5, 2.2, 12),         -- Level 5: 120% bonus + 12 extra
('treasury', 'coins', 6, 2.7, 18),         -- Level 6: 170% bonus + 18 extra
('treasury', 'coins', 7, 3.3, 25),         -- Level 7: 230% bonus + 25 extra
('treasury', 'coins', 8, 4.0, 35),         -- Level 8: 300% bonus + 35 extra
('treasury', 'coins', 9, 5.0, 50);         -- Level 9: 400% bonus + 50 extra

-- Crafting Guild (gems) - keep higher values here since gems are more rare
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('crafting_guild', 'gems', 1, 1.0, 0),     -- Base rate
('crafting_guild', 'gems', 2, 1.2, 2),     -- Level 2: 20% bonus + 2 extra
('crafting_guild', 'gems', 3, 1.5, 4),     -- Level 3: 50% bonus + 4 extra
('crafting_guild', 'gems', 4, 1.8, 7),     -- Level 4: 80% bonus + 7 extra
('crafting_guild', 'gems', 5, 2.2, 12),    -- Level 5: 120% bonus + 12 extra
('crafting_guild', 'gems', 6, 2.7, 18),    -- Level 6: 170% bonus + 18 extra
('crafting_guild', 'gems', 7, 3.3, 25),    -- Level 7: 230% bonus + 25 extra
('crafting_guild', 'gems', 8, 4.0, 35),    -- Level 8: 300% bonus + 35 extra
('crafting_guild', 'gems', 9, 5.0, 50);    -- Level 9: 400% bonus + 50 extra

-- Update the database version
PRAGMA user_version = 12;