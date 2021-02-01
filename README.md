# Mountain Bikers Club

## Prerequisites

What things you need to install the software and how to install them.

- To simulate S3, use [minio](https://min.io).

## Deployment

### Environment variables

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
