-- Sample tables for testing FugDB with MySQL
CREATE TABLE IF NOT EXISTS products (
    id INT AUTO_INCREMENT PRIMARY KEY,
    sku VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(150) NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    stock_quantity INT DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS inventory_logs (
    id INT AUTO_INCREMENT PRIMARY KEY,
    product_id INT NOT NULL,
    change_amount INT NOT NULL,
    reason VARCHAR(100),
    logged_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
);

INSERT INTO products (sku, name, price, stock_quantity) VALUES
('FUG-001', 'Mechanical Keyboard Pro', 149.99, 45),
('FUG-002', 'Wireless Gaming Mouse', 79.50, 120),
('FUG-003', 'Ultra-wide Monitor 34"', 599.00, 15);

INSERT INTO inventory_logs (product_id, change_amount, reason) VALUES
(1, 45, 'Initial stock import'),
(2, 120, 'Initial stock import'),
(3, 15, 'Initial stock import');
