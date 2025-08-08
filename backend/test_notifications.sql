-- Test script to create sample data for testing notifications

-- Create test user
INSERT INTO users (email, password_hash, role) 
VALUES ('testuser@example.com', '$2b$12$dummy.hash.for.testing', 'student')
ON CONFLICT (email) DO NOTHING;

-- Get user ID
DO $$
DECLARE
    test_user_id integer;
BEGIN
    SELECT id INTO test_user_id FROM users WHERE email = 'testuser@example.com';
    
    -- Create test applications with old updated_at dates (simulate stale applications)
    INSERT INTO applications (user_id, company, job_url, applied_date, status, created_at, updated_at)
    VALUES 
        (test_user_id, 'Google', 'https://careers.google.com/jobs/123', '2024-01-01', 'waiting', 
         NOW() - INTERVAL '10 days', NOW() - INTERVAL '10 days'),
        (test_user_id, 'Microsoft', 'https://careers.microsoft.com/jobs/456', '2024-01-02', 'next_stage', 
         NOW() - INTERVAL '8 days', NOW() - INTERVAL '8 days'),
        (test_user_id, 'Amazon', 'https://amazon.jobs/en/jobs/789', '2024-01-03', 'rejected', 
         NOW() - INTERVAL '5 days', NOW() - INTERVAL '5 days'),
        (test_user_id, 'Facebook', 'https://www.facebook.com/careers/jobs/101', '2024-01-04', 'waiting', 
         NOW() - INTERVAL '3 days', NOW() - INTERVAL '3 days')
    ON CONFLICT DO NOTHING;
         
    RAISE NOTICE 'Test data created for user ID: %', test_user_id;
END $$;

-- Query to check stale applications (should return 2 applications - Google and Microsoft)
SELECT 
    u.email,
    a.company,
    a.status,
    a.updated_at,
    NOW() - a.updated_at as days_since_update
FROM applications a
JOIN users u ON u.id = a.user_id
WHERE a.updated_at < NOW() - INTERVAL '7 days' 
AND a.status IN ('waiting', 'next_stage')
ORDER BY a.updated_at ASC;