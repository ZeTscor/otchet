-- Add first_name and last_name columns to users table
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'first_name') THEN
        ALTER TABLE users ADD COLUMN first_name VARCHAR(255) NOT NULL DEFAULT '';
    END IF;
END $$;

DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'last_name') THEN
        ALTER TABLE users ADD COLUMN last_name VARCHAR(255) NOT NULL DEFAULT '';
    END IF;
END $$;

-- Update existing users with default names if any exist
UPDATE users SET first_name = 'Имя', last_name = 'Фамилия' WHERE first_name = '' OR last_name = '';