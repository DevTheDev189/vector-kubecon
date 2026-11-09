use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::ApiError;
use crate::models::{BuilderProfile, RepoSummary};

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
        Self { http }
    }

    /// Scrapes public GitHub profile & top pushed repositories using concurrent tokio tasks.
    /// Falls back gracefully to seed/judge_profiles.json if rate-limited or offline.
    pub async fn scrape_github_user(
        &self,
        username: &str,
        oauth_token: Option<&str>,
    ) -> Result<ScrapedGitHubData, ApiError> {
        let user_url = format!("https://api.github.com/users/{}", username);
        let repos_url = format!(
            "https://api.github.com/users/{}/repos?sort=pushed&per_page=8",
            username
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
            tracing::warn!(
                username = %username,
                "GitHub API rate limit or error; falling back to offline seed dataset"
            );
            return self.fallback_from_seed(username);
        }

        let user_dto: GitHubUserDto = user_resp.unwrap().json().await?;
        let repos_dto: Vec<GitHubRepoDto> = repos_resp.unwrap().json().await.unwrap_or_default();

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

    fn fallback_from_seed(&self, username: &str) -> Result<ScrapedGitHubData, ApiError> {
        let seed_paths = ["seed/judge_profiles.json", "../seed/judge_profiles.json"];
        let mut profiles: Vec<BuilderProfile> = Vec::new();

        for p in &seed_paths {
            if Path::new(p).exists() {
                if let Ok(content) = std::fs::read_to_string(p) {
                    if let Ok(parsed) = serde_json::from_str(&content) {
                        profiles = parsed;
                        break;
                    }
                }
            }
        }

        // Look for exact match or default to first
        let matched = profiles
            .into_iter()
            .find(|p| p.username.eq_ignore_ascii_case(username))
            .unwrap_or_else(|| BuilderProfile {
                username: username.to_string(),
                github_id: 123456,
                display_name: username.to_string(),
                avatar_url: format!("https://github.com/{}.png", username),
                bio: "Cloud-native builder attending KubeCon NA SLC.".to_string(),
                company: Some("Community".to_string()),
                github_url: format!("https://github.com/{}", username),
                linkedin_url: None,
                email: None,
                persona_title: "Cloud-Native Infrastructure Engineer".to_string(),
                dna_tags: vec![
                    "Kubernetes".to_string(),
                    "Rust".to_string(),
                    "eBPF".to_string(),
                    "OpenTelemetry".to_string(),
                ],
                top_languages: vec!["Rust".to_string(), "Go".to_string()],
                hallway_icebreaker: "Let's connect on modern cloud-native architectures!"
                    .to_string(),
                ideal_talk_query: "kubernetes eBPF rust opentelemetry".to_string(),
                top_repos: vec![],
                current_room_id: "ballroom-b".to_string(),
                coordinates: crate::models::GeoPoint {
                    lat: 40.76630,
                    lon: -111.89670,
                },
                sensory_prefs: crate::models::SensoryPreferences::default(),
                bookmarked_talk_ids: vec![],
                bumped_from_talk_id: None,
                pin_code: "1234".to_string(),
                embedding: vec![],
                updated_at: chrono::Utc::now(),
            });

        Ok(ScrapedGitHubData {
            username: matched.username,
            github_id: matched.github_id,
            display_name: matched.display_name,
            avatar_url: matched.avatar_url,
            bio: matched.bio,
            company: matched.company,
            github_url: matched.github_url,
            email: matched.email,
            top_repos: matched.top_repos,
            languages: matched.top_languages,
        })
    }
}
