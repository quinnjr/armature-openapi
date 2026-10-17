//! Swagger UI integration

use crate::spec::OpenApiSpec;
use armature_core::{Error, HttpResponse};

/// Swagger UI configuration
#[derive(Debug, Clone)]
pub struct SwaggerConfig {
    /// Path where Swagger UI will be served (e.g., "/api-docs")
    pub path: String,
    /// Title for the Swagger UI page
    pub title: String,
    /// OpenAPI specification
    pub spec: OpenApiSpec,
}

impl SwaggerConfig {
    /// Create a new Swagger configuration
    pub fn new(path: impl Into<String>, spec: OpenApiSpec) -> Self {
        Self {
            path: path.into(),
            title: "API Documentation".to_string(),
            spec,
        }
    }

    /// Set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Get the OpenAPI spec as JSON
    pub fn spec_json(&self) -> Result<String, Error> {
        serde_json::to_string_pretty(&self.spec)
            .map_err(|e| Error::Internal(format!("Failed to serialize spec: {}", e)))
    }

    /// Get the OpenAPI spec as YAML
    pub fn spec_yaml(&self) -> Result<String, Error> {
        serde_norway::to_string(&self.spec)
            .map_err(|e| Error::Internal(format!("Failed to serialize spec: {}", e)))
    }
}

/// Pinned `swagger-ui-dist` release loaded by [`swagger_ui_response`].
pub const SWAGGER_UI_VERSION: &str = "5.33.1";

// Subresource Integrity hashes for the exact pinned files below, computed with
// `curl -sfL <url> | openssl dgst -sha384 -binary | base64` (and cross-checked
// against the same files on unpkg). Recompute them whenever
// `SWAGGER_UI_VERSION` changes.
const SWAGGER_UI_CSS_URL: &str =
    "https://cdn.jsdelivr.net/npm/swagger-ui-dist@5.33.1/swagger-ui.css";
const SWAGGER_UI_CSS_SRI: &str =
    "sha384-Ov4/wv3j2bmct8cDc5X4ngJZohVPzEmc6uDPH8WeljUxO5vtoykvMEfbu9Vh6RaW";
const SWAGGER_UI_BUNDLE_URL: &str =
    "https://cdn.jsdelivr.net/npm/swagger-ui-dist@5.33.1/swagger-ui-bundle.js";
const SWAGGER_UI_BUNDLE_SRI: &str =
    "sha384-ZPehFMQommnnuaZ4rpxgkgTT2DKFVp4hZC/7pLit+9Lek9T1YGSo23eHFbvNkXkw";
const SWAGGER_UI_PRESET_URL: &str =
    "https://cdn.jsdelivr.net/npm/swagger-ui-dist@5.33.1/swagger-ui-standalone-preset.js";
const SWAGGER_UI_PRESET_SRI: &str =
    "sha384-My2aDM4r2Mbm3ybHcubKm9O9U8FEjvF/O5nGvE9YK5dzqOTbWEKa79RPJ1krdMaF";

/// Escape text for an HTML text node / attribute value.
fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

/// Generate a Swagger UI HTML response.
///
/// The returned page loads a pinned `swagger-ui-dist` release
/// ([`SWAGGER_UI_VERSION`]) from the jsdelivr CDN (`cdn.jsdelivr.net`) with
/// Subresource Integrity (`integrity="sha384-..."` + `crossorigin`), so the
/// browser refuses a modified or substituted file. It is not self-contained:
/// it will fail to render offline or under a Content-Security-Policy that
/// blocks that origin.
pub fn swagger_ui_response(config: &SwaggerConfig) -> Result<HttpResponse, Error> {
    let spec_json = config.spec_json()?;
    // JSON is already valid JS object-literal syntax, so it can be injected
    // as-is. The only hazard is a literal `</script>` (or any `</`) inside a
    // string value breaking out of the enclosing <script> block, since serde
    // does not escape `/`. Escaping `</` to `<\/` is a JSON/JS-safe transform
    // that renders identically. We must NOT double backslashes: that would
    // corrupt data (e.g. a `\d+` pattern would render as `\\d+`).
    let spec_json_safe = spec_json.replace("</", "<\\/");

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <link rel="stylesheet" href="{css_url}" integrity="{css_sri}" crossorigin="anonymous">
    <style>
        body {{
            margin: 0;
            padding: 0;
        }}
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="{bundle_url}" integrity="{bundle_sri}" crossorigin="anonymous"></script>
    <script src="{preset_url}" integrity="{preset_sri}" crossorigin="anonymous"></script>
    <script>
        window.onload = function() {{
            const spec = {spec};
            SwaggerUIBundle({{
                spec: spec,
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout"
            }});
        }};
    </script>
</body>
</html>"#,
        title = escape_html(&config.title),
        spec = spec_json_safe,
        css_url = SWAGGER_UI_CSS_URL,
        css_sri = SWAGGER_UI_CSS_SRI,
        bundle_url = SWAGGER_UI_BUNDLE_URL,
        bundle_sri = SWAGGER_UI_BUNDLE_SRI,
        preset_url = SWAGGER_UI_PRESET_URL,
        preset_sri = SWAGGER_UI_PRESET_SRI,
    );

    Ok(HttpResponse::ok()
        .with_header("content-type".to_string(), "text/html".to_string())
        .with_body(html.into_bytes()))
}

