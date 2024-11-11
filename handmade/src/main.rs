use std::io;
use actix_web::{body::BoxBody, dev::ServiceResponse, get, http::{header::ContentType, StatusCode}, middleware::{ErrorHandlerResponse, ErrorHandlers}, web::{self, Data, Html, Redirect, ReqData}, App, HttpResponse, HttpServer, Responder};
use handlebars::{DirectorySourceOptions, Handlebars};
use serde_json::json;
use sessions::Session;

mod sessions;
mod session_middleware;

type HBS<'a> = Data<Handlebars<'a>>;

#[get("/foo")]
async fn foo(hb: HBS<'_>, session: ReqData<Session>) -> impl Responder {
    let session = session.into_inner();
    let flash = match &session.map {
        Some(map) => map.get("flash"),
        None => None,
    };

    let body = hb
        .render("foo", &json!({
            "flash": flash
        }))
        .unwrap();

    return Html::new(body);
}

#[get("/forward")]
async fn forward_session(session: ReqData<Session>) -> impl Responder {
    let session = session.into_inner();
    sessions::Sessions::forward(session);
    return Redirect::new("/forward", "/foo");
}

#[get("/redirect/forward")]
async fn redirect_to_forward(session: ReqData<Session>) -> impl Responder {
    sessions::Sessions::store(
        &session.id(),
        "flash",
        serde_json::to_value("Flash message from forward redirect!".to_string()).unwrap()
    );

    return Redirect::new("/redirect/forward", "/forward");
}


#[get("/redirect")]
async fn redirect(session: ReqData<Session>) -> impl Responder {
    sessions::Sessions::store(
        &session.id(),
        "flash",
        serde_json::to_value("Flash message from redirect!".to_string()).unwrap()
    );

    return Redirect::new("/redirect", "/foo");
}

#[get("/")]
async fn index(hb: HBS<'_>) -> impl Responder {
    let body = hb.
        render("index", &json!({
            "title": "Home!"
        }))
        .unwrap();

    return Html::new(body);
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(
            "./www",
            DirectorySourceOptions::default(),
        )
        .unwrap();
    let handlebars_ref = web::Data::new(handlebars);

    HttpServer::new(move || {
        App::new()
            .wrap(error_handlers())
            .wrap(session_middleware::CheckSession)
            .app_data(handlebars_ref.clone())
            .service(index)
            .service(foo)
            .service(redirect)
            .service(redirect_to_forward)
            .service(forward_session)
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

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
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