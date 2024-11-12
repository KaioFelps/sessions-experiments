use std::future::{ready, Ready};
use actix_session::SessionExt;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use actix_web::HttpMessage;
use futures_util::future::LocalBoxFuture;
use crate::flash::Flash;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct FlushOnceSessions;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for FlushOnceSessions
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = FlushOnceSessionsMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(FlushOnceSessionsMiddleware { service }))
    }
}

pub struct FlushOnceSessionsMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for FlushOnceSessionsMiddleware<S>
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
        let session = req.get_session();
        let once_session = session.flush_flash();
        req.extensions_mut().insert(once_session);

        let fut: <S as Service<ServiceRequest>>::Future = self.service.call(req);

        Box::pin(async move {
            let res: ServiceResponse<B> = fut.await?;
            return Ok(res);
        })
    }
}
