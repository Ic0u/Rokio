# 🚀 Rokio - Cross-Platform Deployment Guide

## Overview

This guide explains how to deploy cross-platform releases for **Rokio** using GitHub Actions. The workflows are already configured to build for:
- **macOS** (Universal Binary: Apple Silicon + Intel)
- **Windows** (`.msi` and `.exe`)
- **Linux** (`.deb` and `.AppImage`)

---

## ✅ Prerequisites

### 1. GitHub Repository
Your repo is already at: `https://github.com/Ic0u/Rokio.git`

### 2. Workflows
Two GitHub Actions workflows are configured:
- `.github/workflows/build.yml` → Builds on every push/PR (no release)
- `.github/workflows/release.yml` → Creates releases with binaries

---

## 🎯 Release Methods

### **Method 1: Git Tag (Recommended)**

This is the standard semantic versioning approach:

```bash
# 1. Update version in tauri.conf.json (already at 1.1.0-alpha)
# 2. Commit all changes
git add .
git commit -m "chore: prepare release v1.1.0-alpha"

# 3. Create and push the tag
git tag v1.1.0-alpha
git push origin main --tags
```

### **Method 2: Manual Workflow Dispatch**

Trigger a release manually from GitHub UI:

1. Go to: https://github.com/Ic0u/Rokio/actions/workflows/release.yml
2. Click **"Run workflow"**
3. Enter version: `v1.1.0-alpha`
4. Click **"Run workflow"**

---

## 📦 What Happens During Release

### Build Matrix
The workflow runs 3 parallel jobs:

| Platform | Runner | Build Output |
|----------|--------|--------------|
| macOS | `macos-latest` | Universal `.dmg` (Intel + ARM) |
| Linux | `ubuntu-24.04` | `.deb`, `.AppImage` |
| Windows | `windows-latest` | `.msi`, `.exe` |

### Artifacts Generated
After successful build, the following will be uploaded to GitHub Releases:

**macOS:**
```
Rokio_1.1.0-alpha_universal.dmg
```

**Windows:**
```
Rokio_1.1.0-alpha_x64_en-US.msi
Rokio_1.1.0-alpha_x64-setup.exe
```

**Linux:**
```
rokio_1.1.0-alpha_amd64.deb
rokio_1.1.0-alpha_amd64.AppImage
```

---

## 🛡️ Pre-Flight Checks (Run Before Release)

### 1. Build Locally
```bash
npm run build
```

### 2. Type Check
```bash
npm run check
```

### 3. Build Tauri (macOS only)
```bash
npm run tauri build -- --target universal-apple-darwin
```

### 4. Test Installation
After local build, test the binary in:
```
src-tauri/target/universal-apple-darwin/release/bundle/dmg/
```

---

## 🚨 Current Version Status

**Current Version:** `1.1.0-alpha`
**Location:** `src-tauri/tauri.conf.json`

Before releasing, you may want to:
- [ ] Review `CHANGELOG.md` (already updated)
- [ ] Test all features work correctly
- [ ] Remove `console.log` statements (if any)
- [ ] Verify no hardcoded secrets or tokens

---

## 📋 Step-by-Step Release Workflow

### Option A: Release via Git Tag

```bash
# 1. Ensure you're on main and up to date
git checkout main
git pull origin main

# 2. Make sure everything builds
npm run check
npm run build

# 3. Update version if needed (already 1.1.0-alpha in tauri.conf.json)
# No changes needed

# 4. Commit pending changes
git add .
git commit -m "chore: finalize v1.1.0-alpha release"

# 5. Create and push tag
git tag v1.1.0-alpha
git push origin main
git push origin v1.1.0-alpha
```

### Option B: Manual GitHub Actions Trigger

1. Visit: https://github.com/Ic0u/Rokio/actions/workflows/release.yml
2. Click **"Run workflow"** dropdown
3. Enter version: `v1.1.0-alpha`
4. Click green **"Run workflow"** button

---

## 🔍 Monitoring the Build

### View Progress
1. Go to: https://github.com/Ic0u/Rokio/actions
2. Click the running workflow
3. Monitor all 3 jobs (macOS, Windows, Linux)

