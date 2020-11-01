CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  uid VARCHAR(255) UNIQUE,
  user_name VARCHAR(255)
);

CREATE TABLE reviews (
  id  integer PRIMARY KEY,
  uid VARCHAR(255),
  problem_name VARCHAR(100),
  problem_url VARCHAR(255),  
  comment VARCHAR(300)
);
