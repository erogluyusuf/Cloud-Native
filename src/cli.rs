use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "Vault Hound")]
#[command(version = "1.0")]
#[command(about = "Cloud-Native Secret & Config Scanner", long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value = ".")]
    pub path: String,

    #[arg(short = 'i', long)]
    pub image: Option<String>,

    #[arg(short = 'f', long, default_value = "text")]
    pub format: String,

    #[arg(short, long)]
    pub strict: bool,

    /// Test için Issue açılacak repo sahibi (Örn: github_kullanici_adin)
    #[arg(long)]
    pub report_owner: Option<String>,

    /// Test için Issue açılacak repo adı (Örn: test_repo)
    #[arg(long)]
    pub report_repo: Option<String>,
    /// Otomatik Avlanma Modu (Örn: "language:python size:<1000")
    #[arg(long)]
    pub hunt: Option<String>,
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
