## About
- This is a web application to help competitive programmers to review the problems.

## For Developers

### Chores
- You can do miscellaneous work by `manage.sh`.

1. Enter DB
- run `./manage.sh enterdb`

### Testing
- Run `make test` to run the all backend test.
- There is a shell file to export dummy environmental varialbes for testing in `./tests/env.sh`

### Stop docker things
- Run `make down` to stop containers and to remove networks, volumes, and images.
