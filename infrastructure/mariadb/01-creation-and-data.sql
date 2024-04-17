-- Crear la base de datos si no existe
CREATE DATABASE IF NOT EXISTS mydatabase;
USE mydatabase;

-- Crear tabla 'usuarios'
CREATE TABLE IF NOT EXISTS usuarios (
    id INT AUTO_INCREMENT PRIMARY KEY,
    nombre VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    creado_en TIMESTAMP DEFAULT CURRENT_TIMESTAMP
) ENGINE=INNODB;

-- Crear tabla 'productos'
CREATE TABLE IF NOT EXISTS productos (
    id INT AUTO_INCREMENT PRIMARY KEY,
    nombre VARCHAR(255) NOT NULL,
    precio DECIMAL(10, 2) NOT NULL,
    creado_en TIMESTAMP DEFAULT CURRENT_TIMESTAMP
) ENGINE=INNODB;

-- Crear tabla 'compras'
CREATE TABLE IF NOT EXISTS compras (
    id INT AUTO_INCREMENT PRIMARY KEY,
    usuario_id INT,
    producto_id INT,
    cantidad INT,
    fecha TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (usuario_id) REFERENCES usuarios(id),
    FOREIGN KEY (producto_id) REFERENCES productos(id)
) ENGINE=INNODB;

-- Crear tabla 'detalles_compra' con clave primaria compuesta y campos nulables
CREATE TABLE IF NOT EXISTS detalles_compra (
    compra_id INT,
    producto_id INT,
    precio_compra DECIMAL(10, 2),
    comentario VARCHAR(255) NULL,
    PRIMARY KEY (compra_id, producto_id),
    FOREIGN KEY (compra_id) REFERENCES compras(id),
    FOREIGN KEY (producto_id) REFERENCES productos(id)
) ENGINE=INNODB;

CREATE TABLE IF NOT EXISTS data_types_1 (
    dt_bit BIT,
    dt_blog BLOB,
    dt_json JSON,
    dt_date DATE,
    dt_datetime DATETIME,
    dt_time TIME
) ENGINE=INNODB;

-- Insertar datos de ejemplo en 'usuarios'
INSERT INTO usuarios (nombre, email) VALUES
('Ana Perez', 'ana.perez@example.com'),
('Luis Martinez', 'luis.martinez@example.com'),
('Carlos Gomez', 'carlos.gomez@example.com');

-- Insertar datos de ejemplo en 'productos'
INSERT INTO productos (nombre, precio) VALUES
('Laptop', 1200.00),
('Teléfono', 800.00),
('Tablet', 600.00),
('Cargador', 20.00),
('Audífonos', 50.00);

-- Insertar datos de ejemplo en 'compras'
INSERT INTO compras (usuario_id, producto_id, cantidad) VALUES
(1, 1, 1),
(2, 3, 2),
(3, 2, 1),
(1, 4, 2),
(2, 5, 1);

-- Insertar datos de ejemplo en 'detalles_compra'
INSERT INTO detalles_compra (compra_id, producto_id, precio_compra, comentario) VALUES
(1, 1, 1200.00, 'Compra para trabajo'),
(2, 3, 600.00, NULL),
(3, 2, 800.00, 'Regalo'),
(4, 4, 20.00, 'Necesario'),
(5, 5, 50.00, NULL);
