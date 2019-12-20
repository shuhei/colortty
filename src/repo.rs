use crate::error::{ErrorKind, Result};
use failure::ResultExt;

pub struct Repo {
    user_name: String,
    repo_name: String,
    list_path: String,
}

impl Repo {
    pub fn new(user_name: &str, repo_name: &str, list_path: &str) -> Self {
        Repo {
            user_name: user_name.to_string(),
            repo_name: repo_name.to_string(),
            list_path: list_path.to_string(),
        }
    }

    pub fn get(&self, filename: &str) -> Result<String> {
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/master/{}/{}",
            self.user_name, self.repo_name, self.list_path, filename
        );
        http_get(&url)
    }
}

pub fn http_get(url: &str) -> Result<String> {
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
