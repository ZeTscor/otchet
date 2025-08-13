-- Simple Performance Optimizations Migration
-- This migration adds basic indexes and constraints without complex predicates

-- Add basic composite indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_applications_user_status 
ON applications(user_id, status);

CREATE INDEX IF NOT EXISTS idx_applications_company_date 
ON applications(company, applied_date DESC);

CREATE INDEX IF NOT EXISTS idx_applications_created_at_user 
ON applications(created_at DESC, user_id);

-- Add simple date-based indexes without predicates
CREATE INDEX IF NOT EXISTS idx_applications_applied_date 
ON applications(applied_date);

CREATE INDEX IF NOT EXISTS idx_screenings_created_at 
ON screenings(created_at);

CREATE INDEX IF NOT EXISTS idx_interviews_created_at 
ON interviews(created_at);

-- Add result-based indexes 
CREATE INDEX IF NOT EXISTS idx_screenings_result_date 
ON screenings(result, screening_date DESC);

CREATE INDEX IF NOT EXISTS idx_interviews_result_date 
ON interviews(result, interview_date DESC);

-- Add updated_at index for stale query
CREATE INDEX IF NOT EXISTS idx_applications_updated_status 
ON applications(updated_at, status);

-- Add basic constraint for applied_date (simplified)
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'check_applied_date_reasonable') THEN
        ALTER TABLE applications ADD CONSTRAINT check_applied_date_reasonable 
        CHECK (applied_date >= '2020-01-01' AND applied_date <= CURRENT_DATE + INTERVAL '1 day');
    END IF;
END $$;

-- Add table for caching query results
CREATE TABLE IF NOT EXISTS cache_store (
    key VARCHAR(500) PRIMARY KEY,
    value JSONB NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cache_expires ON cache_store(expires_at);

-- Add table for audit logging
CREATE TABLE IF NOT EXISTS audit_log (
    id BIGSERIAL PRIMARY KEY,
    table_name VARCHAR(100) NOT NULL,
    operation VARCHAR(10) NOT NULL, -- INSERT, UPDATE, DELETE
    old_data JSONB,
    new_data JSONB,
    user_id INTEGER REFERENCES users(id),
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_table_operation ON audit_log(table_name, operation);
CREATE INDEX IF NOT EXISTS idx_audit_log_user ON audit_log(user_id);

-- Generic audit trigger function
CREATE OR REPLACE FUNCTION audit_trigger()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO audit_log (table_name, operation, old_data, new_data, user_id)
    VALUES (
        TG_TABLE_NAME,
        TG_OP,
        CASE WHEN TG_OP = 'DELETE' THEN row_to_json(OLD) ELSE NULL END,
        CASE WHEN TG_OP IN ('INSERT', 'UPDATE') THEN row_to_json(NEW) ELSE NULL END,
        CASE 
            WHEN TG_OP = 'DELETE' THEN OLD.user_id 
            WHEN TG_TABLE_NAME = 'users' THEN NEW.id
            ELSE NEW.user_id 
        END
    );
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Add audit triggers to critical tables
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'audit_applications') THEN
        CREATE TRIGGER audit_applications AFTER INSERT OR UPDATE OR DELETE ON applications
        FOR EACH ROW EXECUTE FUNCTION audit_trigger();
    END IF;
END $$;

DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'audit_users') THEN
        CREATE TRIGGER audit_users AFTER INSERT OR UPDATE OR DELETE ON users  
        FOR EACH ROW EXECUTE FUNCTION audit_trigger();
    END IF;
END $$;

-- Update table statistics
ANALYZE applications;
ANALYZE screenings; 
ANALYZE interviews;
ANALYZE users;