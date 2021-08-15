SHELL=/bin/bash

build:
	docker compose up -d --build db
	sleep 3
	source ./tests/env.sh && cargo build

test:
	docker compose up -d --build db redis
	sleep 3 # Sleep for 3 seconds to ensure the db connection
	source ./tests/env.sh && cargo test -- --test-threads=1

down:
	docker compose down --rmi all --volumes --remove-orphans
