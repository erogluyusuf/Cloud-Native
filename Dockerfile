# Aşama 1: Derleme (Builder)
FROM rust:1.76-slim-bookworm AS builder

# Gerekli C ve SQLite derleme kütüphanelerini kur
RUN apt-get update && apt-get install -y pkg-config libssl-dev clang libsqlite3-dev

# Çalışma dizinini ayarla
WORKDIR /usr/src/vault_hound

# Kodları kopyala ve Release modunda derle
COPY . .
RUN cargo build --release

# Aşama 2: Çalıştırma (Runtime)
FROM debian:bookworm-slim

# HTTPS istekleri (GitHub API) ve SQLite için gerekli minimum paketleri kur
RUN apt-get update && apt-get install -y ca-certificates libsqlite3-0 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Derlenen motoru Builder aşamasından al
COPY --from=builder /usr/src/vault_hound/target/release/vault_hound /usr/local/bin/vault_hound

# Çevre değişkenleri için .env dosyası (Docker run ile de verilebilir)
# COPY .env .env 

# Varsayılan komut (Yardım menüsünü gösterir)
ENTRYPOINT ["vault_hound"]
CMD ["--help"]
