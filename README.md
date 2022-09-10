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

## Build & run

After performing the setup above, you can build and run by doing the following:

```
rustup default nightly   # recommended
cargo run
```

## API reference

### Uploading an image

Images can be uploaded via `POST /images` with the following JSON body:

```json
{
    "image_url": "<your image url>",
    "object_detection": true,
    "label": "<omit this field to autogenerate a label>"
}
```

Alternatively, you can instead upload an image by base64 encoding it:
```json
{
    "image_base64": "<your base64-encoded image>",
    ...
}
```

Note that including both `image_url` and `image_base64` in a request will result in a `400 Bad Request` error.

### Querying images

Query an image by id:
```
GET /images/{imageId}
```

Query all images:
```
GET /images
```

Query all images that contain all of the provided tags:
```
GET /images?objects=dog,cat
```

Bonus: query all images that contain one or more of the provided tags:
```
GET /images?some_objects=dog,cat
```

### Response format

`GET /images/{imageID}` and `POST /images` will return a single image. All other endpoints will return an array of images. Returned images have the following format:

```json
{
    "url": "<url you provided, or where a base64-encoded image was uploaded to>",
    "tags": [
        "tag1",
        "tag2",
        ...
    ],
    "label": "<a label you provided, or one that was generated for you>"
}
```
