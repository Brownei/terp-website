# Terp Network Website

Official website for Terp Network, featuring the terp-core installer integration.

## Features

- Responsive dark-themed landing page with animated backgrounds
- Integrated terp-core installer with tab-based installation options
- Docker-ready deployment
- SEO optimized with sitemap and meta tags

## Development

```sh
python3 -m http.server 8000
```

## Installation Scripts

### Quick Install (Interactive)

**One-line install:**
```bash
curl -fsSL https://terp.network/get | bash
```

The installer will automatically:
1. Check/install Python 3.6+
2. Guide you through selecting installation type (node/client/localterp)
3. Help you choose network (mainnet/testnet)
4. Configure your node settings
5. Optionally install cosmovisor and systemd service

### Command-Line Options

You can also use flags to skip certain prompts:

```bash
curl -fsSL https://terp.network/get | bash -s -- --install node --network morocco-1 --moniker "my-node"
```

**Available flags:**
- `--install <node|client|localterp>` - Installation type
- `--network <morocco-1|90u-4>` - Network to join
- `--home <path>` - Installation directory (default: ~/.terp)
- `--moniker <name>` - Node moniker (default: terp)
- `--pruning <default|nothing|everything>` - Pruning settings
- `--cosmovisor` - Install with cosmovisor
- `--service` - Setup systemd service (Linux only)
- `--overwrite` - Overwrite existing installation

### Alternative: UV Tool

```bash
uvx --from terp-core terpd
```

### Verifying Installation Script Integrity

Before running the installation scripts, verify their integrity using checksums:

#### SHA256 Checksums

```
0e2743c117a3be8e5648427e0e1d8863b7ac59e1e1cff428152eda99cf9dc970  terp-installer.py
a233f0863b439273e772b14d61b985c8a20e719c72506399adebff03551596c7  terp-installer.sh
```

#### BLAKE3 Checksums

```
3113805970499a614c8dda2b8d2730bade6f0b0a3d5a8fa99bac4e9856396cee  terp-installer.py
8c1826931f3c9c620dddabe6756881a2a51aa977c24b60842eca697dfd40ebb7  terp-installer.sh
```

#### Verification Instructions

**Using SHA256:**

```bash
# Download the shell script
curl -sL https://terp.network/get > terp-installer.sh

# Verify with sha256sum (Linux)
echo "a233f0863b439273e772b14d61b985c8a20e719c72506399adebff03551596c7  terp-installer.sh" | sha256sum -c

# Verify with shasum (macOS)
echo "a233f0863b439273e772b14d61b985c8a20e719c72506399adebff03551596c7  terp-installer.sh" | shasum -a 256 -c
```

**Using BLAKE3:**

```bash
# Download the shell script
curl -sL https://terp.network/get > terp-installer.sh

# Install b3sum if not already installed
# macOS: brew install b3sum
# Linux: cargo install b3sum
# or download from: https://github.com/BLAKE3-team/BLAKE3

# Verify with b3sum
echo "8c1826931f3c9c620dddabe6756881a2a51aa977c24b60842eca697dfd40ebb7  terp-installer.sh" | b3sum --check
```

**Expected output on successful verification:**

```
terp-installer.sh: OK
```

⚠️ **Security Note:** Always verify checksums from multiple trusted sources (GitHub releases, official documentation, etc.) to ensure the checksums themselves haven't been tampered with.

## Running Locally

### Using Docker

Build and run the Docker container from repo root:

```bash
docker-compose -f docker/docker-compose.yml up --build
```

Or from the docker directory:

```bash
cd docker && docker-compose up --build
```

The website will be available at `http://localhost:8080`

### Manual Build

Build the Docker image manually for single architecture:

```bash
docker build -f docker/Dockerfile -t terpnetwork/terp-website:latest .
```

Build for multiple architectures (amd64 and arm64):

```bash
# Create a builder instance (first time only)
docker buildx create --name multiarch --use

# Build and push multi-architecture image
docker buildx build -f docker/Dockerfile \
  --platform linux/amd64,linux/arm64 \
  -t terpnetwork/terp-website:latest \
  --push .

# Or build without pushing (loads single arch to local)
docker buildx build -f docker/Dockerfile \
  --platform linux/amd64,linux/arm64 \
  -t terpnetwork/terp-website:latest \
  --load .
```

Run the container:

```bash
docker run -p 8080:80 terpnetwork/terp-website:latest
```

## Project Structure

```
terp.network/
├── index.html              # Main website file
├── robots.txt              # SEO robots file
├── README.md               # This file
├── public/                 # Public assets
│   ├── favicon/           # Favicon files
│   ├── sitemap.xml        # SEO sitemap
│   └── site.webmanifest   # PWA manifest
├── get/                    # Installation scripts
│   ├── terp-installer.py  # Python installer script
│   └── terp-installer.sh  # Shell installer script
└── docker/                 # Docker deployment files
    ├── Dockerfile         # Docker image configuration
    ├── docker-compose.yml # Docker Compose setup
    ├── nginx.conf         # NGINX server configuration
    ├── entrypoint.sh      # Container entrypoint script
    └── deploy.yaml        # Akash deployment manifest
```

## Deployment

### Docker Registry

Push single architecture to Docker registry:

```bash
docker-compose -f docker/docker-compose.yml build
docker push terpnetwork/terp-website:latest
```

Push multi-architecture image to Docker registry:

```bash
# Build and push for both amd64 and arm64
docker buildx build -f docker/Dockerfile \
  --platform linux/amd64,linux/arm64 \
  -t terpnetwork/terp-website:latest \
  --push .
```

### Akash Network

Deploy to Akash using the provided SDL:

```bash
akash tx deployment create docker/deploy.yaml --from <your-wallet>
```

See `docker/deploy.yaml` for the deployment manifest.

## Installation Script Endpoints

- `/get` - Shell wrapper script (downloads and runs Python installer)
- `/run` - Python installer script (main installation logic)
- `/get/` - Directory access for individual files and checksum verification

## Development

The website uses:

- Pure HTML/CSS/JavaScript (no build tools required)
- Satoshi font from CDN Fonts
- SVG graphics for icons and logos
- Canvas API for animated background effects

## Links

- Documentation: <https://docs.terp.network>
- GitHub: <https://github.com/terpnetwork>
- Twitter: <https://x.com/terpnetofficial>
- Discord: <https://discord.gg/W3QnHe77S6>

## License

See the main Terp Network repository for licensing information.
