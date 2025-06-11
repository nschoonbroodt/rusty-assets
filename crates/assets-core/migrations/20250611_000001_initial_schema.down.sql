-- Drop tables in reverse order (respecting foreign key constraints)
DROP TABLE IF EXISTS real_estate;
DROP TABLE IF EXISTS investments;
DROP TABLE IF EXISTS transactions;
DROP TABLE IF EXISTS categories;
DROP TABLE IF EXISTS accounts;
-- Drop custom types
DROP TYPE IF EXISTS investment_type;
DROP TYPE IF EXISTS account_type;
-- Drop the update function
DROP FUNCTION IF EXISTS update_updated_at_column();
-- Drop UUID extension (only if no other tables use it)
-- DROP EXTENSION IF EXISTS "uuid-ossp";