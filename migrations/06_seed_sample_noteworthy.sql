
-- Get the ID of the sample team
INSERT INTO noteworthy_items (id, name)
SELECT 0, 'bones'
WHERE NOT EXISTS (SELECT 1 FROM noteworthy_items WHERE name = 'bones');