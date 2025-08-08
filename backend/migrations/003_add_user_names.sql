-- Add first_name and last_name columns to users table
ALTER TABLE users 
ADD COLUMN first_name VARCHAR(255) NOT NULL DEFAULT '',
ADD COLUMN last_name VARCHAR(255) NOT NULL DEFAULT '';

-- Update existing users with default names if any exist
UPDATE users SET first_name = 'Имя', last_name = 'Фамилия' WHERE first_name = '' OR last_name = '';