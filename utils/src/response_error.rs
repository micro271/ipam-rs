use axum::http::{Response, StatusCode};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseError {
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,

    title: String,

    status: u16,

    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<String>,

    timestamp: OffsetDateTime,
}

impl std::default::Default for ResponseError {
    fn default() -> Self {
        Self {
            r#type: None,
            title: StatusCode::BAD_REQUEST.to_string(),
            status: StatusCode::BAD_REQUEST.as_u16(),
            detail: None,
            instance: None,
            timestamp: OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(-3, 0, 0).unwrap()),
        }
    }
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
            title: title,
            status: status.as_u16(),
            detail: Some(detail),
            instance: Some(instance),
            timestamp: OffsetDateTime::now_utc().to_offset(offset.unwrap_or(UtcOffset::UTC)),
        }
    }

    pub fn builder() -> ResponseErrorBuilder {
        ResponseErrorBuilder::default()
    }

    pub fn unauthorized(uri: Option<String>, detail: Option<String>) -> Self {
        Self {
            title: StatusCode::UNAUTHORIZED.to_string(),
            status: StatusCode::UNAUTHORIZED.as_u16(),
            detail,
            instance: uri,
            ..Default::default()
        }
    }

    pub(self) fn create(
        ResponseErrorBuilder {
            r#type,
            title,
            status,
            detail,
            instance,
            offset,
        }: ResponseErrorBuilder,
    ) -> ResponseError {
        Self {
            r#type,
            title: title.unwrap_or(StatusCode::BAD_REQUEST.to_string()),
            status: status.unwrap_or(StatusCode::BAD_REQUEST.as_u16()),
            detail,
            instance,
            timestamp: OffsetDateTime::now_utc().to_offset(offset.unwrap_or(UtcOffset::UTC)),
        }
    }
}

impl From<ResponseErrorBuilder> for ResponseError {
    fn from(value: ResponseErrorBuilder) -> Self {
        ResponseError::create(value)
    }
}

impl axum::response::IntoResponse for ResponseError {
    fn into_response(self) -> axum::response::Response {
        Response::builder()
            .header(axum::http::header::CONTENT_TYPE, "application/problem+json")
            .status(StatusCode::from_u16(self.status).unwrap())
            .body(serde_json::json!(self).to_string())
            .unwrap_or_default()
            .into_response()
    }
}

#[derive(Debug, Default)]
pub struct ResponseErrorBuilder {
    r#type: Option<String>,
    title: Option<String>,
    status: Option<u16>,
    detail: Option<String>,
    instance: Option<String>,
    offset: Option<UtcOffset>,
}

impl ResponseErrorBuilder {
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

    pub fn offset_hms(mut self, hours: i8, minutes: i8, seconds: i8) -> Self {
        self.offset = UtcOffset::from_hms(hours, minutes, seconds).ok();
        self
    }

    pub fn build(self) -> ResponseError {
        ResponseError::create(self)
    }
}

impl From<ResponseError> for ResponseErrorBuilder {
    fn from(value: ResponseError) -> Self {
        let ResponseError {
            r#type,
            title,
            status,
            detail,
            instance,
            timestamp,
        } = value;
        ResponseErrorBuilder {
            r#type,
            title: Some(title),
            status: Some(status),
            detail,
            instance,
            offset: Some(timestamp.offset()),
        }
    }
}
