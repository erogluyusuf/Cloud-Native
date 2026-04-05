use rusqlite::{Connection, Result};
use colored::*;

pub fn init_db() -> Result<Connection> {
    // vault_hound.db dosyasını oluştur veya bağlan
    let conn = Connection::open("vault_hound.db")?;
    
    // Eğer tablo yoksa oluştur
    conn.execute(
        "CREATE TABLE IF NOT EXISTS scanned_repos (
            id INTEGER PRIMARY KEY,
            owner TEXT NOT NULL,
            repo TEXT NOT NULL,
            scanned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(owner, repo)
        )",
        [],
    )?;
    Ok(conn)
}

pub fn is_repo_scanned(conn: &Connection, owner: &str, repo: &str) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT 1 FROM scanned_repos WHERE owner = ?1 AND repo = ?2")?;
    let exists = stmt.exists([owner, repo])?;
    Ok(exists)
}

pub fn mark_repo_scanned(conn: &Connection, owner: &str, repo: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO scanned_repos (owner, repo) VALUES (?1, ?2)",
        [owner, repo],
    )?;
    println!("{}", format!("[+] DB UPDATE: {}/{} hafızaya kaydedildi. Bir daha taranmayacak.", owner, repo).green());
    Ok(())
}
