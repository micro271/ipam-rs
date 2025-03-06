use axum::http::{Response, StatusCode};

use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseError {
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    timestamp: Option<OffsetDateTime>,
}

impl ResponseError {
    pub fn new(
        r#type: String,
        title: String,
        status: StatusCode,
        detail: String,
        instance: String,
        offset: Option<UtcOffset>,
    ) -> Self {
        Self {
            r#type: Some(r#type),
            title: Some(title),
            status: Some(status.as_u16()),
            detail: Some(detail),
            instance: Some(instance),
            timestamp: Some(
                OffsetDateTime::now_utc()
                    .to_offset(offset.unwrap_or(UtcOffset::from_hms(-3, 0, 0).unwrap())),
            ),
        }
    }

    pub fn builder() -> Builder {
        Builder::default()
    }

    pub fn unauthorized(uri: &axum::http::Uri, detail: Option<String>) -> Self {
        Self {
            r#type: None,
            title: Some(StatusCode::UNAUTHORIZED.to_string()),
            status: Some(StatusCode::UNAUTHORIZED.as_u16()),
            detail,
            instance: Some(uri.to_string()),
            timestamp: Some(
                time::OffsetDateTime::now_utc()
                    .to_offset(time::UtcOffset::from_hms(-3, 0, 0).unwrap()),
            ),
        }
    }

    pub(self) fn create(
        Builder {
            r#type,
            title,
            status,
            detail,
            instance,
            offset,
        }: Builder,
    ) -> ResponseError {
        Self {
            r#type,
            title,
            status: status.or(Some(400)),
            detail,
            instance,
            timestamp: Some(OffsetDateTime::now_utc().to_offset(offset.unwrap_or(UtcOffset::UTC))),
        }
    }
}

impl From<Builder> for ResponseError {
    fn from(value: Builder) -> Self {
        ResponseError::create(value)
    }
}

impl axum::response::IntoResponse for ResponseError {
    fn into_response(self) -> axum::response::Response {
        Response::builder()
            .header(axum::http::header::CONTENT_TYPE, "application/problem+json")
            .status(StatusCode::from_u16(self.status.unwrap()).unwrap())
            .body(serde_json::json!(self).to_string())
            .unwrap_or_default()
            .into_response()
    }
}

#[derive(Debug, Default)]
pub struct Builder {
    r#type: Option<String>,
    title: Option<String>,
    status: Option<u16>,
    detail: Option<String>,
    instance: Option<String>,
    offset: Option<UtcOffset>,
}

impl Builder {
    pub fn r#type(mut self, r#type: String) -> Self {
        self.r#type = Some(r#type);
        self
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = Some(status.as_u16());
        self
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn detail(mut self, detail: String) -> Self {
        self.detail = Some(detail);
        self
    }

    pub fn instance(mut self, instance: String) -> Self {
        self.instance = Some(instance);
        self
    }

    pub fn offset(mut self, offset: time::UtcOffset) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn offset_hms(mut self, (hours, minutes, seconds): (i8, i8, i8)) -> Self {
        self.offset = UtcOffset::from_hms(hours, minutes, seconds).ok();
        self
    }

    pub fn build(self) -> ResponseError {
        ResponseError::create(self)
    }
}

impl From<ResponseError> for Builder {
    fn from(value: ResponseError) -> Self {
        let ResponseError {
            r#type,
            title,
            status,
            detail,
            instance,
            timestamp,
        } = value;
        Builder {
            r#type,
            title,
            status,
            detail,
            instance,
            offset: timestamp.map(|x| x.offset()),
        }
    }
}
