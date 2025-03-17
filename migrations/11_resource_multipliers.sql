-- Create a table to map building levels to resource multipliers and bonuses
CREATE TABLE resource_multiplier_mapping (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    building_name TEXT NOT NULL,
    resource_category TEXT NOT NULL,
    building_level INTEGER NOT NULL,
    multiplier REAL NOT NULL,
    flat_bonus INTEGER NOT NULL DEFAULT 0,  -- Flat bonus amount to add after multiplication
    UNIQUE(building_name, resource_category, building_level)
);

-- Insert base multipliers for different resource buildings
-- Woodcutter's Lodge multipliers (wood category)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('woodcutter_lodge', 'logs', 0, 1.0, 0),    -- Base rate with no building
('woodcutter_lodge', 'logs', 1, 1.2, 1),    -- Level 1: 20% bonus + 1 extra
('woodcutter_lodge', 'logs', 2, 1.4, 2),    -- Level 2: 40% bonus + 2 extra
('woodcutter_lodge', 'logs', 3, 1.6, 3),    -- Level 3: 60% bonus + 3 extra
('woodcutter_lodge', 'logs', 4, 1.8, 5),    -- Level 4: 80% bonus + 5 extra
('woodcutter_lodge', 'logs', 5, 2.0, 8),    -- Level 5: 100% bonus + 8 extra
('woodcutter_lodge', 'logs', 6, 2.2, 12),   -- Level 6: 120% bonus + 12 extra
('woodcutter_lodge', 'logs', 7, 2.4, 18),   -- Level 7: 140% bonus + 18 extra
('woodcutter_lodge', 'logs', 8, 2.6, 25),   -- Level 8: 160% bonus + 25 extra
('woodcutter_lodge', 'logs', 9, 3.0, 35);   -- Level 9: 200% bonus + 35 extra

-- Prospector's Shop multipliers (mining category)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('prospector_shop', 'ores', 0, 1.0, 0),   -- Base rate with no building
('prospector_shop', 'ores', 1, 1.2, 1),   -- Level 1: 20% bonus + 1 extra ore
('prospector_shop', 'ores', 2, 1.4, 2),   -- Level 2: 40% bonus + 2 extra ore
('prospector_shop', 'ores', 3, 1.6, 3),   -- Level 3: 60% bonus + 3 extra ore
('prospector_shop', 'ores', 4, 1.8, 4),   -- Level 4: 80% bonus + 4 extra ore
('prospector_shop', 'ores', 5, 2.0, 6),   -- Level 5: 100% bonus + 6 extra ore
('prospector_shop', 'ores', 6, 2.2, 9),   -- Level 6: 120% bonus + 9 extra ore
('prospector_shop', 'ores', 7, 2.4, 13),  -- Level 7: 140% bonus + 13 extra ore
('prospector_shop', 'ores', 8, 2.6, 18),  -- Level 8: 160% bonus + 18 extra ore
('prospector_shop', 'ores', 9, 3.0, 25);  -- Level 9: 200% bonus + 25 extra ore

-- Fishmonger's Shop multipliers (fishing category)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('fishmonger_shop', 'fish', 0, 1.0, 0),  -- Base rate with no building
('fishmonger_shop', 'fish', 1, 1.2, 1),  -- Level 1: 20% bonus + 1 extra fish
('fishmonger_shop', 'fish', 2, 1.4, 2),  -- Level 2: 40% bonus + 2 extra fish
('fishmonger_shop', 'fish', 3, 1.6, 3),  -- Level 3: 60% bonus + 3 extra fish
('fishmonger_shop', 'fish', 4, 1.8, 4),  -- Level 4: 80% bonus + 4 extra fish
('fishmonger_shop', 'fish', 5, 2.0, 6),  -- Level 5: 100% bonus + 6 extra fish
('fishmonger_shop', 'fish', 6, 2.2, 9),  -- Level 6: 120% bonus + 9 extra fish
('fishmonger_shop', 'fish', 7, 2.4, 12), -- Level 7: 140% bonus + 12 extra fish
('fishmonger_shop', 'fish', 8, 2.6, 16), -- Level 8: 160% bonus + 16 extra fish
('fishmonger_shop', 'fish', 9, 3.0, 22); -- Level 9: 200% bonus + 22 extra fish

-- Herbalist's Hut multipliers (herb category)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('herbalist_hut', 'herbs', 0, 1.0, 0),      -- Base rate with no building
('herbalist_hut', 'herbs', 1, 1.2, 0),      -- Level 1: 20% bonus (herbs are rarer, so no flat bonus at low levels)
('herbalist_hut', 'herbs', 2, 1.4, 1),      -- Level 2: 40% bonus + 1 extra herb
('herbalist_hut', 'herbs', 3, 1.6, 1),      -- Level 3: 60% bonus + 1 extra herb
('herbalist_hut', 'herbs', 4, 1.8, 2),      -- Level 4: 80% bonus + 2 extra herbs
('herbalist_hut', 'herbs', 5, 2.0, 3),      -- Level 5: 100% bonus + 3 extra herbs
('herbalist_hut', 'herbs', 6, 2.2, 4),      -- Level 6: 120% bonus + 4 extra herbs
('herbalist_hut', 'herbs', 7, 2.4, 6),      -- Level 7: 140% bonus + 6 extra herbs
('herbalist_hut', 'herbs', 8, 2.6, 8),      -- Level 8: 160% bonus + 8 extra herbs
('herbalist_hut', 'herbs', 9, 3.0, 12);     -- Level 9: 200% bonus + 12 extra herbs

