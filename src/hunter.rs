use reqwest::Client;
use std::env;
use std::io::{BufRead, BufReader, Cursor};
use tar::Archive;
use flate2::read::GzDecoder;
use colored::*;
use crate::database::{is_repo_scanned, mark_repo_scanned};
use crate::scanner::calculate_entropy;
use octocrab::Octocrab;
use rusqlite::Connection;
use regex::Regex;
use std::fs;
use std::path::Path;

// Yerel Dizindeki Dosyaları Gezmek İçin Yardımcı Fonksiyon
fn scan_local_dir(dir: &Path, re_gemini: &Regex, re_github: &Regex, re_discord: &Regex, re_aws: &Regex) {
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                let path_str = path.to_string_lossy().to_string();
                let p_lower = path_str.to_lowercase();

                // KRİTİK GÜNCELLEME: Çöp ve binary klasörleri (target, .git, vs) tamamen ATLA!
                if p_lower.contains("/target/") || p_lower.contains("\\target\\") ||
                   p_lower.contains("/.git/") || p_lower.contains("\\.git\\") ||
                   p_lower.ends_with(".png") || p_lower.ends_with(".jpg") || p_lower.ends_with(".jpeg") ||
                   p_lower.ends_with(".svg") || p_lower.ends_with(".gif") || p_lower.ends_with(".mp4") ||
                   p_lower.ends_with(".pdf") || p_lower.ends_with(".zip") || p_lower.ends_with(".tar.gz") ||
                   p_lower.ends_with(".lock") || p_lower.ends_with(".exe") || p_lower.ends_with(".dll") ||
                   p_lower.ends_with(".o") || p_lower.ends_with(".bin") || p_lower.ends_with(".rlib") || 
                   p_lower.ends_with(".rmeta") || p_lower.ends_with(".d") || p_lower.ends_with(".so") ||
                   p_lower.contains("node_modules") {
                    continue;
                }

                if path.is_dir() {
                    scan_local_dir(&path, re_gemini, re_github, re_discord, re_aws);
                } else {
                    if let Ok(file) = fs::File::open(&path) {
                        let reader = BufReader::new(file);
                        for (i, line) in reader.lines().enumerate() {
                            if let Ok(content) = line {
                                let line_number = i + 1;
                                
                                if re_gemini.is_match(&content) {
                                    println!("    [BULDUM] [Gemini Key] -> {}:{}", path_str, line_number);
                                }
                                if re_github.is_match(&content) {
                                    println!("    [BULDUM] [GitHub PAT] -> {}:{}", path_str, line_number);
                                }
                                if re_discord.is_match(&content) {
                                    println!("    [BULDUM] [Discord Token] -> {}:{}", path_str, line_number);
                                }
                                if re_aws.is_match(&content) {
                                    println!("    [BULDUM] [AWS Access Key] -> {}:{}", path_str, line_number);
                                }

                                for word in content.split_whitespace() {
                                    if word.len() > 20 && word.len() < 100 {
                                        let ent = calculate_entropy(word);
                                        if ent > 4.9 {
                                            println!("    [ŞÜPHELİ] [High Entropy: {:.2}] -> {}:{}", ent, path_str, line_number);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub async fn start_hunt(query: &str, db_conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let re_gemini = Regex::new(r"AIza[0-9A-Za-z-_]{35}")?;
    let re_github = Regex::new(r"ghp_[a-zA-Z0-9]{36}")?;
    let re_discord = Regex::new(r"([a-zA-Z0-9]{24}\.[a-zA-Z0-9]{6}\.[a-zA-Z0-9]{27})")?;
    let re_aws = Regex::new(r"AKIA[0-9A-Z]{16}")?;

    // EĞER CLI'DEN YEREL DİZİN TARAMASI İSTENİRSE (--path)
    if env::args().any(|arg| arg.starts_with("--path")) {
        println!("[*] Target Path: .");
        scan_local_dir(Path::new("."), &re_gemini, &re_github, &re_discord, &re_aws);
        return Ok(());
    }

    // NORMAL GITHUB API TARAMASI KISMI
    let token = env::var("GITHUB_TOKEN").expect("[ERROR] GITHUB_TOKEN bulunamadı!");
    let octocrab = Octocrab::builder().personal_token(token.clone()).build()?;
    
    println!("{}", format!("[*] AĞ GENİŞLETİLDİ: '{}' sorgusu ile taze Github repoları taranıyor...", query).cyan().bold());
    
    let page = octocrab.search().repositories(query).send().await?;
    let client = Client::builder().user_agent("vault_hound_passive_radar").build()?;

    for repo in page.items {
        let owner = repo.owner.unwrap().login;
        let repo_name = repo.name;
        
        if is_repo_scanned(db_conn, &owner, &repo_name).unwrap_or(false) { continue; }

        println!("{}", format!("[*] Radar Tarıyor: {}/{}", owner, repo_name).blue());
        
        let tar_url = format!("https://api.github.com/repos/{}/{}/tarball", owner, repo_name);
        let res = client.get(&tar_url).bearer_auth(&token).send().await?;

        if !res.status().is_success() { continue; }
        let bytes = res.bytes().await?;
        let mut archive = Archive::new(GzDecoder::new(Cursor::new(bytes)));

        if let Ok(entries) = archive.entries() {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.header().entry_type().is_file() {
                    let path_str = entry.path().unwrap_or_default().to_string_lossy().to_string();
                    let p_lower = path_str.to_lowercase();
                    
                    if p_lower.contains("/target/") || p_lower.ends_with(".png") || p_lower.ends_with(".jpg") || p_lower.ends_with(".jpeg") ||
                       p_lower.ends_with(".svg") || p_lower.ends_with(".gif") || p_lower.ends_with(".mp4") ||
                       p_lower.ends_with(".pdf") || p_lower.ends_with(".zip") || p_lower.ends_with(".tar.gz") ||
                       p_lower.ends_with(".lock") || p_lower.ends_with(".exe") || p_lower.ends_with(".dll") ||
                       p_lower.contains("node_modules") {
                        continue;
                    }

                    let mut reader = BufReader::new(entry);
                    let mut line_buffer = String::new();
                    let mut line_number = 0;

                    loop {
                        line_number += 1;
                        line_buffer.clear();
                        match reader.read_line(&mut line_buffer) {
                            Ok(0) => break,
                            Ok(_) => {
                                let content = line_buffer.trim();
                                if content.is_empty() { continue; }

                                if re_gemini.is_match(content) {
                                    println!("    {} [Gemini Key] -> {}:{}", "[BULDUM]".red().bold(), path_str, line_number);
                                }
                                if re_github.is_match(content) {
                                    println!("    {} [GitHub PAT] -> {}:{}", "[BULDUM]".red().bold(), path_str, line_number);
                                }
                                if re_discord.is_match(content) {
                                    println!("    {} [Discord Token] -> {}:{}", "[BULDUM]".red().bold(), path_str, line_number);
                                }
                                if re_aws.is_match(content) {
                                    println!("    {} [AWS Access Key] -> {}:{}", "[BULDUM]".red().bold(), path_str, line_number);
                                }

                                for word in content.split_whitespace() {
                                    if word.len() > 20 && word.len() < 100 {
                                        let ent = calculate_entropy(word);
                                        if ent > 4.9 {
                                            println!("    {} [High Entropy: {:.2}] -> {}:{}", "[ŞÜPHELİ]".yellow().bold(), ent, path_str, line_number);
                                        }
                                    }
                                }
                            }
                            Err(_) => continue,
                        }
                    }
                }
            }
        }
        let _ = mark_repo_scanned(db_conn, &owner, &repo_name);
    }
    Ok(())
}
