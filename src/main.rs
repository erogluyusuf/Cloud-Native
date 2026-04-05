use octocrab::Octocrab;
use regex::Regex;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Cursor};
use tar::Archive;
use flate2::read::GzDecoder;
use chrono::{Utc, Duration};
use colored::*;
use serde::Deserialize;

// Structs for JSON File
#[derive(Deserialize)]
struct RuleConfig {
    rules: Vec<Rule>,
}

#[derive(Deserialize)]
struct Rule {
    name: String,
    pattern: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN missing");
    let octocrab = Octocrab::builder().personal_token(token.clone()).build()?;
    let user = octocrab.current().user().await?;
    let username = user.login;

    println!("{} {}", "[*] Watchman Active:".cyan().bold(), username.yellow());

    // 1. Read and Compile JSON Rules
    let rules_content = fs::read_to_string("rules.json").expect("[ERROR] rules.json file not found!");
    let config: RuleConfig = serde_json::from_str(&rules_content).expect("[ERROR] Invalid rules.json format!");
    
    let mut signatures: Vec<(String, Regex)> = Vec::new();
    for rule in config.rules {
        if let Ok(re) = Regex::new(&rule.pattern) {
            signatures.push((rule.name, re));
        } else {
            println!("{} Invalid Regex Skipped: {}", "[Warning]".yellow(), rule.name);
        }
    }
    
    println!("{} Total {} security rules loaded.", "[*]".cyan(), signatures.len());

    let repos = octocrab.current().list_repos_for_authenticated_user()
        .sort("pushed")
        .per_page(50)
        .send()
        .await?;

    let now = Utc::now();
    let threshold = Duration::minutes(15); 

    for repo in repos {
        let pushed_at = repo.pushed_at.unwrap_or(repo.created_at.unwrap());
        if now.signed_duration_since(pushed_at) > threshold {
            continue; 
        }

        println!("{} {}/{}", "[!] Activity Detected:".green(), username, repo.name);

        let tarball_url = format!("https://api.github.com/repos/{}/{}/tarball", username, repo.name);
        let client = reqwest::Client::new();
        let res = client.get(tarball_url).bearer_auth(&token).header("User-Agent", "Vault-Hound").send().await?;
        
        if !res.status().is_success() { continue; }
        
        let bytes = res.bytes().await?;
        let mut archive = Archive::new(GzDecoder::new(Cursor::new(bytes)));
        let mut findings = String::new();

        if let Ok(entries) = archive.entries() {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path().unwrap_or_default().to_string_lossy().to_string();
                if path.contains("/target/") || path.contains("/.git/") { continue; }

                if path.ends_with(".env") || path.contains("/.env") {
                    let hit = format!("🚨 CRITICAL FILE EXPOSURE: '.env' file pushed to public repo! -> {}\n", path);
                    println!("    {}", hit.red().bold());
                    findings.push_str(&hit);
                }

                let reader = BufReader::new(entry);
                for (i, line) in reader.lines().enumerate() {
                    if let Ok(content) = line {
                        // Execute Dynamically Loaded Rules
                        for (name, re) in &signatures {
                            if re.is_match(&content) {
                                let hit = format!("🚨 {} leak: {} (Line: {})\n", name, path, i + 1);
                                println!("    {}", hit.red().bold());
                                findings.push_str(&hit);
                            }
                        }
                    }
                }
            }
        }

        if !findings.is_empty() {
            println!("{} Opening Issue...", "[*]".yellow());
            octocrab.issues(&username, &repo.name)
                .create("🚨 CRITICAL SECURITY ALERT")
                .body(format!("**Vault Hound Watchman** has found a serious security vulnerability in this repository:\n\n```\n{}\n```\nPlease delete this file or revoke the leaked key immediately!", findings))
                .send()
                .await?;
        }
    }

    Ok(())
}