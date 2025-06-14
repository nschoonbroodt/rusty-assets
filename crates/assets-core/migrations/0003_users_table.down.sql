-- Drop foreign key constraint and users table
ALTER TABLE transactions DROP CONSTRAINT IF EXISTS fk_transactions_created_by;
DROP TABLE IF EXISTS users;