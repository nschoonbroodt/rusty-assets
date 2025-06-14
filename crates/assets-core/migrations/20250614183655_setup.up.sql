-- Create UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- Enable pg_trgm extension for text similarity if not already enabled
CREATE EXTENSION IF NOT EXISTS pg_trgm;