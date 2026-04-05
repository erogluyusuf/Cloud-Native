use octocrab::Octocrab;
use colored::*;
use std::env;

pub async fn open_issue(owner: &str, repo: &str, title: &str, body: &str) -> Result<(), Box<dyn std::error::Error>> {
    // .env dosyasından token'ı al
    let token = env::var("GITHUB_TOKEN").expect("[ERROR] GITHUB_TOKEN bulunamadı! Lütfen .env dosyanı kontrol et.");
    
    // GitHub API İstemcisini oluştur
    let octocrab = Octocrab::builder().personal_token(token).build()?;
    
    println!("{}", format!("[*] Sending report to GitHub -> {}/{}", owner, repo).cyan());
    
    // Issue'yu aç
    let issue = octocrab.issues(owner, repo)
        .create(title)
        .body(body)
        .send()
        .await?;
        
    println!("{}", format!("[+] Success! Issue created at: {}", issue.html_url).green().bold());
    Ok(())
}
