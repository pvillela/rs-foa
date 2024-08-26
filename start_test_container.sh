docker run -d --rm -p 9999:5432 --name sea-sqlx -v seaSqlxVol:/var/lib/postgresql/data -e \
    POSTGRES_PASSWORD=sea-sqlx postgres
