use axum::extract::ConnectInfo;
use std::net::SocketAddr;
use tracing::Span;

pub fn make_span(req: &axum::http::Request<axum::body::Body>) -> Span {
    let peer = tracing::field::display(
        req.extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map_or("Unknown".to_string(), |x| x.ip().to_string()),
    );

    tracing::info_span!("http_log", uri = %req.uri(), method = %req.method(), peer = %peer, latency = tracing::field::Empty,
    status = tracing::field::Empty, user = tracing::field::Empty, role = tracing::field::Empty)
}

pub fn on_response(
    req: &axum::http::response::Response<axum::body::Body>,
    dur: std::time::Duration,
    span: &Span,
) {
    span.record(
        "latency",
        tracing::field::display(format!("{}ms", dur.as_millis())),
    );
    span.record("status", tracing::field::display(req.status()));
    tracing::info!("Response");
}
