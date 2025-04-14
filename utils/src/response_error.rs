use axum::http::{Response, StatusCode};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset};

/// This type represents the standard response according to the rfc 7807
///
///      `Content-Type: application/json+problem`
///
/// fields:
///     type: A string uri that  identifies the problem type
///     title: A short summary of the problem as a string
///     status: A type number that represents the status code of the reqwest according to rfc 7231
///     detail: an string that providing specific details about the occurence of the problem
///     instance: type String that identified the specific occurence of the problem
///
/// The fields `title`, `status` and `timestamp` are required, as it have an default values
/// The fields `type`, `detail`, and `instance` aren't requirend and will be ignored if not provide
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
    /// The StatusCode as default is BAD_REQUEST
    /// The title is the StatusCode as String
    /// The timestamp is the current with an offset of -3 hours
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

    /// Obtain the ResponseErrorBuilder that allow create an ResponseError
    ///
    /// ```
    /// let builder = ResponseError::builer()
    ///     .title("This is an generic problem")
    ///     .detail("An specific detail")
    ///     .status(StatusCode::INTERNAL_SERVER_ERROR)
    ///     .offset_hms(-3,0,0)/* or .offset(time::UtcOffset::from_hms(-3,0.0))*/
    ///     .build() // <- ResponseError
    /// ```
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
}

impl From<ResponseErrorBuilder> for ResponseError {
    fn from(value: ResponseErrorBuilder) -> Self {
        let ResponseErrorBuilder {
            r#type,
            title,
            status,
            detail,
            instance,
            offset,
        } = value;

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

/// This is the builder for `ResponseError`, allowing you to create a `Response Error`
/// using the builer Pattern
///
/// This type imeplement the `Default` trait, so you will get a `ResponseErrorBuild`
/// with all fields set to `None`
///
/// You can then fill in the values by calling the corresponding builder methos.
///
/// ```
/// let builder = ResponseErrorBuilder::new()
///     .title("Title of the problem")
///     .instance("midom.com/my/api")
///     .detail("An specific detail of the problem")
///     .status(StatusCode::BAD_REQUEST)
///     .build()
/// ```
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
    pub fn new() -> Self {
        Self::default()
    }
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
        self.into()
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
