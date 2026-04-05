mod cli;
mod rules;
mod report;
mod scanner;
mod docker;
mod reporter;
mod database;
mod hunter;
mod validator;

use std::process;
use dotenv::dotenv;
use colored::*;

#[tokio::main]
async fn main() {
    // RUSTLS KRİZİNİ KÖKÜNDEN ÇÖZEN SATIR: Açıkça 'ring' motorunu seçiyoruz
    let _ = rustls::crypto::ring::default_provider().install_default();

    dotenv().ok();
    let args = cli::parse_args();
    let mut all_findings = Vec::new();

    println!("{}", "[*] Vault Hound Started...".cyan().bold());

    let db_conn = database::init_db().expect("[ERROR] SQLite veritabanı başlatılamadı!");

    // Avlanma (Hunter) Modu Aktif mi?
    if let Some(query) = args.hunt {
        if let Err(e) = hunter::start_hunt(&query, &db_conn).await {
            eprintln!("[ERROR] Avlanma sırasında hata: {}", e);
        }
        return;
    }

    if let (Some(owner), Some(repo)) = (args.report_owner, args.report_repo) {
        match database::is_repo_scanned(&db_conn, &owner, &repo) {
            Ok(true) => {
                println!("{}", format!("[!] ATLANDI: {}/{} daha önce raporlanmış.", owner, repo).yellow());
                return;
            }
            Ok(false) => println!("[*] Yeni repo tespit edildi, raporlama başlatılıyor..."),
            Err(e) => eprintln!("[ERROR] DB hatası: {}", e),
        }

        let title = "🚨 [Vault Hound] Security Alert: Hardcoded Secrets Detected!";
        let body = "### Vault Hound Security Scan\n\nMerhaba! Sistemimiz bu repoda hassas bir veri (API Key vb.) tespit etti. Lütfen credential rotasyonunu sağlayın.";
        
        if let Err(e) = reporter::open_issue(&owner, &repo, title, body).await {
            eprintln!("{}", format!("[ERROR] Issue açılamadı: {}", e).red());
        } else {
            let _ = database::mark_repo_scanned(&db_conn, &owner, &repo);
        }
        return; 
    }

    if let Some(image_path) = args.image {
        let mut docker_findings = docker::scan_tar_image(&image_path);
        all_findings.append(&mut docker_findings);
    } else {
        println!("[*] Target Path: {}", args.path);
        let mut dir_findings = scanner::scan_directory(&args.path);
        all_findings.append(&mut dir_findings);
    }

    if args.format.to_lowercase() == "json" {
        report::print_json_report(&all_findings);
    } else {
        report::print_text_report(&all_findings);
    }

    if args.strict && !all_findings.is_empty() {
        process::exit(1);
    }
}
