CREATE TABLE slayer_master_level_mapping (
    slayer_master_level INTEGER PRIMARY KEY,
    slayer_level INTEGER NOT NULL
);

-- Populate with sample data
INSERT INTO slayer_master_level_mapping (slayer_master_level, slayer_level) VALUES
(1, 1),
(2, 50),
(3, 60),
(4, 70),
(5, 75),
(6, 80),
(7, 85),
(8, 90),
(9, 99);