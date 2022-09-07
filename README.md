# Image API

## Setup
Make sure to set the `IMAGGA_API_KEY`, `IMAGGA_API_SECRET`, and `DATABASE_URL` environmental variables first.

The `DATABASE_URL` environmental variable should look something like:
```
postgres://username:password@localhost/image-api
```
Be sure to create the `image-api` database in Postgresql first so the migrations can run properly:
```sql
CREATE DATABASE image-api;
```
or using the command:
```sh
createdb image-api
```