use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::models::RepoSummary;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScrapedGitHubData {
    pub username: String,
    pub github_id: u64,
    pub display_name: String,
    pub avatar_url: String,
    pub bio: String,
    pub company: Option<String>,
    pub github_url: String,
    pub email: Option<String>,
    pub top_repos: Vec<RepoSummary>,
    pub languages: Vec<String>,
}

#[derive(Clone)]
pub struct GitHubScraper {
    pub http: Client,
    pub api_base_url: String,
}

#[derive(Deserialize)]
struct GitHubUserDto {
    id: u64,
    login: String,
    name: Option<String>,
    avatar_url: Option<String>,
    bio: Option<String>,
    company: Option<String>,
    html_url: Option<String>,
    email: Option<String>,
}

#[derive(Deserialize)]
struct GitHubRepoDto {
    name: String,
    description: Option<String>,
    language: Option<String>,
    stargazers_count: Option<u32>,
    topics: Option<Vec<String>>,
    default_branch: Option<String>,
}

impl GitHubScraper {
    pub fn new(http: Client) -> Self {
        Self {
            http,
            api_base_url: "https://api.github.com".into(),
        }
    }

    pub async fn authenticated_identity(&self, token: &str) -> Result<(String, u64), ApiError> {
        let response = self
            .http
            .get(format!("{}/user", self.api_base_url))
            .bearer_auth(token)
            .header("User-Agent", "VECTOR-KubeCon-OS/1.0")
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(ApiError::Unauthorized(
                "GitHub could not verify your identity".into(),
            ));
        }
        let user: GitHubUserDto = response.json().await?;
        if user.id == 0
            || user.login.is_empty()
            || !user
                .login
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        {
            return Err(ApiError::Unauthorized(
                "GitHub returned an invalid identity".into(),
            ));
        }
        Ok((user.login, user.id))
    }

    /// Scrapes public GitHub profile & top pushed repositories using concurrent tokio tasks.
    /// Identity/profile errors are surfaced; authentication never uses seed identities.
    pub async fn scrape_github_user(
        &self,
        username: &str,
        oauth_token: Option<&str>,
    ) -> Result<ScrapedGitHubData, ApiError> {
        let user_url = format!("{}/users/{}", self.api_base_url, username);
        let repos_url = format!(
            "{}/users/{}/repos?sort=pushed&per_page=8",
            self.api_base_url, username
        );

        let mut req_user = self
            .http
            .get(&user_url)
            .header("User-Agent", "VECTOR-KubeCon-OS/1.0");
        let mut req_repos = self
            .http
            .get(&repos_url)
            .header("User-Agent", "VECTOR-KubeCon-OS/1.0");

        if let Some(token) = oauth_token {
            req_user = req_user.bearer_auth(token);
            req_repos = req_repos.bearer_auth(token);
        }

        let (user_resp, repos_resp) = tokio::join!(req_user.send(), req_repos.send());

        // Check for rate limit or errors; if hit, fallback to seed profiles!
        let user_ok = user_resp
            .as_ref()
            .map(|r| r.status().is_success())
            .unwrap_or(false);
        let repos_ok = repos_resp
            .as_ref()
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        if !user_ok || !repos_ok {
            return Err(ApiError::GitHubError(
                "Unable to fetch your GitHub profile. Please retry.".into(),
            ));
        }

        let user_dto: GitHubUserDto = user_resp?.json().await?;
        let repos_dto: Vec<GitHubRepoDto> = repos_resp?.json().await.unwrap_or_default();

        let mut top_repos = Vec::new();
        let mut lang_set = std::collections::BTreeSet::new();

        for repo in repos_dto.iter().take(5) {
            let lang = repo.language.clone().unwrap_or_else(|| "Rust".to_string());
            lang_set.insert(lang.clone());

            let readme_excerpt = self
                .fetch_readme_excerpt(
                    &user_dto.login,
                    &repo.name,
                    repo.default_branch.as_deref().unwrap_or("main"),
                    oauth_token,
                )
                .await
                .unwrap_or_else(|| repo.description.clone().unwrap_or_default());

            top_repos.push(RepoSummary {
                name: repo.name.clone(),
                description: repo.description.clone().unwrap_or_default(),
                language: lang,
                stars: repo.stargazers_count.unwrap_or(0),
                topics: repo.topics.clone().unwrap_or_default(),
                readme_excerpt,
            });
        }

        let languages: Vec<String> = lang_set.into_iter().collect();

        Ok(ScrapedGitHubData {
            username: user_dto.login.clone(),
            github_id: user_dto.id,
            display_name: user_dto.name.unwrap_or_else(|| user_dto.login.clone()),
            avatar_url: user_dto
                .avatar_url
                .unwrap_or_else(|| format!("https://github.com/{}.png", user_dto.login)),
            bio: user_dto.bio.unwrap_or_default(),
            company: user_dto.company,
            github_url: user_dto
                .html_url
                .unwrap_or_else(|| format!("https://github.com/{}", user_dto.login)),
            email: user_dto.email,
            top_repos,
            languages,
        })
    }

    async fn fetch_readme_excerpt(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        oauth_token: Option<&str>,
    ) -> Option<String> {
        let raw_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/README.md",
            owner, repo, branch
        );

        let mut req = self
            .http
            .get(&raw_url)
            .header("User-Agent", "VECTOR-KubeCon-OS/1.0");
        if let Some(token) = oauth_token {
            req = req.bearer_auth(token);
        }

        if let Ok(resp) = req.send().await {
            if resp.status().is_success() {
                if let Ok(text) = resp.text().await {
                    let chars: String = text.chars().take(1200).collect();
                    return Some(chars);
                }
            }
        }
        None
    }
}
