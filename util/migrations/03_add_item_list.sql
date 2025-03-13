-- Migration to add resources tracking for teams
CREATE TABLE resources (
    id SERIAL PRIMARY KEY,
    team_id INTEGER NOT NULL,
    resource_name VARCHAR(100) NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 0,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    UNIQUE (team_id, resource_name)
);

-- Add index for faster lookups by team
CREATE INDEX idx_resources_team_id ON resources(team_id);

-- Optional: Create a view for resource totals per team
CREATE VIEW team_resource_summary AS
SELECT 
    t.name as team_name,
    t.id as team_id,
    r.resource_name,
    r.quantity
FROM teams t
JOIN resources r ON t.id = r.team_id
ORDER BY t.name, r.resource_name;