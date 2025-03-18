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
('woodcutter_lodge', 'logs', 1, 1.0, 0),    -- Base rate
('woodcutter_lodge', 'logs', 2, 1.2, 2),    -- Level 2: 20% bonus + 2 extra
('woodcutter_lodge', 'logs', 3, 1.5, 4),    -- Level 3: 50% bonus + 4 extra
('woodcutter_lodge', 'logs', 4, 1.8, 7),    -- Level 4: 80% bonus + 7 extra
('woodcutter_lodge', 'logs', 5, 2.2, 12),   -- Level 5: 120% bonus + 12 extra
('woodcutter_lodge', 'logs', 6, 2.7, 18),   -- Level 6: 170% bonus + 18 extra
('woodcutter_lodge', 'logs', 7, 3.3, 25),   -- Level 7: 230% bonus + 25 extra
('woodcutter_lodge', 'logs', 8, 4.0, 35),   -- Level 8: 300% bonus + 35 extra
('woodcutter_lodge', 'logs', 9, 5.0, 50);   -- Level 9: 400% bonus + 50 extra

-- Prospector's Shop multipliers (mining category)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('prospector_shop', 'ores', 1, 1.0, 0),    -- Base rate
('prospector_shop', 'ores', 2, 1.3, 2),    -- Level 2: 30% bonus + 2 extra ore
('prospector_shop', 'ores', 3, 1.6, 4),    -- Level 3: 60% bonus + 4 extra ore
('prospector_shop', 'ores', 4, 2.0, 7),    -- Level 4: 100% bonus + 7 extra ore
('prospector_shop', 'ores', 5, 2.5, 10),   -- Level 5: 150% bonus + 10 extra ore
('prospector_shop', 'ores', 6, 3.0, 15),   -- Level 6: 200% bonus + 15 extra ore
('prospector_shop', 'ores', 7, 3.6, 22),   -- Level 7: 260% bonus + 22 extra ore
('prospector_shop', 'ores', 8, 4.3, 30),   -- Level 8: 330% bonus + 30 extra ore
('prospector_shop', 'ores', 9, 5.2, 40);   -- Level 9: 420% bonus + 40 extra ore

-- Fishmonger's Shop multipliers (fishing category)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('fishmonger_shop', 'fish', 1, 1.0, 0),    -- Base rate
('fishmonger_shop', 'fish', 2, 1.2, 2),    -- Level 2: 20% bonus + 2 extra fish
('fishmonger_shop', 'fish', 3, 1.5, 4),    -- Level 3: 50% bonus + 4 extra fish
('fishmonger_shop', 'fish', 4, 1.9, 7),    -- Level 4: 90% bonus + 7 extra fish
('fishmonger_shop', 'fish', 5, 2.3, 11),   -- Level 5: 130% bonus + 11 extra fish
('fishmonger_shop', 'fish', 6, 2.8, 16),   -- Level 6: 180% bonus + 16 extra fish
('fishmonger_shop', 'fish', 7, 3.4, 22),   -- Level 7: 240% bonus + 22 extra fish
('fishmonger_shop', 'fish', 8, 4.1, 30),   -- Level 8: 310% bonus + 30 extra fish
('fishmonger_shop', 'fish', 9, 5.0, 40);   -- Level 9: 400% bonus + 40 extra fish

-- Herbalist's Hut multipliers (herb category)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('herbalist_hut', 'herbs', 1, 1.0, 0),      -- Base rate
('herbalist_hut', 'herbs', 2, 1.3, 1),      -- Level 2: 30% bonus + 1 extra herb
('herbalist_hut', 'herbs', 3, 1.7, 2),      -- Level 3: 70% bonus + 2 extra herbs
('herbalist_hut', 'herbs', 4, 2.1, 3),      -- Level 4: 110% bonus + 3 extra herbs
('herbalist_hut', 'herbs', 5, 2.6, 5),      -- Level 5: 160% bonus + 5 extra herbs
('herbalist_hut', 'herbs', 6, 3.2, 8),      -- Level 6: 220% bonus + 8 extra herbs
('herbalist_hut', 'herbs', 7, 3.9, 12),     -- Level 7: 290% bonus + 12 extra herbs
('herbalist_hut', 'herbs', 8, 4.7, 17),     -- Level 8: 370% bonus + 17 extra herbs
('herbalist_hut', 'herbs', 9, 5.8, 25);     -- Level 9: 480% bonus + 25 extra herbs

