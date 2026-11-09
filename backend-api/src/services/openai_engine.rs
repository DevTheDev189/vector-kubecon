use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::AppConfig;
use crate::error::ApiError;
use crate::models::BuilderProfile;
use crate::services::elastic_client::ElasticClient;
use crate::services::github_scraper::ScrapedGitHubData;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtractedDna {
    pub persona_title: String,
    pub dna_tags: Vec<String>,
    pub hallway_icebreaker: String,
    pub ideal_talk_query: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DraftedFollowUp {
    pub ai_context_summary: String,
    pub draft_linkedin_dm: String,
    pub draft_email_subject: String,
    pub draft_email_body: String,
}

#[derive(Clone)]
pub struct OpenAiEngine {
    pub config: AppConfig,
    pub http: Client,
}

impl OpenAiEngine {
    pub fn new(config: AppConfig, http: Client) -> Self {
        Self { config, http }
    }

    /// Extracts Builder DNA from scraped GitHub repositories using OpenAI / OpenRouter.
    /// Includes heuristic fallback so offline testing and unauthenticated users never fail.
    pub async fn extract_builder_dna(
        &self,
        scraped: &ScrapedGitHubData,
    ) -> Result<ExtractedDna, ApiError> {
        if let Some(api_key) = &self.config.openai_api_key {
            let prompt = format!(
                "Analyze this cloud-native developer's GitHub profile:\nUsername: {}\nBio: {}\nLanguages: {:?}\nRepos: {:?}\n\n\
                Extract their developer DNA as JSON with keys:\n\
                - \"persona_title\": (e.g. 'eBPF Kernel Hacker & Cilium Specialist')\n\
                - \"dna_tags\": array of 5 to 7 specific technical tags (e.g. ['eBPF', 'Rust', 'Kubernetes', 'mTLS'])\n\
                - \"hallway_icebreaker\": 1-2 sentence conversational icebreaker referencing their specific projects\n\
                - \"ideal_talk_query\": search query representing their technical interests",
                scraped.username, scraped.bio, scraped.languages, scraped.top_repos
            );

            let endpoint = format!(
                "{}/chat/completions",
                self.config.openai_base_url.trim_end_matches('/')
            );
            let payload = json!({
                "model": self.config.openai_model,
                "messages": [
                    { "role": "system", "content": "You are the VECTOR KubeCon Conference AI. Respond with valid JSON only." },
                    { "role": "user", "content": prompt }
                ],
                "response_format": { "type": "json_object" },
                "temperature": 0.4
            });

            let resp = self
                .http
                .post(&endpoint)
                .bearer_auth(api_key)
                .json(&payload)
                .send()
                .await;

            if let Ok(response) = resp {
                if response.status().is_success() {
                    if let Ok(body) = response.json::<Value>().await {
                        if let Some(content) = body["choices"][0]["message"]["content"].as_str() {
                            if let Ok(extracted) = serde_json::from_str::<ExtractedDna>(content) {
                                return Ok(extracted);
                            }
                        }
                    }
                }
            }
        }

        // Robust heuristic fallback
        Ok(self.heuristic_dna_extraction(scraped))
    }

    /// Generates a 1536-dimensional embedding vector, falling back to deterministic hashing.
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, ApiError> {
        if let Some(api_key) = &self.config.openai_api_key {
            let endpoint = format!(
                "{}/embeddings",
                self.config.openai_base_url.trim_end_matches('/')
            );
            let payload = json!({
                "model": self.config.openai_embedding_model,
                "input": text,
                "dimensions": 1536
            });

            let resp = self
                .http
                .post(&endpoint)
                .bearer_auth(api_key)
                .json(&payload)
                .send()
                .await;

            if let Ok(response) = resp {
                if response.status().is_success() {
                    if let Ok(body) = response.json::<Value>().await {
                        if let Some(data) = body["data"][0]["embedding"].as_array() {
                            let floats: Vec<f32> = data
                                .iter()
                                .filter_map(|v| v.as_f64().map(|f| f as f32))
                                .collect();
                            if floats.len() == 1536 {
                                return Ok(floats);
                            }
                        }
                    }
                }
            }
        }

        // Deterministic unit cosine fallback
        Ok(ElasticClient::generate_deterministic_embedding(text))
    }

    /// Drafts personalized LinkedIn DM and email follow-up connecting two builders.
    pub async fn draft_handshake_followup(
        &self,
        initiator: &BuilderProfile,
        peer: &BuilderProfile,
        room_name: &str,
        talk_title: Option<&str>,
    ) -> Result<DraftedFollowUp, ApiError> {
        let talk_context = talk_title.unwrap_or("the hallway session");

        if let Some(api_key) = &self.config.openai_api_key {
            let prompt = format!(
                "Draft a hyper-relevant follow-up message between two cloud-native builders who just connected at KubeCon NA SLC:\n\
                Builder A: {} ({})\nBuilder B: {} ({})\nLocation: {} during '{}'\n\
                Shared DNA tags: {:?}\n\n\
                Output JSON with:\n\
                - \"ai_context_summary\": 2-sentence summary of why they matched\n\
                - \"draft_linkedin_dm\": concise LinkedIn message under 280 characters\n\
                - \"draft_email_subject\": professional subject line\n\
                - \"draft_email_body\": 2-paragraph email following up on their conversation",
                initiator.display_name, initiator.persona_title,
                peer.display_name, peer.persona_title,
                room_name, talk_context,
                initiator.dna_tags
            );

            let endpoint = format!(
                "{}/chat/completions",
                self.config.openai_base_url.trim_end_matches('/')
            );
            let payload = json!({
                "model": self.config.openai_model,
                "messages": [
                    { "role": "system", "content": "You are the VECTOR Hallway Handshake Copilot. Respond with valid JSON only." },
                    { "role": "user", "content": prompt }
                ],
                "response_format": { "type": "json_object" },
                "temperature": 0.5
            });

            let resp = self
                .http
                .post(&endpoint)
                .bearer_auth(api_key)
                .json(&payload)
                .send()
                .await;

            if let Ok(response) = resp {
                if response.status().is_success() {
                    if let Ok(body) = response.json::<Value>().await {
                        if let Some(content) = body["choices"][0]["message"]["content"].as_str() {
                            if let Ok(followup) = serde_json::from_str::<DraftedFollowUp>(content) {
                                return Ok(followup);
                            }
                        }
                    }
                }
            }
        }

        // Deterministic fallback
        Ok(DraftedFollowUp {
            ai_context_summary: format!(
                "Connected outside {} during '{}'. Strong technical overlap across {}.",
                room_name, talk_context, initiator.dna_tags.first().cloned().unwrap_or_else(|| "Kubernetes".to_string())
            ),
            draft_linkedin_dm: format!(
                "Hey {}! Great catching up outside {} during {}. Let's stay in touch on our cloud-native work!",
                peer.display_name, room_name, talk_context
            ),
            draft_email_subject: format!("Great meeting at KubeCon SLC // {}", talk_context),
            draft_email_body: format!(
                "Hi {},\n\nIt was great connecting with you today outside {} after '{}'. I really enjoyed learning about your work on {}.\n\nLet's definitely follow up next week to discuss collaboration!\n\nBest,\n{}",
                peer.display_name, room_name, talk_context, peer.persona_title, initiator.display_name
            ),
        })
    }

    fn heuristic_dna_extraction(&self, scraped: &ScrapedGitHubData) -> ExtractedDna {
        let mut tags = Vec::new();
        for lang in &scraped.languages {
            tags.push(lang.clone());
        }
        for repo in &scraped.top_repos {
            for topic in &repo.topics {
                if !tags.contains(topic) && tags.len() < 7 {
                    tags.push(topic.clone());
                }
            }
        }
        if tags.is_empty() {
            tags = vec![
                "Rust".to_string(),
                "Kubernetes".to_string(),
                "eBPF".to_string(),
                "OpenTelemetry".to_string(),
                "Cloud-Native".to_string(),
            ];
        }

        let primary_lang = scraped
            .languages
            .first()
            .cloned()
            .unwrap_or_else(|| "Rust".to_string());
        let persona_title = format!("Cloud-Native {} Systems Specialist", primary_lang);

        let top_repo_name = scraped
            .top_repos
            .first()
            .map(|r| r.name.as_str())
            .unwrap_or("cloud-native infrastructure");

        let hallway_icebreaker = format!(
            "Ask me about my work on {} and building high-performance {} services on Kubernetes.",
            top_repo_name, primary_lang
        );

        let ideal_talk_query = format!(
            "{} kubernetes cloud-native performance",
            primary_lang.to_lowercase()
        );

        ExtractedDna {
            persona_title,
            dna_tags: tags,
            hallway_icebreaker,
            ideal_talk_query,
        }
    }
}