### Expected Duration
- **macOS:** ~8-12 minutes
- **Windows:** ~10-15 minutes
- **Linux:** ~6-10 minutes

---

## ✅ After Release Completes

### 1. Check Draft Release
The workflow creates a **draft release** at:
```
https://github.com/Ic0u/Rokio/releases
```

### 2. Review the Release
- Verify all 5 binaries are attached
- Review the auto-generated release notes
- Test download links

### 3. Publish the Release
- Click **"Edit"** on the draft
- Make any final edits to release notes
- Click **"Publish release"**

---

## 🐛 Troubleshooting

### Build Fails: "Failed to bundle project"
**Cause:** Dependencies missing or outdated

**Fix:**
```bash
npm ci
cargo clean
npm run tauri build
```

### Build Fails: "tauri-action error"
**Cause:** Version mismatch or missing icons

**Fix:**
- Verify icons exist in `src-tauri/icons/`
- Check `tauri.conf.json` version is valid semver

### Linux Build Error: "webkit2gtk not found"
**Cause:** Missing system dependencies

**Fix:** Already handled in workflow (line 44-48):
```yaml
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

---

## 🎨 Customizing Release Notes

Edit `.github/workflows/release.yml` lines 66-77 to customize the release body:

```yaml
releaseBody: |
  ## What's New in v1.1.0-alpha

  ### ⚡ Performance
  - Blazing fast builds (~3s incremental)
  - Optimized linkers for all platforms

  ### 🔧 Improvements
  - Consolidated cross-platform utilities
  - Cleaned up compiler warnings

  See [CHANGELOG.md](CHANGELOG.md) for full details.
```

---

## 📝 Version Bumping Guide

For future releases:

### Patch Release (1.1.1)
```bash
# Update src-tauri/tauri.conf.json: "version": "1.1.1"
git commit -am "chore: bump version to 1.1.1"
git tag v1.1.1
git push origin main --tags
```

### Minor Release (1.2.0)
```bash
# Update src-tauri/tauri.conf.json: "version": "1.2.0"
# Update CHANGELOG.md with new features
git commit -am "chore: bump version to 1.2.0"
git tag v1.2.0
git push origin main --tags
```

### Stable Release (1.1.0)
```bash
# Remove "-alpha" suffix in tauri.conf.json: "version": "1.1.0"
# Update CHANGELOG.md: change [1.1.0-alpha] to [1.1.0]
git commit -am "chore: release stable v1.1.0"
git tag v1.1.0
git push origin main --tags
```

---

## 🔐 Security Considerations

### Secrets Required
The workflow uses:
- `GITHUB_TOKEN` (auto-provided by GitHub Actions)

### No Additional Secrets Needed
Tauri doesn't require code signing for alpha releases. For production:
- **macOS:** You'll need an Apple Developer certificate
- **Windows:** Consider code signing certificate

---

## 📊 Release Checklist

Before publishing, ensure:

- [ ] Version updated in `tauri.conf.json`
- [ ] `CHANGELOG.md` updated with changes
- [ ] All TypeScript errors fixed (`npm run check`)
- [ ] Frontend builds successfully (`npm run build`)
- [ ] No `console.log` or debug statements
- [ ] Icons present in `src-tauri/icons/`
- [ ] README updated if needed
- [ ] All features tested locally

---

## 🚀 Quick Command Reference

```bash
# Local Development
npm run dev                    # Start dev server
npm run tauri dev              # Run Tauri app in dev mode

# Testing
npm run check                  # TypeScript + Svelte check
npm run build                  # Build frontend

# Building (Mac only)
npm run tauri build -- --target universal-apple-darwin

# Release
git tag v1.1.0-alpha && git push origin v1.1.0-alpha
```

---

## 📞 Support

If you encounter issues:
1. Check GitHub Actions logs: https://github.com/Ic0u/Rokio/actions
2. Review Tauri docs: https://tauri.app/
3. Check this repository's Issues tab

---

**Ready to Deploy?** 🎯

Run the command below to trigger your first cross-platform release:

```bash
git tag v1.1.0-alpha && git push origin main --tags
```

Then visit: https://github.com/Ic0u/Rokio/actions to monitor the build!