-- Farming Guild multipliers (farming category)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('farming_guild', 'farming', 1, 1.0, 0),    -- Base rate
('farming_guild', 'farming', 2, 1.3, 2),    -- Level 2: 30% bonus + 2 extra seeds
('farming_guild', 'farming', 3, 1.7, 4),    -- Level 3: 70% bonus + 4 extra seeds
('farming_guild', 'farming', 4, 2.2, 7),    -- Level 4: 120% bonus + 7 extra seeds
('farming_guild', 'farming', 5, 2.7, 11),   -- Level 5: 170% bonus + 11 extra seeds
('farming_guild', 'farming', 6, 3.3, 16),   -- Level 6: 230% bonus + 16 extra seeds
('farming_guild', 'farming', 7, 4.0, 23),   -- Level 7: 300% bonus + 23 extra seeds
('farming_guild', 'farming', 8, 4.8, 32),   -- Level 8: 380% bonus + 32 extra seeds
('farming_guild', 'farming', 9, 6.0, 45);   -- Level 9: 500% bonus + 45 extra seeds

-- Rune Shop multipliers (rune category)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('rune_shop', 'runes', 1, 1.0, 0),           -- Base rate
('rune_shop', 'runes', 2, 1.2, 2),           -- Level 2: 20% bonus + 2 extra runes
('rune_shop', 'runes', 3, 1.5, 4),           -- Level 3: 50% bonus + 4 extra runes
('rune_shop', 'runes', 4, 1.9, 7),           -- Level 4: 90% bonus + 7 extra runes
('rune_shop', 'runes', 5, 2.3, 11),          -- Level 5: 130% bonus + 11 extra runes
('rune_shop', 'runes', 6, 2.8, 16),          -- Level 6: 180% bonus + 16 extra runes
('rune_shop', 'runes', 7, 3.5, 22),          -- Level 7: 250% bonus + 22 extra runes
('rune_shop', 'runes', 8, 4.3, 30),          -- Level 8: 330% bonus + 30 extra runes
('rune_shop', 'runes', 9, 5.5, 40);          -- Level 9: 450% bonus + 40 extra runes

-- Treasury multipliers for currency
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('treasury', 'coins', 1, 1.0, 0),        -- Base rate
('treasury', 'coins', 2, 1.2, 15),       -- Level 2: 20% bonus + 15 extra coins
('treasury', 'coins', 3, 1.5, 35),       -- Level 3: 50% bonus + 35 extra coins
('treasury', 'coins', 4, 1.8, 60),       -- Level 4: 80% bonus + 60 extra coins
('treasury', 'coins', 5, 2.2, 100),      -- Level 5: 120% bonus + 100 extra coins
('treasury', 'coins', 6, 2.7, 150),      -- Level 6: 170% bonus + 150 extra coins
('treasury', 'coins', 7, 3.3, 225),      -- Level 7: 230% bonus + 225 extra coins
('treasury', 'coins', 8, 4.0, 325),      -- Level 8: 300% bonus + 325 extra coins
('treasury', 'coins', 9, 5.0, 500);      -- Level 9: 400% bonus + 500 extra coins

-- Crafting Guild multipliers for gems (adjusted to be high-reward but more balanced)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('crafting_guild', 'gems', 1, 1.0, 0),     -- Base rate
('crafting_guild', 'gems', 2, 1.5, 1),     -- Level 2: 50% bonus + 1 extra gem
('crafting_guild', 'gems', 3, 2.5, 2),     -- Level 3: 150% bonus + 2 extra gems
('crafting_guild', 'gems', 4, 4.0, 3),     -- Level 4: 300% bonus + 3 extra gems
('crafting_guild', 'gems', 5, 7.0, 5),     -- Level 5: 600% bonus + 5 extra gems
('crafting_guild', 'gems', 6, 12.0, 8),    -- Level 6: 1100% bonus + 8 extra gems
('crafting_guild', 'gems', 7, 20.0, 12),   -- Level 7: 1900% bonus + 12 extra gems
('crafting_guild', 'gems', 8, 35.0, 18),   -- Level 8: 3400% bonus + 18 extra gems
('crafting_guild', 'gems', 9, 60.0, 25);   -- Level 9: 5900% bonus + 25 extra gems

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