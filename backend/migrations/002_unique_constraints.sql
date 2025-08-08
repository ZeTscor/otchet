-- Add unique constraints to ensure one screening and one interview per application
ALTER TABLE screenings ADD CONSTRAINT unique_application_screening UNIQUE (application_id);
ALTER TABLE interviews ADD CONSTRAINT unique_application_interview UNIQUE (application_id);