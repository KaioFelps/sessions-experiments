use std::future::{ready, Ready};

use actix_web::{
    cookie::{time::Duration, Cookie}, dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage
};
use futures_util::future::LocalBoxFuture;

use crate::sessions::{self, SESSION_COOKIE};

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct CheckSession;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for CheckSession
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckSessionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CheckSessionMiddleware { service }))
    }
}

pub struct CheckSessionMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CheckSessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let session_id_cookie = req.cookie(SESSION_COOKIE);
        
        let session_id = match session_id_cookie {
            Some(cookie) => cookie.value().to_string(),
            None => sessions::Sessions::new_session(),
        };
        
        req.extensions_mut().insert(sessions::Sessions::get(&session_id));

        let fut: <S as Service<ServiceRequest>>::Future = self.service.call(req);

        Box::pin(async move {
            let mut res: ServiceResponse<B> = fut.await?;
            let response = res.response_mut();
            
            let cookie = Cookie::build(
                sessions::SESSION_COOKIE,
                &session_id
            )
                .same_site(actix_web::cookie::SameSite::Strict)
                .path("/")
                .secure(false)
                .http_only(true)
                .max_age(Duration::days(1))
                .finish();

            let cookie_has_been_set = response.add_cookie(&cookie);
            if cookie_has_been_set.is_err() {
                println!("{}", cookie_has_been_set.unwrap_err());
            }
            
            return Ok(res);
        })
    }
}
