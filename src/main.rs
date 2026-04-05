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

    // 1. Kullanıcının tüm repolarını "son güncellenme" sırasına göre çek
    let repos = octocrab.current().list_repos_for_authenticated_user()
        .sort(octocrab::params::repos::Sort::Pushed)
        .per_page(50)
        .send()
        .await?;

    // 2. İmza kütüphanesi
    let signatures = vec![
        ("AWS Key", Regex::new(r"AKIA[0-9A-Z]{16}")?),
        ("Discord Token", Regex::new(r"[a-zA-Z0-9_-]{24}\.[a-zA-Z0-9_-]{6}\.[a-zA-Z0-9_-]{27}")?),
        ("GitHub PAT", Regex::new(r"ghp_[a-zA-Z0-9]{36}")?),
    ];

    let now = Utc::now();
    let threshold = Duration::minutes(10); // Son 10 dakikada güncellenenleri tara

    for repo in repos {
        let pushed_at = repo.pushed_at.unwrap_or(repo.created_at.unwrap());
        if now.signed_duration_since(pushed_at) > threshold {
            continue; // Eski repoları tara geç
        }

        println!("{} {}/{}", "[!] Aktivite Tespit Edildi:".green(), username, repo.name);

        // 3. Reponun içeriğini indir
        let tarball_url = format!("https://api.github.com/repos/{}/{}/tarball", username, repo.name);
        let client = reqwest::Client::new();
        let res = client.get(tarball_url).bearer_auth(&token).header("User-Agent", "Vault-Hound").send().await?;
        
        if !res.status().is_success() { continue; }
        
        let bytes = res.bytes().await?;
        let mut archive = Archive::new(GzDecoder::new(Cursor::new(bytes)));
        let mut findings = String::new();

        // 4. Dosya tarama
        if let Ok(entries) = archive.entries() {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path()?.to_string_lossy().to_string();
                if path.contains("/target/") || path.contains("/.git/") { continue; }

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

        // 5. Sızıntı varsa O REPOYA Issue aç
        if !findings.is_empty() {
            println!("{} Issue açılıyor...", "[*]".yellow());
            octocrab.issues(&username, &repo.name)
                .create("🚨 KRİTİK GÜVENLİK UYARISI")
                .body(format!("Vault Hound Watchman bu repoda sızıntı buldu:\n\n```\n{}\n```\nLütfen acilen bu veriyi silin ve anahtarı iptal edin!", findings))
                .send()
                .await?;
        }
    }

    Ok(())
}
