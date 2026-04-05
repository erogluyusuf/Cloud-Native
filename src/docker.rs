use tar::Archive;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use colored::*;
use crate::rules::get_rules;
use crate::report::Finding;
use crate::scanner::calculate_entropy;

pub fn scan_tar_image(image_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let rules = get_rules();

    let file = match File::open(image_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}", format!("[ERROR] Could not open Docker image {}: {}", image_path, e).red());
            return findings;
        }
    };

    let mut archive = Archive::new(file);

    println!("{}", format!("[*] Inspecting Docker image layers in RAM: {}", image_path).cyan());

    for entry in archive.entries().unwrap().filter_map(|e| e.ok()) {
        let path = entry.path().unwrap().to_string_lossy().to_string();
        
        // Sadece normal dosyaları tarıyoruz, dizinleri ve linkleri geçiyoruz
        if entry.header().entry_type().is_file() {
            let mut reader = BufReader::new(entry);
            let mut line_buffer = String::new();
            let mut line_number = 0;

            loop {
                line_number += 1;
                line_buffer.clear();
                
                match reader.read_line(&mut line_buffer) {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let content = line_buffer.trim();
                        if content.is_empty() { continue; }

                        // 1. Regex Taraması
                        for rule in &rules {
                            if rule.pattern.is_match(content) {
                                findings.push(Finding {
                                    file_path: format!("{} (Inside Docker)", path),
                                    rule_name: rule.name.to_string(),
                                    line_number,
                                });
                            }
                        }

                        // 2. Entropy Taraması
                        for word in content.split_whitespace() {
                            if word.len() > 16 {
                                let ent = calculate_entropy(word);
                                if ent > 4.5 {
                                    findings.push(Finding {
                                        file_path: format!("{} (Inside Docker)", path),
                                        rule_name: format!("High Entropy String (Score: {:.2})", ent),
                                        line_number,
                                    });
                                }
                            }
                        }
                    }
                    Err(_) => continue, // Binary datayı atla
                }
            }
        }
    }
    findings
}
