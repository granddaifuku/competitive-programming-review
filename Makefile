test:
	docker compose up -d --build db
	DATABASE_URL="postgres://postgres:password@localhost:5432/test" cargo test

down:
	docker compose down --rmi all
