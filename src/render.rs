use super::text::PlainDecorator;
use axum::{
    extract::{Path, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::Value;
use std::borrow::Cow;
use utoipa::IntoParams;

use crate::{
    locale_from_optional_code,
    templates::{self, TemplateError},
};

#[derive(Debug, thiserror::Error)]
pub(crate) enum EngineError {
    #[error("Unsupported or invalid language: {0}")]
    BadLanguageCode(Cow<'static, str>),
    #[error("Failed to render template: {0}")]
    Template(#[from] TemplateError),
    #[error("Failed to render MJML: {0}")]
    Render(#[from] mrml::prelude::render::Error),
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Failed to convert HTML to text: {0}")]
    FailedTextConversion(#[from] html2text::Error),
}

impl IntoResponse for EngineError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{self}: {self:?}");
        match self {
            EngineError::TemplateNotFound(_) => (StatusCode::NOT_FOUND, format!("{self}")),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("{self}")),
        }
        .into_response()
    }
}

/// Todo search query
#[derive(Deserialize, IntoParams)]
pub(crate) struct RenderQuery {
    /// Language to render the template with
    lang: Option<String>,
}

pub async fn render_html(
    template_id: String,
    params: Value,
    lang: crate::Locale,
) -> Result<(String, Option<String>), EngineError> {
    let template =
        templates::get(&template_id).ok_or(EngineError::TemplateNotFound(template_id))?;
    let root = template(params, lang)?;
    let opts = mrml::prelude::render::RenderOptions::default();
    let content = root.render(&opts)?;
    Ok((content, root.get_title()))
}

#[utoipa::path(
    get,
    path = "/templates/{template_id}/html",
    responses(
        (status = 200, description = "Template rendered successfully"),
        (status = NOT_FOUND, description = "Template was not found")
    ),
    params(
        ("template_id" = String, Path, description = "Template to render"),
        RenderQuery
    )
)]
pub async fn render_html_route_get(
    Path(template_id): Path<String>,
    Query(RenderQuery { lang }): Query<RenderQuery>,
) -> Result<Response, EngineError> {
    let lang = locale_from_optional_code(lang)?;

    let (content, _title) = render_html(template_id, Value::Null, lang).await?;

    Ok(([(header::CONTENT_TYPE, "text/html")], content).into_response())
}

#[utoipa::path(
    post,
    path = "/templates/{template_id}/html",
    responses(
        (status = 200, description = "Template rendered successfully"),
        (status = NOT_FOUND, description = "Template was not found")
    ),
    params(
        ("template_id" = String, Path, description = "Template to render"),
        RenderQuery
    ),
    request_body = Value
)]
pub async fn render_html_route_post(
    Path(template_id): Path<String>,
    Query(RenderQuery { lang }): Query<RenderQuery>,
    Json(body): Json<Value>,
) -> Result<Response, EngineError> {
    let lang = locale_from_optional_code(lang)?;
    let (content, _title) = render_html(template_id, body, lang).await?;

    Ok(([(header::CONTENT_TYPE, "text/html")], content).into_response())
}

pub async fn render_text(html: &str) -> Result<String, EngineError> {
    let config = html2text::config::with_decorator(PlainDecorator::new())
        .no_table_borders()
        .allow_width_overflow()
        .no_link_wrapping()
        .raw_mode(true);
    let dom = config.parse_html(html.as_bytes())?;
    let tree = config.dom_to_render_tree(&dom)?;
    let text = config.render_to_string(tree, 50)?;

    Ok(text)
}

#[utoipa::path(
    get,
    path = "/templates/{template_id}/text",
    responses(
        (status = 200, description = "Template rendered successfully"),
        (status = NOT_FOUND, description = "Template was not found")
    ),
    params(
        ("template_id" = String, Path, description = "Template to render"),
        RenderQuery
    )
)]
pub async fn render_text_route_get(
    Path(template_id): Path<String>,
    Query(RenderQuery { lang }): Query<RenderQuery>,
) -> Result<Response, EngineError> {
    let lang = locale_from_optional_code(lang)?;
    let (html, _title) = render_html(template_id, Value::Null, lang).await?;
    let content = render_text(&html).await?;

    Ok((
        [(header::CONTENT_TYPE, "text/plain; charset=UTF-8")],
        content,
    )
        .into_response())
}

