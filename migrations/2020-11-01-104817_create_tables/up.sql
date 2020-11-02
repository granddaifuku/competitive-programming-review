CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  uid VARCHAR(255) UNIQUE NOT NULL,
  user_name VARCHAR(255) NOT NULL
);

CREATE TABLE reviews (
  id  integer PRIMARY KEY,
  uid VARCHAR(255) NOT NULL,
  problem_name VARCHAR(100) NOT NULL,
  problem_url VARCHAR(255) NOT NULL,
  comment VARCHAR(300)
);
