use crate::Wish;
use jiff::{Timestamp, Zoned, tz::TimeZone};
use maud::{DOCTYPE, Markup, html};

const MONTHS: [&str; 12] = [
    "januar",
    "februar",
    "mars",
    "april",
    "mai",
    "juni",
    "juli",
    "august",
    "september",
    "oktober",
    "november",
    "desember",
];

fn now_in_oslo() -> Zoned {
    Timestamp::now().to_zoned(oslo())
}

fn oslo() -> TimeZone {
    TimeZone::get("Europe/Oslo").expect("tzdb is bundled")
}

/// "8. november", with the year only when it isn't the current one.
fn norwegian_date(ts: i64, now: &Zoned) -> String {
    let Ok(ts) = Timestamp::from_second(ts) else {
        return String::new();
    };
    let d = ts.to_zoned(oslo());
    let month = MONTHS[d.month() as usize - 1];
    if d.year() == now.year() {
        format!("{}. {month}", d.day())
    } else {
        format!("{}. {month} {}", d.day(), d.year())
    }
}

/// Static, JS-free page. Relative URLs so it also works under the admin's /site/ preview.
pub fn public_page(wishes: &[Wish], last_modified: Option<i64>) -> Markup {
    let now = now_in_oslo();
    html! {
        (DOCTYPE)
        html lang="nb" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width";
                link rel="icon" type="image/svg+xml" href="favicon.svg";
                link rel="icon" type="image/png" href="favicon.png";
                link rel="stylesheet" href="style.css";
                title { "Julegaveønsker Mathias " (now.year()) }
            }
            body {
                main {
                    header {
                        img.avatar src="julemathias.jpg" alt="Mathias" width="360" height="360";
                        h1 { (wishes.len()) " tips til årets julehandel" }
                    }
                    section {
                        @for (i, wish) in wishes.iter().enumerate() {
                            hr;
                            article {
                                @if let Some(image) = &wish.image {
                                    a href=[wish.url.as_deref()] {
                                        img src={ "images/" (image) } alt=(wish.title) loading="lazy";
                                    }
                                }
                                a href=[wish.url.as_deref()] {
                                    h2 { (i + 1) ". " (wish.title) }
                                }
                                @if let Some(subtitle) = &wish.subtitle {
                                    p.subtitle { (subtitle) }
                                }
                                @if let Some(description) = &wish.description {
                                    p.description { (description) }
                                }
                            }
                        }
                        hr;
                    }
                    footer {
                        @if let Some(ts) = last_modified {
                            p.updated { "Sist oppdatert " (norwegian_date(ts, &now)) }
                        }
                        p { "❤️" }
                    }
                }
            }
        }
    }
}

