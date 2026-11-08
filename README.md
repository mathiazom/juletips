# 🎁 juletips

wishlist with a tiny CMS

- **`app/`** is a Rust admin app. It lets you add, edit, reorder, mark as received and delete wishes. Data is stored in SQLite.
  Every change regenerates the public site as plain HTML and CSS.
- **`public`** is a stock nginx container that serves the generated site read-only.

## Development

```sh
docker compose up --build
```

Admin runs on http://localhost:3001 and the public site on http://localhost:3000.

Without Docker: `cd app && DB_PATH=juletips.db SITE_DIR=site cargo run`. Admin is then on http://localhost:8080 and the generated site is in `app/site/`.

## Production

Copy `docker-compose.prod.example.yml` to the server and put an HTTPS reverse proxy in front of it.

- Proxy the public host to `juletips-www:80` and the admin host to `juletips-app:8080`. The containers publish no ports, so the proxy must be able to reach them, for example by sharing a Docker network.
- Protect the admin host with auth at the proxy, for example basic auth or an IP allowlist. The app has no auth of its own, so never publish port 8080.
- Back up both Docker volumes: `juletips-db-data` holds the SQLite database and `juletips-site-data` holds the generated page and the uploaded images.
