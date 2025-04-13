use axum::{
    body::Body,
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
};
use serde::Serialize;
use serde_json::{Value, json};

use crate::database::repository::QueryResult;

pub struct ResponseQuery<T, M>
where
    T: Serialize,
    M: Metadata,
{
    data: Option<T>,
    metadata: Option<M>,
    headers: Option<HeaderMap>,
    statuscode: StatusCode,
}

impl<T: Serialize, M: Metadata> ResponseQuery<T, M> {
    pub fn new(
        data: Option<T>,
        metadata: Option<M>,
        headers: Option<HeaderMap>,
        statuscode: StatusCode,
    ) -> Self {
        Self {
            data,
            metadata,
            headers,
            statuscode,
        }
    }
}

pub trait Metadata
where
    Self: Serialize,
{
    fn get_value(self) -> Value
    where
        Self: Sized,
    {
        json!(self)
    }
}

impl Metadata for Value {
    fn get_value(self) -> Value
    where
        Self: Sized,
    {
        self
    }
}

impl<T: Serialize, M: Metadata> IntoResponse for ResponseQuery<T, M> {
    fn into_response(self) -> Response<Body> {
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

        if let Some(metadata) = self.metadata.map(Metadata::get_value) {
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

impl From<QueryResult> for ResponseQuery<(), Value> {
    fn from(value: QueryResult) -> Self {
        ResponseQuery::new(None, Some(json!(value)), None, StatusCode::OK)
    }
}
