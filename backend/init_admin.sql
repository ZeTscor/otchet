-- Script для создания первого администратора
-- Пароль: admin123 (хешируется с bcrypt cost 12)

INSERT INTO users (email, password_hash, role, created_at, updated_at)
VALUES (
    'admin@example.com',
    '$2b$12$LKgRAKHovekFMxjEJNf95uJkOUV3MmCPD/ZAzPvw6fXJbQFhkTZ1i', -- admin123
    'admin',
    NOW(),
    NOW()
) 
ON CONFLICT (email) DO NOTHING;

-- Проверим что админ создан
SELECT id, email, role, created_at FROM users WHERE role = 'admin';