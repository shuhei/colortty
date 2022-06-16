use crate::color::ColorScheme;
use crate::error::{ErrorKind, Result};
use async_std::{fs, prelude::*};
use dirs;
use failure::ResultExt;
use futures::future;
use std::path::PathBuf;
use surf::{middleware::HttpClient, Request};

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
        let body = http_get(req).await?;
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
            .context(ErrorKind::CreateDirAll)?;

        let list_req = surf::get(&self.list_url());
        let list_body = http_get(list_req).await?;
        let items = json::parse(&list_body).context(ErrorKind::ParseJson)?;

        // Download and save color scheme files.
        let mut futures = Vec::new();
        let client = surf::Client::new();
        for item in items.members() {
            let filename = item["name"].as_str().unwrap();

            // Ignoring files starting with `_` for Gogh.
            if filename.starts_with('_') || !filename.ends_with(&self.extension) {
                continue;
            }

            let name = filename.replace(&self.extension, "");
            let req = client.get(&self.individual_url(&name));
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
            .context(ErrorKind::ReadDir)?;

        // Collect futures and run them in parallel.
        let mut futures = Vec::new();
        while let Some(entry) = entries.next().await {
            let dir_entry = entry.context(ErrorKind::ReadDirEntry)?;
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
            .context(ErrorKind::ReadFile)?;
        let color_scheme = self.parse_color_scheme(&body)?;

        Ok((name, color_scheme))
    }

    // TODO: Pass `Client` instead of `Request`. However, the ownership rule blocks it...
    /// Downloads a color scheme file and save it in the cache directory.
    async fn download_color_scheme<C: HttpClient>(
        &self,
        req: Request<C>,
        name: String,
    ) -> Result<()> {
        let body = http_get(req).await?;
        fs::write(self.individual_path(&name)?, body)
            .await
            .context(ErrorKind::WriteFile)?;
        Ok(())
    }

    /// The repository cache directory.
    fn repo_dir(&self) -> Result<PathBuf> {
        let mut repo_dir = dirs::cache_dir().ok_or(ErrorKind::NoCacheDir)?;
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

/// Returns the body of the given request.
///
/// Fails when the URL responds with non-200 status code. Sends `colortty` as `User-Agent` header
async fn http_get<C: HttpClient>(req: Request<C>) -> Result<String> {
    let mut res = req
        .set_header("User-Agent", "colortty")
        .await
        .map_err(|e| {
            println!("HTTP request error: {}", e);
            ErrorKind::HttpGet
        })?;

    if !res.status().is_success() {
        println!("HTTP status code: {}", res.status());
        return Err(ErrorKind::HttpGet.into());
    }

    // TODO: Propagate information from the original error.
    let body = res.body_string().await.map_err(|_| ErrorKind::HttpGet)?;
    Ok(body)
}
