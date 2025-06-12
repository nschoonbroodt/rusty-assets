-- Migration: 000006_balance_views.up.sql
-- Description: Creates views for daily account balance changes and running balances.

-- View to get the net change for each account on each day transactions occurred
CREATE VIEW account_daily_balance_changes AS
SELECT
    j.account_id,
    t.transaction_date::date AS balance_day, -- Cast to date to group by day
    SUM(j.amount) AS net_change_on_day
FROM
    journal_entries j
JOIN
    transactions t ON j.transaction_id = t.id
GROUP BY
    j.account_id,
    t.transaction_date::date;

-- View to calculate the running balance based on daily changes
CREATE VIEW account_running_balances AS
SELECT
    adc.account_id,
    acc.name AS account_name,
    acc.code AS account_code,
    acc.account_type,
    acc.account_subtype,
    adc.balance_day,
    adc.net_change_on_day,
    SUM(adc.net_change_on_day) OVER (PARTITION BY adc.account_id ORDER BY adc.balance_day ASC) AS running_balance
FROM
    account_daily_balance_changes adc
JOIN
    accounts acc ON adc.account_id = acc.id;
