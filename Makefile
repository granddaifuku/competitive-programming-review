SHELL=/bin/bash

test:
	docker compose up -d --build db
	source ./tests/env.sh && cargo test -- --test-threads=1

down:
	docker compose down --rmi all
