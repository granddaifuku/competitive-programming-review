## About
- This is a web application to help competitive programmers review competitive programmig problems.

## For Developers

### Testing
- `make test` to run all the backend tests.
- There is a shell file to export dummy environmental variables for testing in `./tests/env.sh`

### Stop docker things
- Run `make down` to stop containers and to remove networks, volumes, and images.

### Chores
- You can do miscellaneous work by `manage.sh`.

1. Enter DB
- Run `./manage.sh enterdb`

2. Clear table
- Run `./manage.sh cleartable`
