-- Migration to add file import tracking to prevent duplicate imports

-- Table to track imported files
CREATE TABLE imported_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_path VARCHAR(500) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_hash VARCHAR(64) NOT NULL, -- SHA-256 hash
    file_size BIGINT NOT NULL,
    import_source VARCHAR(50) NOT NULL, -- 'BoursoBank', 'SocieteGenerale', 'Payslip', etc.
    import_batch_id UUID NOT NULL,
    imported_by UUID NOT NULL REFERENCES users(id),
    imported_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    transaction_count INTEGER NOT NULL DEFAULT 0, -- Number of transactions imported from this file
    notes TEXT,
    CONSTRAINT uq_file_hash UNIQUE (file_hash),
    CONSTRAINT uq_file_path_source UNIQUE (file_path, import_source)
);

-- Create indexes for efficient lookups
CREATE INDEX idx_imported_files_hash ON imported_files(file_hash);
CREATE INDEX idx_imported_files_source ON imported_files(import_source);
CREATE INDEX idx_imported_files_batch ON imported_files(import_batch_id);
CREATE INDEX idx_imported_files_imported_at ON imported_files(imported_at);

-- View to show file import history with user information
CREATE VIEW v_imported_files_history AS
SELECT 
    if.id,
    if.file_name,
    if.file_path,
    if.file_size,
    if.import_source,
    if.transaction_count,
    if.imported_at,
    u.name as imported_by_user,
    if.notes
FROM imported_files if
JOIN users u ON if.imported_by = u.id
ORDER BY if.imported_at DESC;
