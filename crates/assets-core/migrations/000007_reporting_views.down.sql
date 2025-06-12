-- Migration: 000007_reporting_views.down.sql
-- Description: Reverts the creation of reporting views.
DROP VIEW IF EXISTS latest_account_market_values;
DROP VIEW IF EXISTS daily_account_market_values;
DROP VIEW IF EXISTS account_monthly_sampled_balances;