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
('woodcutter_lodge', 'wood', 0, 1.0, 0),    -- Base rate with no building
('woodcutter_lodge', 'wood', 1, 1.2, 1),    -- Level 1: 20% bonus + 1 extra
('woodcutter_lodge', 'wood', 2, 1.4, 2),    -- Level 2: 40% bonus + 2 extra
('woodcutter_lodge', 'wood', 3, 1.6, 3),    -- Level 3: 60% bonus + 3 extra
('woodcutter_lodge', 'wood', 4, 1.8, 5),    -- Level 4: 80% bonus + 5 extra
('woodcutter_lodge', 'wood', 5, 2.0, 8),    -- Level 5: 100% bonus + 8 extra
('woodcutter_lodge', 'wood', 6, 2.2, 12),   -- Level 6: 120% bonus + 12 extra
('woodcutter_lodge', 'wood', 7, 2.4, 18),   -- Level 7: 140% bonus + 18 extra
('woodcutter_lodge', 'wood', 8, 2.6, 25),   -- Level 8: 160% bonus + 25 extra
('woodcutter_lodge', 'wood', 9, 3.0, 35);   -- Level 9: 200% bonus + 35 extra

-- Prospector's Shop multipliers (mining category)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('prospector_shop', 'mining', 0, 1.0, 0),   -- Base rate with no building
('prospector_shop', 'mining', 1, 1.2, 1),   -- Level 1: 20% bonus + 1 extra ore
('prospector_shop', 'mining', 2, 1.4, 2),   -- Level 2: 40% bonus + 2 extra ore
('prospector_shop', 'mining', 3, 1.6, 3),   -- Level 3: 60% bonus + 3 extra ore
('prospector_shop', 'mining', 4, 1.8, 4),   -- Level 4: 80% bonus + 4 extra ore
('prospector_shop', 'mining', 5, 2.0, 6),   -- Level 5: 100% bonus + 6 extra ore
('prospector_shop', 'mining', 6, 2.2, 9),   -- Level 6: 120% bonus + 9 extra ore
('prospector_shop', 'mining', 7, 2.4, 13),  -- Level 7: 140% bonus + 13 extra ore
('prospector_shop', 'mining', 8, 2.6, 18),  -- Level 8: 160% bonus + 18 extra ore
('prospector_shop', 'mining', 9, 3.0, 25);  -- Level 9: 200% bonus + 25 extra ore

-- Fishmonger's Shop multipliers (fishing category)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('fishmonger_shop', 'fishing', 0, 1.0, 0),  -- Base rate with no building
('fishmonger_shop', 'fishing', 1, 1.2, 1),  -- Level 1: 20% bonus + 1 extra fish
('fishmonger_shop', 'fishing', 2, 1.4, 2),  -- Level 2: 40% bonus + 2 extra fish
('fishmonger_shop', 'fishing', 3, 1.6, 3),  -- Level 3: 60% bonus + 3 extra fish
('fishmonger_shop', 'fishing', 4, 1.8, 4),  -- Level 4: 80% bonus + 4 extra fish
('fishmonger_shop', 'fishing', 5, 2.0, 6),  -- Level 5: 100% bonus + 6 extra fish
('fishmonger_shop', 'fishing', 6, 2.2, 9),  -- Level 6: 120% bonus + 9 extra fish
('fishmonger_shop', 'fishing', 7, 2.4, 12), -- Level 7: 140% bonus + 12 extra fish
('fishmonger_shop', 'fishing', 8, 2.6, 16), -- Level 8: 160% bonus + 16 extra fish
('fishmonger_shop', 'fishing', 9, 3.0, 22); -- Level 9: 200% bonus + 22 extra fish

-- Herbalist's Hut multipliers (herb category)
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('herbalist_hut', 'herb', 0, 1.0, 0),      -- Base rate with no building
('herbalist_hut', 'herb', 1, 1.2, 0),      -- Level 1: 20% bonus (herbs are rarer, so no flat bonus at low levels)
('herbalist_hut', 'herb', 2, 1.4, 1),      -- Level 2: 40% bonus + 1 extra herb
('herbalist_hut', 'herb', 3, 1.6, 1),      -- Level 3: 60% bonus + 1 extra herb
('herbalist_hut', 'herb', 4, 1.8, 2),      -- Level 4: 80% bonus + 2 extra herbs
('herbalist_hut', 'herb', 5, 2.0, 3),      -- Level 5: 100% bonus + 3 extra herbs
('herbalist_hut', 'herb', 6, 2.2, 4),      -- Level 6: 120% bonus + 4 extra herbs
('herbalist_hut', 'herb', 7, 2.4, 6),      -- Level 7: 140% bonus + 6 extra herbs
('herbalist_hut', 'herb', 8, 2.6, 8),      -- Level 8: 160% bonus + 8 extra herbs
('herbalist_hut', 'herb', 9, 3.0, 12);     -- Level 9: 200% bonus + 12 extra herbs

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
('rune_shop', 'rune', 0, 1.0, 0),          -- Base rate with no building
('rune_shop', 'rune', 1, 1.1, 1),          -- Level 1: 10% bonus + 1 extra rune
('rune_shop', 'rune', 2, 1.2, 2),          -- Level 2: 20% bonus + 2 extra runes
('rune_shop', 'rune', 3, 1.3, 3),          -- Level 3: 30% bonus + 3 extra runes
('rune_shop', 'rune', 4, 1.4, 4),          -- Level 4: 40% bonus + 4 extra runes
('rune_shop', 'rune', 5, 1.5, 5),          -- Level 5: 50% bonus + 5 extra runes
('rune_shop', 'rune', 6, 1.6, 7),          -- Level 6: 60% bonus + 7 extra runes
('rune_shop', 'rune', 7, 1.8, 9),          -- Level 7: 80% bonus + 9 extra runes
('rune_shop', 'rune', 8, 2.0, 12),         -- Level 8: 100% bonus + 12 extra runes
('rune_shop', 'rune', 9, 2.5, 15);         -- Level 9: 150% bonus + 15 extra runes

-- Treasury multipliers for currency
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
('treasury', 'currency', 0, 1.0, 0),       -- Base rate with no building
('treasury', 'currency', 1, 1.1, 5),       -- Level 1: 10% bonus + 5 extra coins
('treasury', 'currency', 2, 1.2, 10),      -- Level 2: 20% bonus + 10 extra coins
('treasury', 'currency', 3, 1.3, 20),      -- Level 3: 30% bonus + 20 extra coins
('treasury', 'currency', 4, 1.4, 35),      -- Level 4: 40% bonus + 35 extra coins
('treasury', 'currency', 5, 1.5, 50),      -- Level 5: 50% bonus + 50 extra coins
('treasury', 'currency', 6, 1.6, 75),      -- Level 6: 60% bonus + 75 extra coins
('treasury', 'currency', 7, 1.7, 100),     -- Level 7: 70% bonus + 100 extra coins
('treasury', 'currency', 8, 1.8, 150),     -- Level 8: 80% bonus + 150 extra coins
('treasury', 'currency', 9, 2.0, 250);     -- Level 9: 100% bonus + 250 extra coins

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