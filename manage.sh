#!/bin/bash
export PGPASSWORD=password

if [ $1 = "enterdb" ]; then
	psql -h 127.0.0.1 -p 5432 -U postgres test
fi
