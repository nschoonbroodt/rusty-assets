-- Add up migration script here

-- Auto-assign 100% ownership of all accounts to the first user
-- This trigger ensures that when the first user is created, they automatically
-- get 100% ownership of all existing accounts, eliminating the need for manual assignment.

-- Create function to handle automatic ownership assignment
CREATE OR REPLACE FUNCTION assign_ownership_to_first_user()
RETURNS TRIGGER AS $$
DECLARE
    user_count_before INTEGER;
    account_record RECORD;
    accounts_assigned INTEGER := 0;
BEGIN
    -- Count how many users existed before this insertion
    -- We subtract 1 because the current user is already inserted
    SELECT COUNT(*) - 1 INTO user_count_before FROM users;
    
    -- If this is the first user (user_count_before = 0)
    IF user_count_before = 0 THEN
        -- Assign 100% ownership of all accounts to this new user
        -- Only assign accounts that don't already have ownership
        FOR account_record IN 
            SELECT id FROM accounts 
            WHERE id NOT IN (SELECT DISTINCT account_id FROM account_ownership)
        LOOP
            INSERT INTO account_ownership (user_id, account_id, ownership_percentage)
            VALUES (NEW.id, account_record.id, 1.0)
            ON CONFLICT (user_id, account_id) DO NOTHING;
            
            accounts_assigned := accounts_assigned + 1;
        END LOOP;
        
        -- Log the assignment for debugging/auditing purposes
        RAISE NOTICE 'Auto-assigned 100%% ownership of % accounts to first user: % (ID: %)', 
                     accounts_assigned, NEW.name, NEW.id;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger that fires after user insertion
CREATE OR REPLACE TRIGGER trigger_assign_ownership_to_first_user
    AFTER INSERT ON users
    FOR EACH ROW
    EXECUTE FUNCTION assign_ownership_to_first_user();
