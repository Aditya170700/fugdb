-- Sample tables for testing FugDB with PostgreSQL
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    role VARCHAR(50) DEFAULT 'member',
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    order_number VARCHAR(50) UNIQUE NOT NULL,
    total_amount NUMERIC(12, 2) NOT NULL,
    status VARCHAR(30) DEFAULT 'pending',
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Seed initial data
INSERT INTO users (name, email, role, is_active) VALUES
('Aditya Pratama', 'aditya@fugdb.dev', 'admin', true),
('Jane Doe', 'jane@example.com', 'developer', true),
('John Smith', 'john@example.com', 'qa', false);

INSERT INTO orders (user_id, order_number, total_amount, status, metadata) VALUES
(1, 'ORD-2026-001', 299.50, 'completed', '{"source": "web", "coupon": "WELCOME2026"}'),
(1, 'ORD-2026-002', 1250.00, 'processing', '{"source": "mobile", "priority": "high"}'),
(2, 'ORD-2026-003', 45.00, 'pending', '{"source": "api"}');
