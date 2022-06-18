use anyhow::{anyhow, bail, Context, Result};
use async_std::{fs, prelude::*};
use dirs;
use futures::future;
use std::path::PathBuf;
use surf::RequestBuilder;

use crate::color::ColorScheme;

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

    /// Returns a provider for `Gogh-Co/Gogh`.
    pub fn gogh() -> Self {
        Provider::new("Gogh-Co", "Gogh", "themes", ".sh")
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
    pub async fn get(&self, name: &str) -> Result<ColorScheme> {
        let req = surf::get(&self.individual_url(name));
        let body = send_http_request(req)
            .await
            .with_context(|| format!("Failed to get color scheme raw content for {}", name))?;
        self.parse_color_scheme(&body)
    }

    /// Returns all color schemes in the provider.
    ///
    /// This function caches color schemes in the file system.
    pub async fn list(self) -> Result<Vec<(String, ColorScheme)>> {
        match self.read_color_schemes().await {
            Ok(color_schemes) => {
                if color_schemes.len() > 0 {
                    return Ok(color_schemes);
                }
            }
            _ => {}
        }

        // If there are no cached files, download them.
        self.download_all().await?;
        self.read_color_schemes().await
    }

    /// Download color scheme files into the cache directory.
    pub async fn download_all(&self) -> Result<()> {
        let repo_dir = self.repo_dir()?;

        eprintln!(
            "Downloading color schemes into {}",
            repo_dir.to_str().unwrap()
        );

        // Create the cache directory if it doesn't exist.
        fs::create_dir_all(&repo_dir)
            .await
            .context("Failed to create the cache directory")?;

        let list_req = surf::get(&self.list_url());
        let list_body = send_http_request(list_req)
            .await
            .context("Failed to download a color scheme list")?;
        let items = json::parse(&list_body).context("Failed to parse a color scheme list")?;

        // Download and save color scheme files.
        let mut futures = Vec::new();
        for item in items.members() {
            let filename = item["name"].as_str().unwrap();

            // Ignoring files starting with `_` for Gogh.
            if filename.starts_with('_') || !filename.ends_with(&self.extension) {
                continue;
            }

            let name = filename.replace(&self.extension, "");
            let req = surf::get(&self.individual_url(&name));
            futures.push(self.download_color_scheme(req, name));

            // Download files in batches.
            //
            // If this requests all files in parallel, the HTTP client (isahc) throws the
            // following error:
            //
            //   HTTP request error: ConnectFailed: failed to connect to the server
            //
            // isahc doesn't limit the number of connections per client by default, but
            // it exposes an API to limit it. However, surf doesn't expose the API.
            if futures.len() > 10 {
                future::try_join_all(futures).await?;
                futures = Vec::new();
            }
        }

        Ok(())
    }

    /// Read color schemes from the cache directory.
    async fn read_color_schemes(&self) -> Result<Vec<(String, ColorScheme)>> {
        let mut entries = fs::read_dir(self.repo_dir()?)
            .await
            .context("Failed to read the cache directory")?;

        // Collect futures and run them in parallel.
        let mut futures = Vec::new();
        while let Some(entry) = entries.next().await {
            let dir_entry = entry.context("Failed to read the cache directory entry")?;
            let filename = dir_entry.file_name().into_string().unwrap();

            let name = filename.replace(&self.extension, "").to_string();
            futures.push(self.read_color_scheme(name));
        }

        let color_schemes = future::try_join_all(futures).await?;

        Ok(color_schemes)
    }

    /// Reads a color scheme from the repository cache.
    async fn read_color_scheme(&self, name: String) -> Result<(String, ColorScheme)> {
        let file_path = self.individual_path(&name)?;

        let body = fs::read_to_string(file_path)
            .await
            .with_context(|| format!("Failed to read the color scheme file for {}", name))?;
        let color_scheme = self.parse_color_scheme(&body)?;

        Ok((name, color_scheme))
    }

    /// Downloads a color scheme file and save it in the cache directory.
    async fn download_color_scheme(&self, req: RequestBuilder, name: String) -> Result<()> {
        let body = send_http_request(req)
            .await
            .with_context(|| format!("Failed to download a color scheme file for {}", name))?;
        fs::write(self.individual_path(&name)?, body)
            .await
            .with_context(|| format!("Failed to write a color scheme file for {}", name))?;
        Ok(())
    }

    /// The repository cache directory.
    fn repo_dir(&self) -> Result<PathBuf> {
        let mut repo_dir = dirs::cache_dir().ok_or(anyhow!("There is no cache directory"))?;
        repo_dir.push("colortty");
        repo_dir.push("repositories");
        repo_dir.push(&self.user_name);
        repo_dir.push(&self.repo_name);
        Ok(repo_dir)
    }

    /// Returns the path for the given color scheme name.
    fn individual_path(&self, name: &str) -> Result<PathBuf> {
        let mut file_path = self.repo_dir()?;
        file_path.push(name);
        file_path.set_extension(&self.extension[1..]);
        Ok(file_path)
    }

    /// Returns the URL for a color scheme on GitHub.
    fn individual_url(&self, name: &str) -> String {
        format!(
            "https://raw.githubusercontent.com/{}/{}/master/{}/{}{}",
            self.user_name, self.repo_name, self.list_path, name, self.extension
        )
    }

    /// Returns the URL for the color scheme list on GitHub API.
    fn list_url(&self) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            self.user_name, self.repo_name, self.list_path
        )
    }

    /// Parses a color scheme data.
    fn parse_color_scheme(&self, body: &str) -> Result<ColorScheme> {
        // TODO: Think about better abstraction.
        if self.extension == ".itermcolors" {
            ColorScheme::from_iterm(&body)
        } else {
            ColorScheme::from_gogh(&body)
        }
    }
}

/// Sends an HTTP request and returns the body of the given request.
///
/// Fails when the URL responds with non-200 status code. Also sends
/// `colortty` as `User-Agent` header.
async fn send_http_request(req: RequestBuilder) -> Result<String> {
    let mut res = req
        .header("User-Agent", "colortty")
        .await
        // Surf::Error (http_types::Error) is not a std::error:Error.
        .map_err(|e| e.into_inner())
        .context("Failed to send an HTTP request")?;

    if !res.status().is_success() {
        bail!("Received non-success status code: {}", res.status());
    }

    let body = res
        .body_string()
        .await
        .map_err(|e| e.into_inner())
        .context("Failed to read HTTP response body")?;
    return Ok(body);
}
