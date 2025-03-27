use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::{Value, json};

struct QueryResultBuild<T, M>
where
    T: Serialize,
    M: Metadata,
{
    data: Option<T>,
    metadata: Option<M>,
    headers: Option<HeaderMap>,
    statuscode: StatusCode,
}

pub trait Metadata
where
    Self: Serialize,
{
    fn get_value(&self) -> Value {
        json!(self)
    }
}

impl<T: Serialize, M: Metadata> IntoResponse for QueryResultBuild<T, M> {
    fn into_response(self) -> Response {
        let mut response = Response::builder();

        if let Some(headers) = self.headers.filter(|x| !x.is_empty()) {
            response = headers
                .into_iter()
                .filter_map(|(v, k)| v.map(|v| (v, k)))
                .fold(response, |builder, (k, v)| builder.header(k, v));
        }
        let mut body = serde_json::Map::new();

        if let Some(data) = self.data {
            body.insert("data".to_string(), json!(data));
        }

        if let Some(metadata) = self.metadata.and_then(|x| serde_json::to_value(x).ok()) {
            for (k, v) in metadata.as_object().unwrap() {
                body.insert(k.clone(), v.clone());
            }
        }

        response
            .status(self.statuscode)
            .body::<String>(json!(body).to_string())
            .unwrap_or_default()
            .into_response()
    }
}
