-- Insert some default categories for common expense tracking
INSERT INTO categories (id, name, color, is_active)
VALUES (
        uuid_generate_v4(),
        'Food & Dining',
        '#FF6B6B',
        true
    ),
    (
        uuid_generate_v4(),
        'Transportation',
        '#4ECDC4',
        true
    ),
    (uuid_generate_v4(), 'Shopping', '#45B7D1', true),
    (
        uuid_generate_v4(),
        'Entertainment',
        '#96CEB4',
        true
    ),
    (
        uuid_generate_v4(),
        'Bills & Utilities',
        '#FFEAA7',
        true
    ),
    (
        uuid_generate_v4(),
        'Healthcare',
        '#DDA0DD',
        true
    ),
    (uuid_generate_v4(), 'Education', '#98D8C8', true),
    (uuid_generate_v4(), 'Travel', '#F7DC6F', true),
    (uuid_generate_v4(), 'Income', '#82E0AA', true),
    (
        uuid_generate_v4(),
        'Investments',
        '#AED6F1',
        true
    ),
    (uuid_generate_v4(), 'Savings', '#F8C471', true),
    (uuid_generate_v4(), 'Other', '#D2B4DE', true);
-- Insert subcategories for Food & Dining
INSERT INTO categories (id, name, parent_id, color, is_active)
VALUES (
        uuid_generate_v4(),
        'Restaurants',
        (
            SELECT id
            FROM categories
            WHERE name = 'Food & Dining'
        ),
        '#FF6B6B',
        true
    ),
    (
        uuid_generate_v4(),
        'Groceries',
        (
            SELECT id
            FROM categories
            WHERE name = 'Food & Dining'
        ),
        '#FF6B6B',
        true
    ),
    (
        uuid_generate_v4(),
        'Coffee & Cafes',
        (
            SELECT id
            FROM categories
            WHERE name = 'Food & Dining'
        ),
        '#FF6B6B',
        true
    );
-- Insert subcategories for Transportation
INSERT INTO categories (id, name, parent_id, color, is_active)
VALUES (
        uuid_generate_v4(),
        'Gas & Fuel',
        (
            SELECT id
            FROM categories
            WHERE name = 'Transportation'
        ),
        '#4ECDC4',
        true
    ),
    (
        uuid_generate_v4(),
        'Public Transport',
        (
            SELECT id
            FROM categories
            WHERE name = 'Transportation'
        ),
        '#4ECDC4',
        true
    ),
    (
        uuid_generate_v4(),
        'Car Maintenance',
        (
            SELECT id
            FROM categories
            WHERE name = 'Transportation'
        ),
        '#4ECDC4',
        true
    );
-- Insert subcategories for Bills & Utilities
INSERT INTO categories (id, name, parent_id, color, is_active)
VALUES (
        uuid_generate_v4(),
        'Electricity',
        (
            SELECT id
            FROM categories
            WHERE name = 'Bills & Utilities'
        ),
        '#FFEAA7',
        true
    ),
    (
        uuid_generate_v4(),
        'Internet',
        (
            SELECT id
            FROM categories
            WHERE name = 'Bills & Utilities'
        ),
        '#FFEAA7',
        true
    ),
    (
        uuid_generate_v4(),
        'Phone',
        (
            SELECT id
            FROM categories
            WHERE name = 'Bills & Utilities'
        ),
        '#FFEAA7',
        true
    ),
    (
        uuid_generate_v4(),
        'Rent/Mortgage',
        (
            SELECT id
            FROM categories
            WHERE name = 'Bills & Utilities'
        ),
        '#FFEAA7',
        true
    );