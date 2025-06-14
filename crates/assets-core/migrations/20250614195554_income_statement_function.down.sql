DROP FUNCTION IF EXISTS fn_income_statement(
    p_user_ids UUID[],
    p_start_date DATE,
    p_end_date DATE
);