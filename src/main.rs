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

// JSON Dosyası için Yapılar (Structs)
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

    println!("{} {}", "[*] Watchman Aktif:".cyan().bold(), username.yellow());

    // 1. JSON Kurallarını Oku ve Derle
    let rules_content = fs::read_to_string("rules.json").expect("[HATA] rules.json dosyası bulunamadı!");
    let config: RuleConfig = serde_json::from_str(&rules_content).expect("[HATA] rules.json formatı bozuk!");
    
    let mut signatures: Vec<(String, Regex)> = Vec::new();
    for rule in config.rules {
        if let Ok(re) = Regex::new(&rule.pattern) {
            signatures.push((rule.name, re));
        } else {
            println!("{} Geçersiz Regex Atlandı: {}", "[Uyarı]".yellow(), rule.name);
        }
    }
    
    println!("{} Toplam {} güvenlik kuralı yüklendi.", "[*]".cyan(), signatures.len());

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
                        // Dinamik Yüklenen Kuralları Çalıştır
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