-- Farming Guild multipliers (farming category)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('farming_guild', 'farming', 0, 1.0, 0),   -- Base rate with no building
('farming_guild', 'farming', 1, 1.2, 1),   -- Level 1: 20% bonus + 1 extra seed
('farming_guild', 'farming', 2, 1.4, 1),   -- Level 2: 40% bonus + 1 extra seed
('farming_guild', 'farming', 3, 1.6, 2),   -- Level 3: 60% bonus + 2 extra seeds
('farming_guild', 'farming', 4, 1.8, 3),   -- Level 4: 80% bonus + 3 extra seeds
('farming_guild', 'farming', 5, 2.0, 4),   -- Level 5: 100% bonus + 4 extra seeds
('farming_guild', 'farming', 6, 2.2, 6),   -- Level 6: 120% bonus + 6 extra seeds
('farming_guild', 'farming', 7, 2.4, 8),   -- Level 7: 140% bonus + 8 extra seeds
('farming_guild', 'farming', 8, 2.6, 11),  -- Level 8: 160% bonus + 11 extra seeds
('farming_guild', 'farming', 9, 3.0, 15);  -- Level 9: 200% bonus + 15 extra seeds

-- Rune Shop multipliers (rune category)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('rune_shop', 'runes', 0, 1.0, 0),          -- Base rate with no building
('rune_shop', 'runes', 1, 1.1, 1),          -- Level 1: 10% bonus + 1 extra rune
('rune_shop', 'runes', 2, 1.2, 2),          -- Level 2: 20% bonus + 2 extra runes
('rune_shop', 'runes', 3, 1.3, 3),          -- Level 3: 30% bonus + 3 extra runes
('rune_shop', 'runes', 4, 1.4, 4),          -- Level 4: 40% bonus + 4 extra runes
('rune_shop', 'runes', 5, 1.5, 5),          -- Level 5: 50% bonus + 5 extra runes
('rune_shop', 'runes', 6, 1.6, 7),          -- Level 6: 60% bonus + 7 extra runes
('rune_shop', 'runes', 7, 1.8, 9),          -- Level 7: 80% bonus + 9 extra runes
('rune_shop', 'runes', 8, 2.0, 12),         -- Level 8: 100% bonus + 12 extra runes
('rune_shop', 'runes', 9, 2.5, 15);         -- Level 9: 150% bonus + 15 extra runes

-- Treasury multipliers for currency
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('treasury', 'coins', 0, 1.0, 0),       -- Base rate with no building
('treasury', 'coins', 1, 1.1, 5),       -- Level 1: 10% bonus + 5 extra coins
('treasury', 'coins', 2, 1.2, 10),      -- Level 2: 20% bonus + 10 extra coins
('treasury', 'coins', 3, 1.3, 20),      -- Level 3: 30% bonus + 20 extra coins
('treasury', 'coins', 4, 1.4, 35),      -- Level 4: 40% bonus + 35 extra coins
('treasury', 'coins', 5, 1.5, 50),      -- Level 5: 50% bonus + 50 extra coins
('treasury', 'coins', 6, 1.6, 75),      -- Level 6: 60% bonus + 75 extra coins
('treasury', 'coins', 7, 1.7, 100),     -- Level 7: 70% bonus + 100 extra coins
('treasury', 'coins', 8, 1.8, 150),     -- Level 8: 80% bonus + 150 extra coins
('treasury', 'coins', 9, 2.0, 250);     -- Level 9: 100% bonus + 250 extra coins

INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('crafting_guild', 'gems', 0, 1.0, 0),    -- Base rate with no building
('crafting_guild', 'gems', 1, 2.0, 1),   -- Level 1: 15% bonus + 1 extra gem
('crafting_guild', 'gems', 2, 5.0, 1),    -- Level 2: 30% bonus + 1 extra gem
('crafting_guild', 'gems', 3, 10.0, 2),   -- Level 3: 45% bonus + 2 extra gems
('crafting_guild', 'gems', 4, 20.0, 2),    -- Level 4: 60% bonus + 2 extra gems
('crafting_guild', 'gems', 5, 40.0, 3),    -- Level 5: 80% bonus + 3 extra gems
('crafting_guild', 'gems', 6, 80.0, 4),    -- Level 6: 100% bonus + 4 extra gems
('crafting_guild', 'gems', 7, 160.0, 5),   -- Level 7: 125% bonus + 5 extra gems
('crafting_guild', 'gems', 8, 320.0, 7),    -- Level 8: 150% bonus + 7 extra gems
('crafting_guild', 'gems', 9, 640.0, 10);   -- Level 9: 200% bonus + 10 extra gems

-- Create a view to make it easier to query team resource multipliers
CREATE VIEW team_resource_multipliers AS
SELECT 
    t.id as team_id,
    t.name as team_name,
    tb.building_name,
    rm.resource_category,
    rm.multiplier,
    rm.flat_bonus
FROM 
    teams t
JOIN 
    team_buildings tb ON t.id = tb.team_id
JOIN 
    resource_multiplier_mapping rm ON tb.building_name = rm.building_name AND tb.level = rm.building_level;