SHELL=/bin/bash

test:
	docker compose up -d --build db
	sleep 3 # Sleep for 3 seconds to ensure the db connection
	source ./tests/env.sh && cargo test -- --test-threads=1

down:
	docker compose down --rmi all --volumes --remove-orphans
