-- Migration to add combat and slayer level limits to teams

-- First, add new columns to the teams table
ALTER TABLE teams 
ADD COLUMN max_combat_level INTEGER NOT NULL DEFAULT 50;

ALTER TABLE teams 
ADD COLUMN max_slayer_level INTEGER NOT NULL DEFAULT 1;