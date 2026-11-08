mod html;

use axum::{
    Router,
    extract::{DefaultBodyLimit, Multipart, Path, Request, State},
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use image::{
    DynamicImage, ImageDecoder, ImageReader, codecs::jpeg::JpegEncoder, imageops::FilterType,
};
use rusqlite::{Connection, OptionalExtension, Row, params};
use std::{
    fs, io,
    io::Cursor,
    path::{Path as FsPath, PathBuf},
    sync::{Arc, Mutex},
};
use tower_http::services::ServeDir;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS wish(
  id INTEGER PRIMARY KEY, position INTEGER NOT NULL,
  title TEXT NOT NULL, url TEXT, subtitle TEXT, description TEXT,
  image TEXT, received INTEGER NOT NULL DEFAULT 0);
CREATE TABLE IF NOT EXISTS site(id INTEGER PRIMARY KEY CHECK(id = 1), last_modified INTEGER);
";
const MAX_IMAGE_WIDTH: u32 = 720;

type HttpErr = (StatusCode, String);
type Res = Result<Response, HttpErr>;

fn internal(e: impl std::fmt::Display) -> HttpErr {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

fn bad(e: impl std::fmt::Display) -> HttpErr {
    (StatusCode::BAD_REQUEST, e.to_string())
}

#[derive(Clone)]
struct App {
    db: Arc<Mutex<Connection>>,
    site: PathBuf,
}

pub struct Wish {
    pub id: i64,
    pub title: String,
    pub url: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub received: bool,
}

#[derive(Default)]
struct WishForm {
    title: String,
    url: Option<String>,
    subtitle: Option<String>,
    description: Option<String>,
    image: Option<Vec<u8>>,
    received: bool,
}

#[tokio::main]
async fn main() {
    let env = |key: &str, default: &str| std::env::var(key).unwrap_or_else(|_| default.into());
    let db_path = env("DB_PATH", "/data/db/juletips.db");
    let site = PathBuf::from(env("SITE_DIR", "/data/site"));

    let db = Connection::open(&db_path).expect("open database");
    db.execute_batch(SCHEMA).expect("create schema");
    migrate(&db).expect("migrate schema");
    publish(&db, &site, false).expect("initial publish");

    let app = router(App {
        db: Arc::new(Mutex::new(db)),
        site,
    });
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("bind :8080");
    println!("juletips admin listening on :8080");
    axum::serve(listener, app).await.expect("server");
}

fn router(app: App) -> Router {
    Router::new()
        .route("/", get(|s: State<App>| list(s, false)))
        .route("/mottatt", get(|s: State<App>| list(s, true)))
        .route(
            "/opprett",
            get(|| async { html::admin_form(None) }).post(create),
        )
        // axum matches the percent-encoded path, so "ø" must be spelled %C3%B8.
        .route("/%C3%B8nske/{id}", get(edit_form).post(update))
        .route("/%C3%B8nske/{id}/move/{dir}", post(move_wish))
        .route("/%C3%B8nske/{id}/delete", post(delete))
        .nest_service("/site", ServeDir::new(&app.site))
        .layer(middleware::from_fn(same_origin_posts))
        .layer(DefaultBodyLimit::max(20 * 1024 * 1024))
        .with_state(app)
}

/// Browsers send proxy basic-auth credentials on cross-site POSTs too, so this is the CSRF guard.
async fn same_origin_posts(req: Request, next: Next) -> Response {
    let same_origin = req
        .headers()
        .get("sec-fetch-site")
        .is_some_and(|v| v == "same-origin");
    if req.method() == Method::POST && !same_origin {
        return (StatusCode::FORBIDDEN, "Cross-site request blocked").into_response();
    }
    next.run(req).await
}

async fn list(State(app): State<App>, received: bool) -> Res {
    let db = app.db.lock().unwrap();
    let wishes = load(&db, received).map_err(internal)?;
    Ok(html::admin_list(&wishes, received).into_response())
}

async fn edit_form(State(app): State<App>, Path(id): Path<i64>) -> Res {
    let db = app.db.lock().unwrap();
    Ok(html::admin_form(Some(&find(&db, id)?)).into_response())
}

async fn create(State(app): State<App>, multipart: Multipart) -> Res {
    let form = read_form(multipart).await?;
    // ponytail: resize runs on the async worker; move to spawn_blocking if uploads ever stall other requests
    let image = resize(form.image.as_deref().ok_or_else(|| bad("Bilde mangler"))?)?;
    let db = app.db.lock().unwrap();
    let id = insert(&db, &form).map_err(internal)?;
    set_image(&db, &app.site, id, None, &image)?;
    done(&db, &app.site)
}

async fn update(State(app): State<App>, Path(id): Path<i64>, multipart: Multipart) -> Res {
    let form = read_form(multipart).await?;
    let image = form.image.as_deref().map(resize).transpose()?;
    let db = app.db.lock().unwrap();
    let old = find(&db, id)?;
    if image.is_none() && old.image.is_none() {
        return Err(bad("Bilde mangler"));
    }
    db.execute(
        "UPDATE wish SET title = ?1, url = ?2, subtitle = ?3, description = ?4, received = ?5 WHERE id = ?6",
        params![form.title, form.url, form.subtitle, form.description, form.received, id],
    )
    .map_err(internal)?;
    if let Some(bytes) = image {
        set_image(&db, &app.site, id, old.image.as_deref(), &bytes)?;
    }
    done(&db, &app.site)
}

async fn move_wish(State(app): State<App>, Path((id, dir)): Path<(i64, String)>) -> Res {
    let up = match dir.as_str() {
        "up" => true,
        "down" => false,
        _ => return Err(not_found()),
    };
    let mut db = app.db.lock().unwrap();
    find(&db, id)?;
    swap(&mut db, id, up).map_err(internal)?;
    done(&db, &app.site)
}

async fn delete(State(app): State<App>, Path(id): Path<i64>) -> Res {
    let db = app.db.lock().unwrap();
    let old = find(&db, id)?;
    db.execute("DELETE FROM wish WHERE id = ?1", [id])
        .map_err(internal)?;
    remove_image_file(&app.site, old.image.as_deref());
    done(&db, &app.site)
}

fn done(db: &Connection, site: &FsPath) -> Res {
    publish(db, site, true).map_err(internal)?;
    Ok(Redirect::to("/").into_response())
}

fn not_found() -> HttpErr {
    (StatusCode::NOT_FOUND, "Fant ikke ønsket".into())
}

async fn read_form(mut multipart: Multipart) -> Result<WishForm, HttpErr> {
    let mut form = WishForm::default();
    while let Some(field) = multipart.next_field().await.map_err(bad)? {
        let name = field.name().unwrap_or_default().to_owned();
        if name == "image" {
            let bytes = field.bytes().await.map_err(bad)?;
            form.image = (!bytes.is_empty()).then(|| bytes.to_vec());
            continue;
        }
        let text = field.text().await.map_err(bad)?.trim().to_owned();
        let value = (!text.is_empty()).then_some(text);
        match name.as_str() {
            "title" => form.title = value.unwrap_or_default(),
            "url" => form.url = value,
            "subtitle" => form.subtitle = value,
            "description" => form.description = value,
            "received" => form.received = true,
            _ => {}
        }
    }
    if form.title.is_empty() {
        return Err(bad("Tittel mangler"));
    }
    // Keeps javascript: and friends out of the public page's href.
    if let Some(url) = &form.url
        && !(url.starts_with("https://") || url.starts_with("http://"))
    {
        return Err(bad("Lenken må starte med https:// eller http://"));
    }
    Ok(form)
}

fn resize(bytes: &[u8]) -> Result<Vec<u8>, HttpErr> {
    let invalid = |e: image::ImageError| bad(format!("Ugyldig bilde: {e}"));
    let mut decoder = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(bad)?
        .into_decoder()
        .map_err(invalid)?;
    let orientation = decoder.orientation().map_err(invalid)?;
    let mut img = DynamicImage::from_decoder(decoder).map_err(invalid)?;
    img.apply_orientation(orientation);
    if img.width() > MAX_IMAGE_WIDTH {
        img = img.resize(MAX_IMAGE_WIDTH, u32::MAX, FilterType::Lanczos3);
    }
    let mut out = Vec::new();
    DynamicImage::ImageRgb8(img.into_rgb8())
        .write_with_encoder(JpegEncoder::new_with_quality(&mut out, 80))
        .map_err(internal)?;
    Ok(out)
}

fn set_image(
    db: &Connection,
    site: &FsPath,
    id: i64,
    old: Option<&str>,
    bytes: &[u8],
) -> Result<(), HttpErr> {
    // A fresh name per upload busts browser and proxy caches.
    let name = format!("{id}-{}.jpg", jiff::Timestamp::now().as_millisecond());
    let dir = site.join("images");
    fs::create_dir_all(&dir).map_err(internal)?;
    write_atomic(&dir, &name, bytes).map_err(internal)?;
    db.execute(
        "UPDATE wish SET image = ?1 WHERE id = ?2",
        params![name, id],
    )
    .map_err(internal)?;
    remove_image_file(site, old);
    Ok(())
}

fn remove_image_file(site: &FsPath, name: Option<&str>) {
    if let Some(name) = name {
        let _ = fs::remove_file(site.join("images").join(name));
    }
}

fn insert(db: &Connection, form: &WishForm) -> rusqlite::Result<i64> {
    db.execute(
        "INSERT INTO wish(position, title, url, subtitle, description)
         VALUES ((SELECT COALESCE(MAX(position), 0) + 1 FROM wish), ?1, ?2, ?3, ?4)",
        params![form.title, form.url, form.subtitle, form.description],
    )?;
    Ok(db.last_insert_rowid())
}

fn swap(db: &mut Connection, id: i64, up: bool) -> rusqlite::Result<()> {
    let tx = db.transaction()?;
    let pos: i64 = tx.query_row("SELECT position FROM wish WHERE id = ?1", [id], |r| {
        r.get(0)
    })?;
    let neighbour = if up {
        "SELECT id, position FROM wish WHERE received = 0 AND position < ?1 ORDER BY position DESC LIMIT 1"
    } else {
        "SELECT id, position FROM wish WHERE received = 0 AND position > ?1 ORDER BY position ASC LIMIT 1"
    };
    let other: Option<(i64, i64)> = tx
        .query_row(neighbour, [pos], |r| Ok((r.get(0)?, r.get(1)?)))
        .optional()?;
    if let Some((other_id, other_pos)) = other {
        tx.execute(
            "UPDATE wish SET position = ?1 WHERE id = ?2",
            params![other_pos, id],
        )?;
        tx.execute(
            "UPDATE wish SET position = ?1 WHERE id = ?2",
            params![pos, other_id],
        )?;
    }
    tx.commit()
}

fn find(db: &Connection, id: i64) -> Result<Wish, HttpErr> {
    db.query_row(
        "SELECT id, title, url, subtitle, description, image, received FROM wish WHERE id = ?1",
        [id],
        to_wish,
    )
    .optional()
    .map_err(internal)?
    .ok_or_else(not_found)
}

fn load(db: &Connection, received: bool) -> rusqlite::Result<Vec<Wish>> {
    let mut stmt = db.prepare(
        "SELECT id, title, url, subtitle, description, image, received FROM wish
         WHERE received = ?1 ORDER BY position",
    )?;
    stmt.query_map([received], to_wish)?.collect()
}

fn to_wish(r: &Row) -> rusqlite::Result<Wish> {
    Ok(Wish {
        id: r.get(0)?,
        title: r.get(1)?,
        url: r.get(2)?,
        subtitle: r.get(3)?,
        description: r.get(4)?,
        image: r.get(5)?,
        received: r.get(6)?,
    })
}

fn migrate(db: &Connection) -> rusqlite::Result<()> {
    let has_note: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM pragma_table_info('wish') WHERE name = 'note')",
        [],
        |r| r.get(0),
    )?;
    if has_note {
        db.execute_batch(
            "BEGIN;
             ALTER TABLE wish RENAME COLUMN description TO subtitle;
             ALTER TABLE wish RENAME COLUMN note TO description;
             COMMIT;",
        )?;
    }
    Ok(())
}

