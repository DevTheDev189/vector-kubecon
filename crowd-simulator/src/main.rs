use clap::{Parser, ValueEnum};
use reqwest::Client;
use serde_json::{json, Value};
use std::{
    error::Error,
    time::{Duration, Instant},
};
use tokio::task::JoinSet;

type SimError = Box<dyn Error + Send + Sync>;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Mode {
    Wave,
    Surge,
    Reset,
}

#[derive(Parser, Debug)]
#[command(
    name = "crowd-simulator",
    about = "VECTOR live crowd and telemetry simulator"
)]
struct CliArgs {
    #[arg(short, long, default_value = "http://127.0.0.1:8080")]
    target: String,
    #[arg(short, long, value_enum, default_value = "wave")]
    mode: Mode,
    #[arg(short, long, default_value_t = 25, value_parser = clap::value_parser!(u16).range(1..=256))]
    concurrency: u16,
}

async fn post(client: &Client, target: &str, path: &str, body: Value) -> Result<Value, SimError> {
    let response = client
        .post(format!("{target}{path}"))
        .json(&body)
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        return Err(format!("{path}: {status}: {text}").into());
    }
    Ok(serde_json::from_str(&text)?)
}

#[tokio::main]
async fn main() -> Result<(), SimError> {
    let args = CliArgs::parse();
    let target = args.target.trim_end_matches('/');
    let url = reqwest::Url::parse(target)?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("Target must use HTTP or HTTPS".into());
    }
    let session_cookie = std::env::var("VECTOR_SESSION_COOKIE")
        .map_err(|_| "Set VECTOR_SESSION_COOKIE to a signed-in administrator's vector_sid cookie before running the simulator")?;
    let mut headers = reqwest::header::HeaderMap::new();
    let mut cookie = reqwest::header::HeaderValue::from_str(&session_cookie)?;
    cookie.set_sensitive(true);
    headers.insert(reqwest::header::COOKIE, cookie);
    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(30))
        .build()?;
    let started = Instant::now();
    if matches!(args.mode, Mode::Reset) {
        let result = post(&client, target, "/api/surge/reset", json!({})).await?;
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }
    // Discover real room IDs so wave mode works with the deployed dataset.
    let rooms: Value = client
        .get(format!("{target}/api/rooms"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let ids: Vec<String> = rooms["rooms"]
        .as_array()
        .ok_or("Missing rooms array")?
        .iter()
        .filter_map(|r| r["room_id"].as_str().map(str::to_owned))
        .collect();
    if ids.is_empty() {
        return Err("No rooms available".into());
    }
    let mut tasks = JoinSet::new();
    for index in 0..usize::from(args.concurrency) {
        let client = client.clone();
        let target = target.to_owned();
        let surge = matches!(args.mode, Mode::Surge);
        let room = if surge {
            "ballroom-b".to_owned()
        } else {
            ids[index % ids.len()].clone()
        };
        tasks.spawn(async move {
            post(&client, &target, "/api/rooms/checkin", json!({"room_id": room, "delta": 1, "decibel_reading": if surge {84.5} else {45.0 + (index % 30) as f64}})).await?;
            if !surge {
                post(&client, &target, "/api/search/talks", json!({"query": "Rust Kubernetes observability", "exclude_full_rooms": true})).await?;
                // Simulate departure after the search.
                post(&client, &target, "/api/rooms/checkin", json!({"room_id": room, "delta": -1})).await?;
            }
            Ok::<_, SimError>(())
        });
    }
    let mut failures = 0;
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                failures += 1;
                eprintln!("Attendee request failed: {error}");
            }
            Err(error) => {
                failures += 1;
                eprintln!("Attendee task failed: {error}");
            }
        }
    }
    if failures > 0 {
        return Err(format!("{failures} attendee tasks failed").into());
    }
    if matches!(args.mode, Mode::Surge) {
        let result = post(
            &client,
            target,
            "/api/surge",
            json!({"room_id": "ballroom-b"}),
        )
        .await?;
        println!("{}", serde_json::to_string_pretty(&result)?);
    }
    println!(
        "Completed {:?}: {} attendees in {:.1}ms",
        args.mode,
        args.concurrency,
        started.elapsed().as_secs_f64() * 1000.0
    );
    Ok(())
}
