use std::path::PathBuf;

use homedir::unix::my_home;
use reqwest::{self, Response, StatusCode};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct StarredRepo {
    id: usize,

    node_id: String,

    name: String,

    full_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Release {
    html_url: String,

    name: String,

    published_at: chrono::DateTime<chrono::Utc>,
}

struct LatestRelease {
    repo: String,

    version: String,

    published_at: chrono::DateTime<chrono::Utc>,

    html_url: String,
}

async fn get_repos(client: &Client) -> Result<Vec<StarredRepo>, reqwest::Error> {
    return client
        .get("https://api.github.com/user/starred?per_page=100")
        .send()
        .await?
        .json::<Vec<StarredRepo>>()
        .await;
}

async fn get_latest_release(client: &Client, repo: &String) -> Result<Response, reqwest::Error> {
    return client
        .get(format!(
            "https://api.github.com/repos/{}/releases/latest",
            repo,
        ))
        .send()
        .await;
}

fn get_access_token() -> String {
    let home_directory = my_home()
        .expect("Could not find user home directory")
        .unwrap();

    let token_file = PathBuf::new().join(home_directory).join(".rrs_token");

    let github_token = std::fs::read(token_file).expect("Failed to access GitHub Access Token");
    return String::from_utf8(github_token).expect("Failed to convert access token to string");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let access_token = get_access_token();
    let authorization_header = format!("Bearer {}", access_token.trim()).leak();

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "X-GitHub-Api-Version",
        header::HeaderValue::from_static("2022-11-28"),
    );
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_static(authorization_header),
    );
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("App-Recent-Releases-Stars"),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    let repos = get_repos(&client).await?;

    let mut releases = Vec::<LatestRelease>::new();
    for repo in repos {
        let repo_name = repo.full_name.clone();

        let response = get_latest_release(&client, &repo_name).await?;
        if response.status() == StatusCode::NOT_FOUND {
            continue;
        }

        let release = response.json::<Release>().await?;
        releases.push(LatestRelease {
            repo: repo_name,
            version: release.name,
            published_at: release.published_at,
            html_url: release.html_url,
        });
    }

    releases.sort_by(|a, b| b.published_at.cmp(&a.published_at));

    for release in releases {
        println!(
            "{}\n   - Version : {}\n   - Date : {}\n   - {}\n",
            release.repo,
            release.version,
            release.published_at.format("%d/%m/%Y"),
            release.html_url,
        );
    }

    return Ok(());
}
