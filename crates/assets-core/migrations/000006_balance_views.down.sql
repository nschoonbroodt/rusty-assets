-- Migration: 000006_balance_views.down.sql
-- Description: Reverts the creation of balance views.

DROP VIEW IF EXISTS account_running_balances;
DROP VIEW IF EXISTS account_daily_balance_changes;
