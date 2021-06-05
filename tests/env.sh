#!/bin/bash

# This shell script exports the environmental variables used in tests.
export DATABASE_URL="postgres://postgres:password@localhost:5432/test"
export SMTP_USERNAME="dummy_username"
export SMTP_PASSWORD="dummy_password"
export MAILER="dummy_mailer"
