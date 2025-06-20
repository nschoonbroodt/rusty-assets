CREATE TABLE imported_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_path VARCHAR(500) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_hash VARCHAR(64) NOT NULL, -- SHA-256 hash
    file_size BIGINT NOT NULL,
    import_source VARCHAR(50) NOT NULL, -- 'BoursoBank', 'SocieteGenerale', 'Payslip', etc.
    import_batch_id UUID NOT NULL,
    imported_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    transaction_count INTEGER NOT NULL DEFAULT 0, -- Number of transactions imported from this file
    notes TEXT,
    CONSTRAINT uq_file_hash UNIQUE (file_hash),
    CONSTRAINT uq_file_path_source UNIQUE (file_path, import_source)
);