/// Regenerates the public site. `bump` marks it as changed for the "Sist oppdatert" footer.
fn publish(db: &Connection, site: &FsPath, bump: bool) -> Result<(), Box<dyn std::error::Error>> {
    if bump {
        db.execute(
            "INSERT INTO site(id, last_modified) VALUES (1, ?1)
             ON CONFLICT(id) DO UPDATE SET last_modified = excluded.last_modified",
            [jiff::Timestamp::now().as_second()],
        )?;
    }
    let last_modified: Option<i64> = db
        .query_row("SELECT last_modified FROM site WHERE id = 1", [], |r| {
            r.get(0)
        })
        .optional()?
        .flatten();
    let page = html::public_page(&load(db, false)?, last_modified);

    fs::create_dir_all(site.join("images"))?;
    write_atomic(site, "style.css", include_bytes!("../assets/style.css"))?;
    write_atomic(
        site,
        "julemathias.jpg",
        include_bytes!("../assets/julemathias.jpg"),
    )?;
    write_atomic(site, "favicon.svg", include_bytes!("../assets/favicon.svg"))?;
    write_atomic(site, "favicon.png", include_bytes!("../assets/favicon.png"))?;
    write_atomic(site, "index.html", page.into_string().as_bytes())?;
    Ok(())
}

