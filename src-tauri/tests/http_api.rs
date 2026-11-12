//! Q2-13 (audit 10): the local HTTP API refuses web pages (CSRF, DNS
//! rebinding) and always needs a bearer token.

use axum::http::{HeaderMap, HeaderValue, StatusCode};
use voxxa_lib::{authorize, resolve_token};

const PORT: u16 = 7575;
const TOKEN: &str = "0123456789abcdef0123456789abcdef";

fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
    let mut h = HeaderMap::new();
    for (name, value) in pairs {
        h.insert(*name, HeaderValue::from_str(value).unwrap());
    }
    h
}

fn check(pairs: &[(&'static str, &str)]) -> Result<(), StatusCode> {
    authorize(&headers(pairs), PORT, TOKEN)
}

#[test]
fn a_blank_token_is_generated_and_a_short_one_refused() {
    let generated = resolve_token(None).unwrap();
    assert_eq!(generated.len(), 32);
    assert!(generated.chars().all(|c| c.is_ascii_hexdigit()));
    assert_ne!(generated, resolve_token(Some("  ".into())).unwrap());
    assert!(resolve_token(Some("short".into())).is_err());
    assert_eq!(resolve_token(Some(TOKEN.into())).unwrap(), TOKEN);
}

#[test]
fn the_token_is_required() {
    let host = ("host", "127.0.0.1:7575");
    let bearer = format!("Bearer {TOKEN}");
    assert_eq!(check(&[host]), Err(StatusCode::UNAUTHORIZED));
    assert_eq!(
        check(&[host, ("authorization", "Bearer wrong-token-wrong-token-wr")]),
        Err(StatusCode::UNAUTHORIZED)
    );
    assert_eq!(check(&[host, ("authorization", &bearer)]), Ok(()));
    let lower = format!("bearer {TOKEN}");
    assert_eq!(
        check(&[("host", "localhost:7575"), ("authorization", &lower)]),
        Ok(())
    );
}

#[test]
fn browser_requests_are_forbidden_even_with_the_token() {
    let auth = format!("Bearer {TOKEN}");
    let auth = ("authorization", auth.as_str());
    // DNS rebinding: the page's own host name reaches 127.0.0.1.
    assert_eq!(
        check(&[("host", "evil.example:7575"), auth]),
        Err(StatusCode::FORBIDDEN)
    );
    assert_eq!(
        check(&[("host", "127.0.0.1:9999"), auth]),
        Err(StatusCode::FORBIDDEN)
    );
    assert_eq!(check(&[auth]), Err(StatusCode::FORBIDDEN));
    // CSRF: a cross-site form or fetch always carries Origin.
    let host = ("host", "127.0.0.1:7575");
    assert_eq!(
        check(&[host, ("origin", "https://evil.example"), auth]),
        Err(StatusCode::FORBIDDEN)
    );
    assert_eq!(
        check(&[host, ("origin", "null"), auth]),
        Err(StatusCode::FORBIDDEN)
    );
}
