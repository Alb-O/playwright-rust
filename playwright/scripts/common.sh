#!/usr/bin/env bash
# Common utilities for playwright scripts
# Source this file: source "$(dirname "$0")/common.sh"

set -euo pipefail

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_success() { echo -e "${GREEN}[OK]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

# Find project root (directory containing playwright.config.*)
find_project_root() {
  local dir="$PWD"
  while [[ "$dir" != "/" ]]; do
    if [[ -f "$dir/playwright.config.js" ]] || [[ -f "$dir/playwright.config.ts" ]]; then
      echo "$dir"
      return 0
    fi
    dir="$(dirname "$dir")"
  done
  log_error "Could not find playwright.config.js or playwright.config.ts"
  return 1
}

# Get playwright directory path
get_playwright_dir() {
  local root
  root="$(find_project_root)"
  echo "$root/playwright"
}

# Ensure we're in project root
ensure_project_root() {
  local root
  root="$(find_project_root)" || exit 1
  cd "$root"
}
