# 🎁 juletips

```sh
docker compose up --build
```

Admin runs on http://localhost:3001 and the public site on http://localhost:3000.

Without Docker: `cd app && DB_PATH=juletips.db SITE_DIR=site cargo run`. Admin is then on http://localhost:8080 and the generated site is in `app/site/`.