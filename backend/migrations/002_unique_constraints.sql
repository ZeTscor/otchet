-- Add unique constraints to ensure one screening and one interview per application
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'unique_application_screening') THEN
        ALTER TABLE screenings ADD CONSTRAINT unique_application_screening UNIQUE (application_id);
    END IF;
END $$;

DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'unique_application_interview') THEN
        ALTER TABLE interviews ADD CONSTRAINT unique_application_interview UNIQUE (application_id);
    END IF;
END $$;