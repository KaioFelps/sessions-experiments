use once_session::{OnceSession, OnceSessionExt};
use std::collections::HashMap;
use std::io;
use actix_session::{Session, SessionMiddleware};
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use actix_web::body::BoxBody;
use actix_web::cookie::Key;
use actix_web::dev::ServiceResponse;
use actix_web::http::{header::ContentType, StatusCode};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::web::{self, Data, Html, Redirect};
use handlebars::{DirectorySourceOptions, Handlebars};
use once_sessions_middleware::FlushOnceSessions;
use serde_json::json;
use stateful_session::StatefulSessions;

mod stateful_session;
mod once_sessions_middleware;
mod once_session;

type HBS<'a> = Data<Handlebars<'a>>;

#[get("/backwitherrors")]
async fn back_with_errors(session: Session, once_session: OnceSession) -> impl Responder {
    if let Err(err) = session.insert_errors(json!({"name": "Your name is too ugly!".to_string()})) {
        eprintln!("{}", err);
    };
    
    return Redirect::to(once_session.prev_req).using_status_code(StatusCode::SEE_OTHER);
}

#[get("/foo")]
async fn foo(hb: HBS<'_>, session: OnceSession) -> impl Responder {
    let mapped_session = session
        .map::<String, Option<HashMap<String, String>>>()
        .unwrap();

    return hb
        .render("foo", &mapped_session)
        .map(Html::new)
        .unwrap();
}

#[get("/forward")]
async fn forward_session(session: Session, once_session: OnceSession) -> impl Responder {
    let _ = session.forward_once_session::<String, String>(once_session);
    return Redirect::new("/forward", "/foo");
}

#[get("/redirect/forward")]
async fn redirect_to_forward(session: Session) -> impl Responder {
    let _ = session.insert_flash("Flash message from forward redirect!");
    return Redirect::new("/redirect/forward", "/forward");
}

#[get("/redirect")]
async fn redirect(session: Session) -> impl Responder {
    let _ = session.insert_flash("Flash message from redirect!");
    return Redirect::new("/redirect", "/foo");
}

#[get("/")]
async fn index(hb: HBS<'_>, once_session: OnceSession) -> impl Responder {
    let sessions = once_session
        .map::<String, HashMap<String, String>>()
        .unwrap();

    return hb
        .render("index", &json!({
            "title": "Home!",
            "errors": sessions.errors,
            "flash": sessions.flash
        }))
        .map(Html::new)
        .unwrap();
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory("./www", DirectorySourceOptions::default())
        .unwrap();

    let handlebars_ref = web::Data::new(handlebars);

    let secret = Key::generate();

    HttpServer::new(move || {
        App::new()
            .wrap(error_handlers())
            .wrap(FlushOnceSessions)
            .wrap(SessionMiddleware::new(StatefulSessions, secret.clone()))
            .app_data(handlebars_ref.clone())
            .service(index)
            .service(foo)
            .service(redirect)
            .service(redirect_to_forward)
            .service(forward_session)
            .service(back_with_errors)
            .service(actix_files::Files::new("/", "./public/").prefer_utf8(true))
    })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

fn error_handlers() -> ErrorHandlers<BoxBody> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

fn not_found<B>(res: ServiceResponse<B>) -> actix_web::Result<ErrorHandlerResponse<BoxBody>> {
    let response = get_error_response(&res, "Page not found");
    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        res.into_parts().0,
        response.map_into_left_body(),
    )))
}

fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> HttpResponse<BoxBody> {
    let request = res.request();
    let fallback = |err: &str| {
        HttpResponse::build(res.status())
            .content_type(ContentType::plaintext())
            .body(err.to_string())
    };

    let hb = request
        .app_data::<web::Data<Handlebars>>()
        .map(|t| t.get_ref());
    match hb {
        Some(hb) => {
            let data = json!({
                "error": error,
                "status_code": res.status().as_str()
            });
            let body = hb.render("error", &data);

            match body {
                Ok(body) => HttpResponse::build(res.status())
                    .content_type(ContentType::html())
                    .body(body),
                Err(_) => fallback(error),
            }
        }
        None => fallback(error),
    }
}