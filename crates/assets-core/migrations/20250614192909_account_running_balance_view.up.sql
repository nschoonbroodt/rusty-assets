-- View to calculate the running balance based on daily changes
CREATE VIEW account_running_balances AS
SELECT adc.account_id,
    acc.name AS account_name,
    acc.account_type,
    acc.account_subtype,
    adc.balance_day,
    adc.net_change_on_day,
    SUM(adc.net_change_on_day) OVER (
        PARTITION BY adc.account_id
        ORDER BY adc.balance_day ASC
    ) AS running_balance
FROM account_daily_balance_changes adc
    JOIN accounts acc ON adc.account_id = acc.id;