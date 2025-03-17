-- Migration to add resources tracking for teams
CREATE TABLE resources (
    id SERIAL PRIMARY KEY,
    team_id INTEGER NOT NULL,
    name VARCHAR(100) NOT NULL,
    category VARCHAR(50) NOT NULL DEFAULT 'misc',
    quantity INTEGER NOT NULL DEFAULT 0,
    
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    UNIQUE (team_id, name)
);

-- Add index for faster lookups by team
CREATE INDEX idx_resources_team_id ON resources(team_id);

-- Add index for category to improve filtering
CREATE INDEX idx_resources_category ON resources(category);

-- Optional: Create a view for resource totals per team
CREATE VIEW team_resource_summary AS
SELECT 
    t.name as team_name,
    t.id as team_id,
    r.name as resource_name,
    r.category as resource_category,
    r.quantity
FROM teams t
JOIN resources r ON t.id = r.team_id
ORDER BY t.name, r.category, r.name;

-- Create view for resource totals by category
CREATE VIEW team_category_summary AS
SELECT
    t.name as team_name,
    t.id as team_id,
    r.category,
    SUM(r.quantity) as total_quantity,
    COUNT(r.id) as unique_resources
FROM teams t
JOIN resources r ON t.id = r.team_id
GROUP BY t.id, t.name, r.category
ORDER BY t.name, r.category;