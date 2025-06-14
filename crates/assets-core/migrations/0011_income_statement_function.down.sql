-- migrations/000011_income_statement_function.down.sql
DROP FUNCTION IF EXISTS fn_income_statement(INT[], DATE, DATE); -- Changed to match the modified function signature
