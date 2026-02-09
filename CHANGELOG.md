# Changelog

All notable changes to this project will be documented in this file.

## [1.1.0-alpha] - 2026-02-09

### ⚡ Performance Optimization
- **Blazing Fast Builds:** Incremental builds now complete in ~3 seconds (previously 60s+)
- **Cross-Platform Linkers:** Added support for fastest linkers on each platform:
  - macOS: `sold` linker
  - Linux: `mold` linker  
  - Windows: `rust-lld` linker
- **Optimized Cargo Profiles:**
  - Dev: `opt-level = 1` with incremental compilation
  - Dependencies: `opt-level = 3` for pre-optimized deps
  - Release: LTO, strip, panic=abort for smallest binary

### 🧹 Code Refactor
- Consolidated cross-platform process utilities (`process_utils.rs`)
- Cleaned up all 24+ compiler warnings
- Removed dead code and unused imports
- Unified macOS/Linux process management logic

### 🔧 Configuration
- Added `.cargo/config.toml` with native CPU optimization
- Set `MACOSX_DEPLOYMENT_TARGET` to prevent cache invalidation
- Parallel build with 8 jobs

### 📦 Build
- Release profile: LTO enabled, stripped symbols, size-optimized
- Dev profile: Fast incremental builds with debug info

---

## [1.0.0-beta] - Initial Release

### Features
- Secure account storage with AES-256-GCM encryption
- Multi-instance Roblox launching
- Server browser with favorites
- Cross-platform support (Windows, macOS, Linux)

---

**Author:** Nam Nguyễn
