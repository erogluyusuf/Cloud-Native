use walkdir::WalkDir;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;
use colored::*;
use crate::rules::get_rules;
use crate::report::Finding;

// Shannon Entropy Hesaplama Fonksiyonu
pub fn calculate_entropy(data: &str) -> f64 {
    let mut char_counts = HashMap::new();
    for ch in data.chars() {
        *char_counts.entry(ch).or_insert(0) += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;
    for count in char_counts.values() {
        let p = *count as f64 / len;
        entropy -= p * p.log2();
    }
    entropy
}

pub fn scan_directory(path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let rules = get_rules();

    // 1. ZIRH: Kısayolları (Symlinks) takip et, hiçbir yere gizlenemesinler.
    let walker = WalkDir::new(path)
        .follow_links(true) 
        .into_iter();

    for entry in walker {
        // 2. ZIRH: Hataları yutma! Yetki yoksa ekrana uyarı bas ki kaçak nerede bilelim.
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                if let Some(path) = err.path() {
                    eprintln!("{}", format!("[WARNING] Permission Denied or Unreadable: {}", path.display()).yellow());
                }
                continue;
            }
        };

        let file_name = entry.file_name().to_string_lossy();
        if file_name == "target" || file_name == ".git" {
            continue; // Kendi derleme klasörümüzü taramayalım
        }

        if entry.file_type().is_file() {
            let file_path = entry.path().to_string_lossy().to_string();
            
            match File::open(entry.path()) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    let mut line_buffer = String::new();
                    let mut line_number = 0;

                    // 3. ZIRH: Satırları iterator ile değil, raw byte parse mantığıyla oku.
                    // Eğer dosya binary ise veya bozuk karakter varsa crash olmadan okuyabildiği kadarını okur.
                    loop {
                        line_number += 1;
                        line_buffer.clear();
                        
                        match reader.read_line(&mut line_buffer) {
                            Ok(0) => break, // Dosya sonu (EOF)
                            Ok(_) => {
                                let content = line_buffer.trim();
                                if content.is_empty() { continue; }

                                // Regex Taraması
                                for rule in &rules {
                                    if rule.pattern.is_match(content) {
                                        findings.push(Finding {
                                            file_path: file_path.clone(),
                                            rule_name: rule.name.to_string(),
                                            line_number,
                                        });
                                    }
                                }

                                // Entropy Taraması
                                for word in content.split_whitespace() {
                                    if word.len() > 16 {
                                        let ent = calculate_entropy(word);
                                        if ent > 4.5 {
                                            // Kendi rule dosyamızdaki patternleri zafiyet sanmasını engelleyelim
                                            if !file_path.ends_with("rules.rs") {
                                                findings.push(Finding {
                                                    file_path: file_path.clone(),
                                                    rule_name: format!("High Entropy String (Score: {:.2})", ent),
                                                    line_number,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Eğer dosya okunamaz karakterler içeriyorsa (örn: .png, .so) bu satırı atla
                                // Ama dosyanın geri kalanını taramaya devam et! Kaçış yok.
                                continue; 
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", format!("[WARNING] Could not open file {}: {}", file_path, e).yellow());
                }
            }
        }
    }
    findings
}