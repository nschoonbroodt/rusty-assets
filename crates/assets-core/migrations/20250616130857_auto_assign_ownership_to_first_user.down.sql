-- Add down migration script here

-- Remove auto-ownership assignment trigger and function
DROP TRIGGER IF EXISTS trigger_assign_ownership_to_first_user ON users;
DROP FUNCTION IF EXISTS assign_ownership_to_first_user();
