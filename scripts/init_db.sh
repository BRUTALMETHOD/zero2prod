#!/bin/bash

set -x
set -eo pipefail

# Commands required
if ! [ -x "$(command -v psql)" ];then
    echo >&2 "Error: psql not installed"
    exit 1
fi
if ! [ -x "$(command -v sqlx)" ];then
    echo >&2 "Error: sqlx not installed"
    exit 1
fi

# Check custom user or default to 'postgres'
DB_USER="${POSTGRES_USER:=postgres}"
# Check custom password or default 'password'
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
# Check custom db name or default newsletter
DB_NAME="${POSTGRES_DB:=newsletter}"
# Check if custom port or default '5432'
DB_PORT="${POSTGRES_POST:=5432}"
# Check if custom host or default 'localhost'
DB_HOST="${POSTGRES_HOST:=localhost}"

if [[ -z ${SKIP_DOCKER} ]];then
# Launch postgres using Docker
docker run \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -p "${DB_PORT}":5432 \
    -d postgres \
    postgres -N 1000
    # increased max conn for testing
fi

until psql -h "${DB_HOST}" -U "${DB_USER}" -p "{$DB_PORT}" -d "postgres" -c '\q';do
    >&2 echo "Postgres is still unavailable - sleeping"
    sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT}"

export DATABASE_URL=postgres://${DB_NAME}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated."


