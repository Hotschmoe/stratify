# Hosting and Release Setup

This document covers setting up automated builds, releases, and WASM hosting using GitHub (free for public repositories).

## Overview

| Component | Service | Cost |
|-----------|---------|------|
| Multi-platform builds | GitHub Actions | Free |
| Binary downloads | GitHub Releases | Free |
| WASM hosting | GitHub Pages | Free |
| Custom domain | GitHub Pages DNS | Free (you own domain) |

## GitHub Actions Workflow

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags: ['v*']

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    name: Build ${{ matrix.target }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: calc_gui
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: calc_gui.exe
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: calc_gui
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: calc_gui

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build release binary
        run: cargo build --release --bin calc_gui --target ${{ matrix.target }}

      - name: Package binary (Unix)
        if: runner.os != 'Windows'
        run: |
          cd target/${{ matrix.target }}/release
          tar -czvf ../../../calc_gui-${{ matrix.target }}.tar.gz ${{ matrix.artifact }}

      - name: Package binary (Windows)
        if: runner.os == 'Windows'
        run: |
          cd target/${{ matrix.target }}/release
          7z a ../../../calc_gui-${{ matrix.target }}.zip ${{ matrix.artifact }}

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: calc_gui-${{ matrix.target }}
          path: calc_gui-${{ matrix.target }}.*

  build-wasm:
    name: Build WASM
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install Trunk
        uses: jetli/trunk-action@v0.5.0

      - name: Build WASM
        run: |
          cd calc_gui
          trunk build --release --public-url /${{ github.event.repository.name }}/

      - name: Upload WASM artifact
        uses: actions/upload-artifact@v4
        with:
          name: wasm-dist
          path: calc_gui/dist

  release:
    name: Create Release
    needs: [build, build-wasm]
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Display structure
        run: ls -R artifacts

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: |
            artifacts/calc_gui-*/*
          generate_release_notes: true

  deploy-pages:
    name: Deploy to GitHub Pages
    needs: build-wasm
    runs-on: ubuntu-latest
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Download WASM artifact
        uses: actions/download-artifact@v4
        with:
          name: wasm-dist
          path: dist

      - name: Setup Pages
        uses: actions/configure-pages@v4

      - name: Upload Pages artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: dist

      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

## Repository Setup

### 1. Enable GitHub Pages

1. Go to repository **Settings** > **Pages**
2. Under "Build and deployment", select **GitHub Actions** as the source
3. Save

### 2. Create a Release

Tag a version to trigger the workflow:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The workflow will:
1. Build binaries for Linux, Windows, macOS (x64 + ARM)
2. Build WASM for browser
3. Create a GitHub Release with all binaries
4. Deploy WASM to GitHub Pages

### 3. Access Points

After the first release:

- **Releases**: `https://github.com/YOUR_USERNAME/YOUR_REPO/releases`
- **WASM App**: `https://YOUR_USERNAME.github.io/YOUR_REPO/`
- **Latest Release API**: `https://api.github.com/repos/YOUR_USERNAME/YOUR_REPO/releases/latest`

## Custom Domain (Optional)

To use a custom domain with GitHub Pages:

1. Add a `CNAME` file to your `calc_gui/` directory containing your domain:
   ```
   app.yourdomain.com
   ```

2. Configure DNS:
   - For apex domain: A records pointing to GitHub's IPs
   - For subdomain: CNAME record pointing to `YOUR_USERNAME.github.io`

3. In repository Settings > Pages, enter your custom domain

See [GitHub Pages custom domain docs](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site) for details.

## Update Checker

The application includes a built-in update checker that queries GitHub Releases API.

### How It Works

1. App checks `https://api.github.com/repos/OWNER/REPO/releases/latest`
2. Compares remote version tag against compiled-in version
3. If newer version available, shows notification with download link

### Manual Check

Users can trigger a check via the application menu or by running:

```bash
calc_gui --check-update
```

### Automatic Check

On startup, the app performs a background check (non-blocking). Users are notified if an update is available but never forced to update.

### Rate Limits

GitHub API allows 60 requests/hour for unauthenticated requests. The app caches the last check result for 1 hour to stay well within limits.

## Self-Hosting Alternative

If you prefer to self-host instead of using GitHub Pages:

### Docker (WASM)

```dockerfile
FROM nginx:alpine
COPY calc_gui/dist /usr/share/nginx/html
EXPOSE 80
```

Build and run:

```bash
cd calc_gui && trunk build --release
docker build -t calc-gui-wasm .
docker run -p 8080:80 calc-gui-wasm
```

### Static File Server

Any static file server works. The `dist/` folder contains:
- `index.html`
- `*.wasm` - WebAssembly binary
- `*.js` - JavaScript glue code

No server-side processing required.
