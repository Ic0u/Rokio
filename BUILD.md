# 🛠️ Rokio — Build & Cross-Compile Guide

## 📦 Quick Build Commands

### **Development (macOS — Your System)**
```bash
# Run in dev mode (fast hot-reload)
npm run tauri dev

# Build debug version (faster compile)
npm run tauri build -- --debug
```

### **Local Production Build (macOS Universal)**
```bash
# Universal binary (Apple Silicon + Intel)
npm run tauri build -- --target universal-apple-darwin

# Apple Silicon only (M1/M2/M3)
npm run tauri build -- --target aarch64-apple-darwin

# Intel only
npm run tauri build -- --target x86_64-apple-darwin
```

**Output location:**
```
src-tauri/target/universal-apple-darwin/release/bundle/dmg/Rokio.dmg
```

---

## 🌍 Cross-Platform Builds (GitHub Actions)

You **cannot** native cross-compile Windows/Linux binaries on macOS locally. Use GitHub Actions instead:

### **Method 1: Git Tag (Automatic)**
```bash
# 1. Commit everything
git add .
git commit -m "chore: prepare v1.1.0-alpha release"

# 2. Tag and push
git tag v1.1.0-alpha
git push origin main --tags
```

### **Method 2: Manual Workflow Trigger**
1. Go to: https://github.com/Ic0u/Rokio/actions/workflows/release.yml
2. Click **"Run workflow"**
3. Enter version: `v1.1.0-alpha`
4. Click **"Run workflow"**

**Build outputs:**
- macOS: `Rokio_1.1.0-alpha_universal.dmg`
- Windows: `Rokio_1.1.0-alpha_x64-setup.exe`, `Rokio_1.1.0-alpha_x64_en-US.msi`
- Linux: `rokio_1.1.0-alpha_amd64.deb`, `rokio_1.1.0-alpha_amd64.AppImage`

---

## ⚡ Optimizations Applied

### **macOS (Your Dev Machine)**
- ✅ **Native ld linker** → 25-40% faster linking
- ✅ **`split-debuginfo`** → unpacked debug info for instant linking
- ✅ **Dependency opt-level 2** → faster incremental builds

### **Windows Builds (CI)**
- ✅ **Static CRT** → no runtime dependencies
- ✅ **LLD linker** → faster than MSVC link.exe

### **Linux Builds (CI)**
- ✅ **LLD linker** → faster than GNU ld

### **Release Profile (`release-dist`)**
- ✅ **Fat LTO** → maximum optimization
- ✅ **Size optimization** → smaller binaries
- ✅ **Stripped symbols** → production-ready

---

## 🧪 Pre-Release Checklist

Run these **before** pushing a tag:

```bash
# 1. Type check
npm run check

# 2. Frontend build
npm run build

# 3. Rust check (fast)
cargo check --manifest-path src-tauri/Cargo.toml

# 4. Run tests
cargo test --manifest-path src-tauri/Cargo.toml

# 5. Local macOS build
npm run tauri build -- --target universal-apple-darwin
```

---

## 🐛 Troubleshooting

### "Cannot find linker `lld`" on macOS
Install LLVM:
```bash
brew install llvm
```

### Slow Incremental Builds
Clean and rebuild:
```bash
cargo clean --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

### GitHub Actions Build Fails
Check logs at: https://github.com/Ic0u/Rokio/actions

Common fixes:
- Verify version in `tauri.conf.json` matches tag
- Ensure icons exist in `src-tauri/icons/`
- Check `CHANGELOG.md` is updated

---

## 📊 Build Times (Estimated)

| Platform | Clean Build | Incremental Build |
|----------|-------------|-------------------|
| macOS (dev) | ~3-5 min | **~5-15 sec** ⚡ |
| macOS (release) | ~8-12 min | - |
| Windows (CI) | ~10-15 min | - |
| Linux (CI) | ~6-10 min | - |

---

## 🚀 Release Workflow

```bash
# Full release flow
git checkout main
git pull
npm run check && npm run build
git add .
git commit -m "chore: release v1.1.0-alpha"
git tag v1.1.0-alpha
git push origin main --tags

# Monitor at:
# https://github.com/Ic0u/Rokio/actions
```

**Done!** Binaries will be at: https://github.com/Ic0u/Rokio/releases
