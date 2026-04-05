use serde::Serialize;
use colored::*;

#[derive(Serialize, Debug)]
pub struct Finding {
    pub file_path: String,
    pub rule_name: String,
    pub line_number: usize,
}

pub fn print_text_report(findings: &[Finding]) {
    if findings.is_empty() {
        println!("{}", "[+] No secrets found. You are safe!".green());
        return;
    }
    println!("{}", "[!] Potential secrets found:".red().bold());
    for finding in findings {
        println!("  - [{}] found in {} at line {}", 
            finding.rule_name.red(), 
            finding.file_path.yellow(), 
            finding.line_number
        );
    }
}

pub fn print_json_report(findings: &[Finding]) {
    let json = serde_json::to_string_pretty(findings).unwrap();
    println!("{}", json);
}
