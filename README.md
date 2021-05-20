# Mountain Bikers Club

## Prerequisites

What things you need to install the software and how to install them.

- To simulate S3, use [minio](https://min.io).

### Postgres

- Run `CREATE EXTENSION fuzzystrmatch` to use `METAPHONE()`

## Deployment

### Environment variables

During development, you can create a `.env` file on the root.

```ini
ACTIX_ADDRESS=0.0.0.0
ACTIX_PORT=8080
# openssl rand -base64 32
DATABASE_URL=postgresql://
SECRET_KEY=
DB_PASSWORD_SALT=
TOKEN_SECRET=
EMAIL_SMTP_SERVER=
EMAIL_SMTP_USERNAME=
EMAIL_SMTP_PASSWORD=

AWS_ACCESS_KEY_ID=
AWS_SECRET_ACCESS_KEY=
AWS_S3_BUCKET_NAME=
# AWS_REGION=
AWS_S3_ENDPOINT=
```

## Contribution
Please make sure to read the [Contributing Guide](CONTRIBUTING.md) before making a pull request.

Thank you to all [the people who already contributed to Mountain Bikers Club](https://github.com/cedeber/mountain-bikers-club/graphs/contributors)!

### Vocabulary

- *user*: a current logged-in user or someone that tries to do something
- *member*: the current member page, member trip...

## License
[AGPL-3.0](LICENSE)

Copyright (c) 2018-2021, Cédric Eberhardt
