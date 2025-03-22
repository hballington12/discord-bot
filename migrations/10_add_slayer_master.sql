CREATE TABLE slayer_master_level_mapping (
    slayer_master_level INTEGER PRIMARY KEY,
    slayer_level INTEGER NOT NULL
);

-- Populate with sample data
INSERT INTO slayer_master_level_mapping (slayer_master_level, slayer_level) VALUES
(1, 1),
(2, 70),
(3, 75),
(4, 80),
(5, 85),
(6, 87),
(7, 91),
(8, 92),
(9, 99);