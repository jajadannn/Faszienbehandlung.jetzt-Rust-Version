use crate::config::AppConfig;

#[derive(Clone, Debug)]
pub struct SeoMeta {
    pub title: String,
    pub description: String,
    pub canonical_url: String,
    pub og_title: String,
    pub og_description: String,
    pub og_url: String,
    pub og_image: String,
    pub og_type: String,
}

impl SeoMeta {
    pub fn new(config: &AppConfig, path: &str, title: &str, description: &str) -> Self {
        let canonical_url = if path == "/" {
            config.base_url.clone()
        } else {
            format!("{}{}", config.base_url, path)
        };

        Self {
            title: title.to_string(),
            description: description.to_string(),
            canonical_url: canonical_url.clone(),
            og_title: title.to_string(),
            og_description: description.to_string(),
            og_url: canonical_url,
            og_image: format!("{}/static/og/og-image.svg", config.base_url),
            og_type: "website".to_string(),
        }
    }
}