/// Generate a response for the OpenAPI spec JSON endpoint
pub fn spec_json_response(config: &SwaggerConfig) -> Result<HttpResponse, Error> {
    let spec_json = config.spec_json()?;

    Ok(HttpResponse::ok()
        .with_header("content-type".to_string(), "application/json".to_string())
        .with_body(spec_json.into_bytes()))
}

/// Generate a response for the OpenAPI spec YAML endpoint
pub fn spec_yaml_response(config: &SwaggerConfig) -> Result<HttpResponse, Error> {
    let spec_yaml = config.spec_yaml()?;

    Ok(HttpResponse::ok()
        .with_header("content-type".to_string(), "application/x-yaml".to_string())
        .with_body(spec_yaml.into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::OpenApiBuilder;

    #[test]
    fn test_swagger_config() {
        let spec = OpenApiBuilder::new("Test API", "1.0.0").build();
        let config = SwaggerConfig::new("/api-docs", spec).with_title("Test Docs");

        assert_eq!(config.path, "/api-docs");
        assert_eq!(config.title, "Test Docs");
    }

    #[test]
    fn test_spec_json() {
        let spec = OpenApiBuilder::new("Test API", "1.0.0")
            .description("A test API")
            .build();
        let config = SwaggerConfig::new("/api-docs", spec);

        let json = config.spec_json().unwrap();
        assert!(json.contains("Test API"));
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn test_spec_yaml() {
        let spec = OpenApiBuilder::new("Test API", "1.0.0")
            .description("A test API")
            .build();
        let config = SwaggerConfig::new("/api-docs", spec);

        let yaml = config.spec_yaml().unwrap();
        assert!(yaml.contains("Test API"));
        assert!(yaml.contains("1.0.0"));
    }

    fn swagger_html(title: &str) -> String {
        let spec = OpenApiBuilder::new("Test API", "1.0.0").build();
        let config = SwaggerConfig::new("/api-docs", spec).with_title(title);
        let response = swagger_ui_response(&config).unwrap();
        String::from_utf8(response.body.to_vec()).unwrap()
    }

    #[test]
    fn test_swagger_ui_assets_pinned_with_sri() {
        let html = swagger_html("Docs");
        let tags: Vec<&str> = html
            .split('<')
            .filter(|t| t.starts_with("script src") || t.starts_with("link rel=\"stylesheet\""))
            .collect();
        assert_eq!(tags.len(), 3, "expected css + 2 scripts: {tags:?}");
        for tag in tags {
            assert!(tag.contains("integrity=\"sha384-"), "missing SRI: <{tag}");
            assert!(
                tag.contains("crossorigin=\"anonymous\""),
                "missing crossorigin: <{tag}"
            );
            assert!(
                tag.contains(&format!("swagger-ui-dist@{SWAGGER_UI_VERSION}/")),
                "asset must be pinned to the exact version: <{tag}"
            );
        }
    }

    #[test]
    fn test_swagger_ui_title_is_escaped() {
        let html = swagger_html("</title><script>alert(1)</script>");
        assert!(!html.contains("<script>alert(1)"));
        assert!(html.contains("&lt;/title&gt;"));
    }

    #[test]
    fn test_spec_yaml_round_trips() {
        let spec = OpenApiBuilder::new("Test API", "1.0.0").build();
        let yaml = SwaggerConfig::new("/api-docs", spec).spec_yaml().unwrap();
        let value: serde_json::Value = serde_norway::from_str(&yaml).unwrap();
        assert_eq!(value["info"]["title"], "Test API");
    }
}