/// nginx must never serve a half-written file.
fn write_atomic(dir: &FsPath, name: &str, bytes: &[u8]) -> io::Result<()> {
    let tmp = dir.join(format!(".{name}.tmp"));
    fs::write(&tmp, bytes)?;
    fs::rename(tmp, dir.join(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publishes_open_wishes_in_order() {
        let mut db = Connection::open_in_memory().unwrap();
        db.execute_batch(SCHEMA).unwrap();
        let add = |db: &Connection, title: &str| {
            insert(
                db,
                &WishForm {
                    title: title.into(),
                    ..Default::default()
                },
            )
            .unwrap()
        };
        let first = add(&db, "Første");
        add(&db, "Andre");
        let third = add(&db, "Tredje");

        swap(&mut db, third, true).unwrap();
        db.execute("UPDATE wish SET received = 1 WHERE id = ?1", [first])
            .unwrap();

        let site = std::env::temp_dir().join(format!("juletips-test-{}", std::process::id()));
        publish(&db, &site, true).unwrap();
        let html = fs::read_to_string(site.join("index.html")).unwrap();
        fs::remove_dir_all(&site).unwrap();

        assert!(html.contains("2 tips til årets julehandel"));
        assert!(!html.contains("Første"));
        let (tredje, andre) = (html.find("Tredje").unwrap(), html.find("Andre").unwrap());
        assert!(tredje < andre, "moved-up wish should come first");
        assert!(!html.contains("<script"));
        assert!(html.contains("Sist oppdatert"));
    }
}