#[utoipa::path(
    post,
    path = "/templates/{template_id}/text",
    responses(
        (status = 200, description = "Template rendered successfully"),
        (status = NOT_FOUND, description = "Template was not found")
    ),
    params(
        ("template_id" = String, Path, description = "Template to render"),
        RenderQuery
    ),
    request_body = Value
)]
pub async fn render_text_route_post(
    Path(template_id): Path<String>,
    Query(RenderQuery { lang }): Query<RenderQuery>,
    Json(body): Json<Value>,
) -> Result<Response, EngineError> {
    let lang = locale_from_optional_code(lang)?;
    let (html, _title) = render_html(template_id, body, lang).await?;
    let content = render_text(&html).await?;

    Ok((
        [(header::CONTENT_TYPE, "text/plain; charset=UTF-8")],
        content,
    )
        .into_response())
}

#[cfg(test)]
mod test {
    use expect_test::expect_file;
    use serde_json::{json, Value};

    use crate::Locale;

    #[tokio::test]
    async fn basic_template_html() {
        let (res, _) = super::render_html("basic".to_string(), Value::Null, Locale::default())
            .await
            .unwrap();
        let expected = expect_file!["../fixtures/basic.html"];
        expected.assert_eq(&res);
    }

    #[tokio::test]
    async fn subscription_template_html() {
        let (res, _) = super::render_html(
            "subscription".to_string(),
            json!({
                "to_name": "Jade",
                "subscription_settings_url": "https://example.com/prefs",
                "edit_subscriptions_url": "https://example.com/subscribed",
                "edits": {
                  "artist": [
                    {
                      "entity_name": "Nathan",
                      "entity_comment": "English EDM artist",
                      "entity_url": "https://musicbrainz.org/artist/8d8d8a80-f74f-4f21-a44c-518cd6944ed2/edits",
                      "open_size": 0,
                      "applied_size": 1
                    },
                    {
                      "entity_name": "Example Artist",
                      "entity_url": "https://musicbrainz.org/artist/8d8d8a80-f74f-4f21-a44c-518cd6944ed2/edits",
                      "open_size": 0,
                      "applied_size": 1
                    }
                  ]
                }
              }),
            Locale::default(),
        )
        .await
        .unwrap();
        let expected = expect_file!["../fixtures/subscription.html"];
        expected.assert_eq(&res);
    }

    #[tokio::test]
    async fn basic_template_text() {
        let (html, _) = super::render_html("basic".to_string(), Value::Null, Locale::default())
            .await
            .unwrap();
        let res: String = super::render_text(&html).await.unwrap();
        let expected = expect_file!["../fixtures/basic.txt"];
        expected.assert_eq(&res);
    }

    #[tokio::test]
    async fn subscription_template_text() {
        let (html, _) = super::render_html(
            "subscription".to_string(),
            json!({
                "to_name": "Jade",
                "subscription_settings_url": "https://example.com/prefs",
                "edit_subscriptions_url": "https://example.com/subscribed",
                "edits": {
                  "artist": [
                    {
                      "entity_name": "Nathan",
                      "entity_comment": "English EDM artist",
                      "entity_url": "https://musicbrainz.org/artist/8d8d8a80-f74f-4f21-a44c-518cd6944ed2/edits",
                      "open_size": 0,
                      "applied_size": 1
                    },
                    {
                      "entity_name": "Example Artist",
                      "entity_url": "https://musicbrainz.org/artist/8d8d8a80-f74f-4f21-a44c-518cd6944ed2/edits",
                      "open_size": 0,
                      "applied_size": 1
                    }
                  ]
                }
              }),
            Locale::default(),
        )
        .await
        .unwrap();
        let res = super::render_text(&html).await.unwrap();
        let expected = expect_file!["../fixtures/subscription.txt"];
        expected.assert_eq(&res);
    }
}
