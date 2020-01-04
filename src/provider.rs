use crate::color::ColorScheme;
use crate::error::{ErrorKind, Result};
use async_std::{fs, prelude::*};
use dirs;
use failure::ResultExt;
use futures::future;
use git2::Repository;
use std::path::{Path, PathBuf};
use surf;

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
    pub async fn get(&self, name: &str) -> Result<ColorScheme> {
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/master/{}/{}{}",
            self.user_name, self.repo_name, self.list_path, name, self.extension
        );
        let body = http_get(&url).await?;
        self.parse_color_scheme(&body)
    }

    /// Returns all color schemes in the provider.
    ///
    /// This function caches color schemes in the file system.
    pub async fn list(&self) -> Result<Vec<(String, ColorScheme)>> {
        self.prepare_cache().await?;

        let mut entries = fs::read_dir(self.schemes_dir()?)
            .await
            .context(ErrorKind::ReadDir)?;

        // Collect futures and run them in parallel.
        let mut futures = Vec::new();
        while let Some(entry) = entries.next().await {
            let dir_entry = entry.context(ErrorKind::ReadDirEntry)?;
            let filename = dir_entry.file_name().into_string().unwrap();

            // Ignoring files starting with `_` for Gogh.
            if filename.starts_with('_') || !filename.ends_with(&self.extension) {
                continue;
            }

            let name = filename.replace(&self.extension, "").to_string();
            futures.push(self.read_color_scheme(name));
        }

        let color_schemes = future::try_join_all(futures).await?;

        Ok(color_schemes)
    }

    /// Caches the repository in the file system if the cache doesn't exist.
    async fn prepare_cache(&self) -> Result<()> {
        // Create the parent directory if it doesn't exist.
        fs::create_dir_all(self.parent_dir()?)
            .await
            .context(ErrorKind::CreateDirAll)?;

        let repo_dir = self.repo_dir()?;
        if !Path::new(&repo_dir).exists() {
            // TODO: The entire repository occupies ~100MB. Consider fetching only necessary files with HTTP.
            // Clone the repository.
            let repo_url = format!("https://github.com/{}/{}", self.user_name, self.repo_name);
            println!("Cloning {}", repo_url);
            Repository::clone(&repo_url, &repo_dir).context(ErrorKind::GitClone)?;
        }

        Ok(())
    }

    /// Reads a color scheme from the repository cache.
    async fn read_color_scheme(&self, name: String) -> Result<(String, ColorScheme)> {
        let mut file_path = self.schemes_dir()?;
        file_path.push(&name);
        file_path.set_extension(&self.extension[1..]);

        let body = fs::read_to_string(file_path)
            .await
            .context(ErrorKind::ReadFile)?;
        let color_scheme = self.parse_color_scheme(&body)?;

        Ok((name, color_scheme))
    }

    /// The parent directory to clone the repository cache into.
    fn parent_dir(&self) -> Result<PathBuf> {
        let mut parent_dir = dirs::cache_dir().ok_or(ErrorKind::NoCacheDir)?;
        parent_dir.push("colortty");
        parent_dir.push("repositories");
        parent_dir.push(&self.user_name);
        Ok(parent_dir)
    }

    /// The repository cache directory.
    fn repo_dir(&self) -> Result<PathBuf> {
        Ok(self.parent_dir()?.join(&self.repo_name))
    }

    /// The directory of all color schemes in the repository.
    fn schemes_dir(&self) -> Result<PathBuf> {
        Ok(self.repo_dir()?.join(&self.list_path))
    }

    fn parse_color_scheme(&self, body: &str) -> Result<ColorScheme> {
        // TODO: Think about better abstraction.
        if self.extension == ".itermcolors" {
            ColorScheme::from_iterm(&body)
        } else {
            ColorScheme::from_gogh(&body)
        }
    }
}

/// Returns the body of the given URL.
async fn http_get(url: &str) -> Result<String> {
    let mut res = surf::get(url)
        .set_header("User-Agent", "colortty")
        .await
        .map_err(|_| ErrorKind::HttpGet)?;

    if !res.status().is_success() {
        return Err(ErrorKind::HttpGet.into());
    }

    // TODO: Propagate information from the original error.
    let body = res.body_string().await.map_err(|_| ErrorKind::HttpGet)?;
    Ok(body)
}
