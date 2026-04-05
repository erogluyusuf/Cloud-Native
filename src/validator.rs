use reqwest::Client;

pub async fn is_valid_gemini(key: &str) -> bool {
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", key);
    let client = Client::new();
    let res = client.get(&url).send().await;
    
    match res {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

pub async fn is_valid_github(key: &str) -> bool {
    let client = Client::new();
    let res = client.get("https://api.github.com/user")
        .header("Authorization", format!("token {}", key))
        .header("User-Agent", "vault_hound")
        .send().await;

    match res {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

pub async fn is_valid_discord(key: &str) -> bool {
    let client = Client::new();
    let res = client.get("https://discord.com/api/v9/users/@me")
        .header("Authorization", key)
        .header("User-Agent", "vault_hound")
        .send().await;

    match res {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}
