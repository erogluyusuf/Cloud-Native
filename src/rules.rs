use regex::Regex;

pub struct Rule {
    pub name: &'static str,
    pub pattern: Regex,
}

pub fn get_rules() -> Vec<Rule> {
    vec![
        Rule {
            name: "AWS Access Key",
            pattern: Regex::new(r"(?i)AKIA[0-9A-Z]{16}").unwrap(),
        },
        Rule {
            name: "RSA Private Key",
            pattern: Regex::new(r"-----BEGIN RSA PRIVATE KEY-----").unwrap(),
        },
        // İleride buraya JWT, GitHub Token vb. için yeni kurallar eklenecek
    ]
}
