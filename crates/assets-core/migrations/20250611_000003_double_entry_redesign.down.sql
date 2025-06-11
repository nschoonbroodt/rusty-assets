-- Reverse the double-entry redesign
DROP TRIGGER IF EXISTS check_journal_entry_balance ON journal_entries;
DROP FUNCTION IF EXISTS check_transaction_balance();
DROP TABLE IF EXISTS journal_entries CASCADE;
DROP TABLE IF EXISTS transactions CASCADE;
DROP TABLE IF EXISTS accounts CASCADE;
DROP TYPE IF EXISTS account_subtype CASCADE;
DROP TYPE IF EXISTS account_type CASCADE;