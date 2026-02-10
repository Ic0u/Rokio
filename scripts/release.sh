#!/bin/bash

# Rokio Release Script
# Usage: ./scripts/release.sh [version]
# Example: ./scripts/release.sh 1.1.0-alpha

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

VERSION="${1}"

print_header() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  🚀 Rokio Release Script${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_step() {
    echo -e "\n${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "\n${RED}✗${NC} $1"
}

print_warning() {
    echo -e "\n${YELLOW}⚠${NC} $1"
}

# Check if version provided
if [ -z "$VERSION" ]; then
    print_header
    print_error "No version specified"
    echo ""
    echo "Usage: ./scripts/release.sh [version]"
    echo "Example: ./scripts/release.sh 1.1.0-alpha"
    echo ""
    echo "Current version in tauri.conf.json:"
    grep '"version"' src-tauri/tauri.conf.json | head -1
    exit 1
fi

# Add 'v' prefix if not present
if [[ ! "$VERSION" =~ ^v ]]; then
    VERSION="v${VERSION}"
fi

print_header
echo ""
echo "📦 Preparing release: ${GREEN}${VERSION}${NC}"
echo ""

# Step 1: Check git status
print_step "Checking git status..."
if ! git diff-index --quiet HEAD --; then
    print_warning "You have uncommitted changes. Please commit or stash them first."
    git status -s
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Step 2: Ensure we're on main
BRANCH=$(git branch --show-current)
if [ "$BRANCH" != "main" ]; then
    print_warning "You are on branch '${BRANCH}', not 'main'"
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Step 3: Pull latest
print_step "Pulling latest changes..."
git pull origin "$BRANCH"

# Step 4: Run checks
print_step "Running TypeScript checks..."
npm run check

# Step 5: Build frontend
print_step "Building frontend..."
npm run build

# Step 6: Check if tag exists
if git rev-parse "$VERSION" >/dev/null 2>&1; then
    print_error "Tag ${VERSION} already exists!"
    echo ""
    echo "Options:"
    echo "  1. Delete existing tag: git tag -d ${VERSION} && git push origin :refs/tags/${VERSION}"
    echo "  2. Use a different version number"
    exit 1
fi

# Step 7: Confirmation
echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Ready to release:${NC} ${GREEN}${VERSION}${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "This will:"
echo "  1. Create git tag: ${VERSION}"
echo "  2. Push to GitHub"
echo "  3. Trigger GitHub Actions workflow"
echo "  4. Build for macOS, Windows, Linux"
echo "  5. Create a draft GitHub release"
echo ""
read -p "Proceed? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_warning "Release cancelled"
    exit 0
fi

# Step 8: Create and push tag
print_step "Creating tag ${VERSION}..."
git tag "$VERSION"

print_step "Pushing tag to GitHub..."
git push origin "$VERSION"

# Step 9: Success
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  ✅ Release triggered successfully!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Monitor build progress:"
echo -e "  ${BLUE}https://github.com/Ic0u/Rokio/actions${NC}"
echo ""
echo "After build completes, review draft release:"
echo -e "  ${BLUE}https://github.com/Ic0u/Rokio/releases${NC}"
echo ""
print_step "Done! 🚀"
