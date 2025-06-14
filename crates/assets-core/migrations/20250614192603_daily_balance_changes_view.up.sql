-- Description: Creates views for daily account balance changes and running balances.
-- View to get the net change for each account on each day transactions occurred
CREATE VIEW account_daily_balance_changes AS
SELECT j.account_id,
    t.transaction_date::date AS balance_day,
    -- Cast to date to group by day
    SUM(j.amount) AS net_change_on_day
FROM journal_entries j
    JOIN transactions t ON j.transaction_id = t.id
GROUP BY j.account_id,
    t.transaction_date::date;