#  Vault Hound 

**Cloud-Native Secret & Hardcoded Credential Scanner**

Vault Hound is a high-performance DevSecOps security engine built from scratch in Rust. It is designed to detect sensitive data (API Keys, Database Passwords, Private Keys, etc.) accidentally left behind in software development processes (CI/CD pipelines) and public GitHub repositories.

##  Key Architectural Features

* **In-Memory Layer Scanning:** Eliminates Disk I/O bottlenecks by scanning Docker images (`.tar`) or GitHub repositories entirely in-memory as a stream, without extracting them to disk. Highly SSD friendly.
* **Shannon Entropy Analysis:** Goes beyond known Regex patterns and uses mathematical entropy calculations to catch randomly generated, complex secrets hidden within the code.
* **Smart Discovery (Hunter Mode):** Automatically discovers and scans new repositories based on specific criteria using the GitHub Code Search API.
* **Autonomous Memory (State Persistence):** Saves scanned repositories to a built-in **SQLite** database. It protects system resources and prevents exceeding GitHub API Rate Limits by ensuring the same repository is never scanned twice.
* **Noise Filtering (Allowlist):** Focuses exclusively on source code (`.py`, `.js`, `.json`, `.env`, etc.); smartly bypasses static files like `.lock`, `.svg`, `.md` that typically generate false positives.

##  Installation & Usage

### Running with Docker (Recommended)
You can run the scanner anywhere with a single command, without needing to install Rust on your system.

```bash
docker build -t vault_hound .
docker run -v $(pwd):/scan vault_hound --path /scan
docker run --env-file .env vault_hound --hunt "language:python size:<1000"
```
### CI/CD Pipeline Integration (Shift-Left Security)
Vault Hound is built to break CI/CD pipelines when necessary. When executed with the `--strict` flag on GitHub Actions or GitLab CI, it will return `Exit Code 1` if a leaked secret is detected, actively preventing vulnerable code from reaching the Production environment.

```yaml
- name: Run Vault Hound Scanner
  run: ./vault_hound --path . --strict
```
##  How it Works

1. **Regex & Pattern Matching:** Catches predefined key formats of well-known service providers (AWS, Google Cloud, Stripe, Slack, etc.).
2. **Shannon Entropy (Randomness Measurement):** Calculates the entropy of strings longer than 16 characters. Strings with an entropy score higher than 4.5 are flagged as highly suspicious.
3. **Smart Extension Filter:** Automatically filters out static media and dependency (lock) files from the scanning stream to drastically improve performance and eliminate noise.

---
*Developed with  Rust & a Cloud-Native Security mindset.*