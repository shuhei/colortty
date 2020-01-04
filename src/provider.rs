use crate::color::ColorScheme;
use crate::error::{ErrorKind, Result};
use failure::ResultExt;

/// A GitHub repository that provides color schemes.
pub struct Provider {
    user_name: String,
    repo_name: String,
    list_path: String,
    extension: String,
}

impl Provider {
    /// Returns a provider for `mbadolato/iTerm2-Color-Schemes`.
    pub fn iterm() -> Self {
        Provider::new(
            "mbadolato",
            "iTerm2-Color-Schemes",
            "schemes",
            ".itermcolors",
        )
    }

    /// Returns a provider for `Mayccoll/Gogh`.
    pub fn gogh() -> Self {
        Provider::new("Mayccoll", "Gogh", "themes", ".sh")
    }

    /// Returns a provider instance.
    fn new(user_name: &str, repo_name: &str, list_path: &str, extension: &str) -> Self {
        Provider {
            user_name: user_name.to_string(),
            repo_name: repo_name.to_string(),
            list_path: list_path.to_string(),
            extension: extension.to_string(),
        }
    }

    /// Fetches the raw content of the color scheme for the given name.
    pub fn get(&self, name: &str) -> Result<ColorScheme> {
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/master/{}/{}{}",
            self.user_name, self.repo_name, self.list_path, name, self.extension
        );
        let body = http_get(&url)?;

        // TODO: Think about better abstraction.
        if self.extension == ".itermcolors" {
            ColorScheme::from_iterm(&body)
        } else {
            ColorScheme::from_gogh(&body)
        }
    }

    /// Returns names of all color schemes in the provider.
    pub fn list(&self) -> Result<Vec<String>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            self.user_name, self.repo_name, self.list_path
        );
        let body = http_get(&url)?;

        let items = json::parse(&body).context(ErrorKind::ParseJson)?;
        let names = items
            .members()
            .filter_map(|item| item["name"].as_str())
            // Ignoring files starting with `_` for Gogh.
            .filter(|filename| !filename.starts_with('_') && filename.ends_with(&self.extension))
            .map(|filename| filename.replace(&self.extension, "").to_string())
            .collect();

        Ok(names)
    }
}

/// Returns the body of the given URL.
fn http_get(url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let mut res = client
        .get(url)
        .header(reqwest::header::USER_AGENT, "colortty")
        .send()
        .context(ErrorKind::HttpGet)?;

    if !res.status().is_success() {
        return Err(ErrorKind::HttpGet.into());
    }

    let body = res.text().context(ErrorKind::HttpGet)?;
    Ok(body)
}
