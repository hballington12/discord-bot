-- Add Crafting Guild multipliers for crafting-related resources
INSERT INTO resource_multiplier_mapping (building_name, resource_category, building_level, multiplier, flat_bonus) VALUES
-- Crafting Guild multipliers (crafting category)
('crafting_guild', 'crafting', 0, 1.0, 0),    -- Base rate with no building
('crafting_guild', 'crafting', 1, 2.0, 1),   -- Level 1: 15% bonus + 1 extra gem
('crafting_guild', 'crafting', 2, 5.0, 1),    -- Level 2: 30% bonus + 1 extra gem
('crafting_guild', 'crafting', 3, 10.0, 2),   -- Level 3: 45% bonus + 2 extra gems
('crafting_guild', 'crafting', 4, 20.0, 2),    -- Level 4: 60% bonus + 2 extra gems
('crafting_guild', 'crafting', 5, 40.0, 3),    -- Level 5: 80% bonus + 3 extra gems
('crafting_guild', 'crafting', 6, 80.0, 4),    -- Level 6: 100% bonus + 4 extra gems
('crafting_guild', 'crafting', 7, 160.0, 5),   -- Level 7: 125% bonus + 5 extra gems
('crafting_guild', 'crafting', 8, 320.0, 7),    -- Level 8: 150% bonus + 7 extra gems
('crafting_guild', 'crafting', 9, 640.0, 10);   -- Level 9: 200% bonus + 10 extra gems