const ADMIN_CSS: &str = "
body { font-family: system-ui, sans-serif; max-width: 48rem; margin: 2rem auto; padding: 0 1rem; color: #1f2937; }
table { width: 100%; border-collapse: collapse; }
td { padding: .5rem; border-bottom: 1px solid #e5e7eb; vertical-align: middle; }
/* Saves and moves redirect to the row's #anchor; keep it mid-screen and mark it. */
tr { scroll-margin-top: 40vh; }
tr:target { animation: flash 2s ease-out; }
@keyframes flash { from { background: #fefce8; } }
td.actions { white-space: nowrap; text-align: right; }
/* max-width: 0 lets a table cell shrink below its content so the ellipsis kicks in. */
td.title { width: 100%; max-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
form.inline { display: inline; }
td.actions button { min-width: 2.75rem; min-height: 2.75rem; font-size: 1.25rem; }
td.actions form + form { margin-left: .5rem; }
img.thumb { width: 3rem; height: 3rem; object-fit: cover; border-radius: .25rem; }
label { display: block; margin-top: 1rem; font-weight: 600; }
input[type=text], input[type=url], input[type=number], textarea { width: 100%; box-sizing: border-box; padding: .4rem; font: inherit; font-weight: normal; display: block; margin-top: .4rem; }
input[type=number] { width: 6rem; }
.image-field { display: flex; flex-wrap: wrap; align-items: flex-start; gap: 1rem; margin-top: .4rem; font-weight: normal; }
/* Same size as the public page's article img. */
img.preview { width: 13rem; max-height: 20rem; object-fit: cover; border-radius: .375rem; border: 2px solid #e5e7eb; }
@media (min-width: 768px) { img.preview { width: 12rem; } }
textarea { min-height: 5rem; }
.row { display: flex; flex-wrap: wrap; gap: .5rem 1rem; align-items: center; justify-content: space-between; margin-top: 1.5rem; }
header { display: flex; flex-wrap: wrap; gap: 1.5rem 1rem; align-items: center; justify-content: space-between; margin: 1.5rem 0 3rem; }
header h1 { margin: 0; display: flex; align-items: center; gap: .5rem; }
img.avatar { display: block; width: 4rem; height: 4rem; border-radius: 50%; border: 3px solid #b91c1c; object-fit: cover; }
nav { display: flex; flex-wrap: wrap; gap: .75rem 1.5rem; }
button.danger { color: #b91c1c; }
.req { color: #b91c1c; font-weight: normal; }
/* Keeps the avatar painted across page loads; the rest swaps instantly. */
@view-transition { navigation: auto; }
img.avatar { view-transition-name: avatar; }
::view-transition-old(root) { display: none; }
::view-transition-new(root) { animation: none; }
";

fn admin_header(title: &str, suffix: Option<&str>, nav: Markup) -> Markup {
    html! {
        header {
            h1 {
                img.avatar src="/site/julemathias.jpg" alt=(title);
                @if let Some(suffix) = suffix { span aria-hidden="true" { (suffix) } }
            }
            nav { (nav) }
        }
    }
}

fn admin_layout(title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="nb" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width";
                link rel="icon" type="image/svg+xml" href="/site/favicon.svg";
                link rel="icon" type="image/png" href="/site/favicon.png";
                title { (title) " – Juletips admin" }
                style { (ADMIN_CSS) }
            }
            body { (body) }
        }
    }
}

pub fn admin_list(wishes: &[Wish], received: bool) -> Markup {
    let title = if received { "Mottatt" } else { "Ønsker" };
    admin_layout(
        title,
        html! {
            (admin_header(title, received.then_some("/mottatt"), html! {
                @if received { a href="/" { "← Ønsker" } } @else {
                    a href="/opprett" { "+ Nytt ønske" }
                    a href="/mottatt" { "Mottatt" }
                    a href="/site/index.html" { "Se offentlig side" }
                }
            }))
            table {
                @for (i, wish) in wishes.iter().enumerate() {
                    tr id={ "w" (wish.id) } {
                        td {
                            @if let Some(image) = &wish.image {
                                img.thumb src={ "/site/images/" (image) } alt="";
                            }
                        }
                        td.title title=(wish.title) { a href={ "/ønske/" (wish.id) } { (wish.title) } }
                        @if !received {
                            td.actions {
                                form.inline method="post" action={ "/ønske/" (wish.id) "/move/up" } {
                                    button title="Flytt opp" disabled[i == 0] { "↑" }
                                }
                                form.inline method="post" action={ "/ønske/" (wish.id) "/move/down" } {
                                    button title="Flytt ned" disabled[i + 1 == wishes.len()] { "↓" }
                                }
                            }
                        }
                    }
                }
            }
            @if wishes.is_empty() { p { @if received { "Ingenting mottatt ennå." } @else { "Ingen ønsker ennå." } } }
        },
    )
}

/// `place` is (1-based rank, highest allowed rank) among open wishes; `None` hides the field.
pub fn admin_form(wish: Option<&Wish>, place: Option<(usize, usize)>) -> Markup {
    let path = wish.map_or("/opprett".to_string(), |w| format!("/ønske/{}", w.id));
    let value = |f: fn(&Wish) -> Option<&str>| wish.and_then(f).unwrap_or_default().to_string();
    let title = if wish.is_some() {
        "Rediger ønske"
    } else {
        "Nytt ønske"
    };
    admin_layout(
        title,
        html! {
            (admin_header(title, Some(&path), html! {
                @if wish.is_some_and(|w| w.received) { a href="/mottatt" { "← Mottatt" } }
                @else { a href="/" { "← Ønsker" } }
            }))
            form method="post" action=(path) enctype="multipart/form-data" {
                label { "Tittel" " " span.req aria-hidden="true" { "*" } input type="text" name="title" required value=(value(|w| Some(w.title.as_str()))); }
                label { "Lenke" input type="url" name="url" placeholder="https://" value=(value(|w| w.url.as_deref())); }
                label { "Undertittel" input type="text" name="subtitle" value=(value(|w| w.subtitle.as_deref())); }
                label { "Beskrivelse" textarea name="description" { (value(|w| w.description.as_deref())) } }
                label {
                    "Bilde" " " span.req aria-hidden="true" { "*" }
                    div.image-field {
                        @if let Some(w) = wish {
                            @if let Some(image) = &w.image {
                                img.preview src={ "/site/images/" (image) } alt="";
                            }
                        }
                        input type="file" name="image" accept="image/jpeg,image/png,image/webp" required[!wish.is_some_and(|w| w.image.is_some())];
                    }
                }
                @if let Some((rank, total)) = place {
                    label { "Plassering (1–" (total) ")" input type="number" name="position" min="1" max=(total) required value=(rank); }
                }
                @if let Some(w) = wish {
                    label { input type="checkbox" name="received" checked[w.received]; " Mottatt (skjules fra offentlig side)" }
                }
                div.row { button type="submit" { "Lagre" } }
            }
            @if let Some(w) = wish {
                form method="post" action={ "/ønske/" (w.id) "/delete" } onsubmit="return confirm('Slette ønsket? Dette kan ikke angres.')" {
                    div.row { span {} button.danger type="submit" { "Slett ønsket" } }
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_norwegian_dates() {
        let now = "2026-11-08T12:00:00+01:00[Europe/Oslo]"
            .parse::<Zoned>()
            .unwrap();
        let this_year = "2026-03-01T00:30:00+01:00[Europe/Oslo]"
            .parse::<Zoned>()
            .unwrap();
        let last_year = "2025-12-24T23:30:00+01:00[Europe/Oslo]"
            .parse::<Zoned>()
            .unwrap();
        assert_eq!(
            norwegian_date(this_year.timestamp().as_second(), &now),
            "1. mars"
        );
        assert_eq!(
            norwegian_date(last_year.timestamp().as_second(), &now),
            "24. desember 2025"
        );
    }
}
