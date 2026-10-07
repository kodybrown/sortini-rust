#!/usr/bin/env pwsh
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    cargo fmt --all --check
    if ($LASTEXITCODE -ne 0) { throw 'Rust formatting check failed' }
    cargo clippy --locked --all-targets -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Clippy failed' }
    cargo test --locked
    if ($LASTEXITCODE -ne 0) { throw 'Debug tests failed' }
    cargo test --locked --release
    if ($LASTEXITCODE -ne 0) { throw 'Release tests failed' }
    cargo build --locked --release
    if ($LASTEXITCODE -ne 0) { throw 'Release build failed' }
    git diff --check
    if ($LASTEXITCODE -ne 0) { throw 'Git whitespace check failed' }
    git diff --cached --check
    if ($LASTEXITCODE -ne 0) { throw 'Staged Git whitespace check failed' }
} finally {
    Pop-Location
}
