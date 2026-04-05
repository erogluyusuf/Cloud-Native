use octocrab::Octocrab;
use regex::Regex;
use std::env;
use std::io::{BufRead, BufReader, Cursor};
use tar::Archive;
use flate2::read::GzDecoder;
use chrono::{Utc, Duration};
use colored::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN missing");
    let octocrab = Octocrab::builder().personal_token(token.clone()).build()?;
    let user = octocrab.current().user().await?;
    let username = user.login;

    println!("{} {}", "[*] Watchman Aktif:".cyan().bold(), username.yellow());

    // ÇÖZÜM BURADA: Enum yerine doğrudan "pushed" yazısı kullanıyoruz
    let repos = octocrab.current().list_repos_for_authenticated_user()
        .sort("pushed")
        .per_page(50)
        .send()
        .await?;

    let signatures = vec![
        ("AWS Key", Regex::new(r"AKIA[0-9A-Z]{16}")?),
        ("Discord Token", Regex::new(r"[a-zA-Z0-9_-]{24}\.[a-zA-Z0-9_-]{6}\.[a-zA-Z0-9_-]{27}")?),
        ("GitHub PAT", Regex::new(r"ghp_[a-zA-Z0-9]{36}")?),
    ];

    let now = Utc::now();
    let threshold = Duration::minutes(15); 

    for repo in repos {
        let pushed_at = repo.pushed_at.unwrap_or(repo.created_at.unwrap());
        if now.signed_duration_since(pushed_at) > threshold {
            continue; 
        }

        println!("{} {}/{}", "[!] Aktivite Tespit Edildi:".green(), username, repo.name);

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
                    let hit = format!("🚨 KRİTİK DOSYA İFŞASI: '.env' dosyası public repoya pushlandı! -> {}\n", path);
                    println!("    {}", hit.red().bold());
                    findings.push_str(&hit);
                }

                let reader = BufReader::new(entry);
                for (i, line) in reader.lines().enumerate() {
                    if let Ok(content) = line {
                        for (name, re) in &signatures {
                            if re.is_match(&content) {
                                let hit = format!("🚨 {} sızıntısı: {} (Satır: {})\n", name, path, i + 1);
                                println!("    {}", hit.red().bold());
                                findings.push_str(&hit);
                            }
                        }
                    }
                }
            }
        }

        if !findings.is_empty() {
            println!("{} Issue açılıyor...", "[*]".yellow());
            octocrab.issues(&username, &repo.name)
                .create("🚨 KRİTİK GÜVENLİK UYARISI")
                .body(format!("**Vault Hound Watchman** bu repoda ciddi bir güvenlik açığı buldu:\n\n```\n{}\n```\nLütfen acilen bu dosyayı silin veya sızan anahtarı iptal edin!", findings))
                .send()
                .await?;
        }
    }

    Ok(())
}
