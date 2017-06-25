CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(40) NOT NULL,
    password VARCHAR(50) NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT 'f'
)
