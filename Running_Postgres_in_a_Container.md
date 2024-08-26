# References and Commands to Run Postgres in a Container for This Project

### Launch Postgres container with a named volume

```bash
docker run -d --rm -p 9999:5432 --name sea-sqlx -v seaSqlxVol:/var/lib/postgresql/data -e POSTGRES_PASSWORD=sea-sqlx postgres
```

### Execute psql as admin on Postgres container

```bash
docker exec -ti sea-sqlx psql -U postgres
```

### Create test database and test user at psql prompt

postgres=#

```bash
CREATE DATABASE testdb;
CREATE ROLE testuser WITH LOGIN PASSWORD 'testpassword';
GRANT ALL PRIVILEGES ON DATABASE testdb TO testuser;
```

Must connect to database before next step
postgres=#

```bash
\c testdb
```

postgres=#

```bash
GRANT ALL ON SCHEMA public TO testuser;
```

### Grant/remove superuser priviledges to test user at psql prompt if necessary

postgres=#

```bash
ALTER USER testuser WITH SUPERUSER;
```

```bash
ALTER USER testuser WITH NOSUPERUSER;
```

#### Execute psql with test user on Postgres container

```bash
docker exec -ti sea-sqlx psql postgres://testuser:testpassword@localhost/testdb
```

### Once in the testdb psql prompt

#### Add the databases

Paste DDL/DML commands on command line. See examples in
https://github.com/eliben/code-for-blog/tree/master/2021/go-postgresql/migrations.

#### Back up database

```bash
docker exec -u postgres sea-sqlx pg_dump -Cc | xz > sea-sqlx-$(date -u +%Y-%m-%d).sql.xz
```

#### Restore database from backup (replace date below as needed)

```bash
xz -dc sea-sqlx-2022-02-17.sql.xz | docker exec -i -u postgres sea-sqlx psql –set ON_ERROR_STOP=on –single-transaction
```

### Other ways to execute psql

#### Execute psql with test user on host

```bash
psql postgres://testuser:testpassword@localhost:9999/testdb
```

### Execute psql with test user on client container

```bash
docker run -it --rm --network host --name psql postgres psql postgresql://testuser:testpassword@localhost:9999/testdb
```

or

```bash
docker run -it --rm --network host --name psql jbergknoff/postgresql-client postgresql://testuser:testpassword@localhost:9999/testdb
```

### References

- Postgres Docker image
  https://hub.docker.com/_/postgres
  https://github.com/docker-library/docs/blob/master/postgres/README.md
- Accessing PostgreSQL databases in Go
  https://eli.thegreenplace.net/2021/accessing-postgresql-databases-in-go/
  https://github.com/eliben/code-for-blog/tree/master/2021/go-postgresql
- Connecting to Postgresql in a docker container from outside
  https://stackoverflow.com/questions/37694987/connecting-to-postgresql-in-a-docker-container-from-outside
- Connect From Your Local Machine to a PostgreSQL Database in Docker
  https://betterprogramming.pub/connect-from-local-machine-to-postgresql-docker-container-f785f00461a7
- Accessing a PostgreSQL Database in a Docker Container (incl. backup/restore)
  https://inedo.com/support/kb/1145/accessing-a-postgresql-database-in-a-docker-container
- psql Docker image
  https://hub.docker.com/r/jbergknoff/postgresql-client
- Access host from a docker container
  https://dev.to/bufferings/access-host-from-a-docker-container-4099
  https://docs.docker.com/network/host/
  https://docs.docker.com/network/network-tutorial-host/
