# Veo Pipeline Startup Script

Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "  VEO PIPELINE - STARTUP SCRIPT" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

# Check if TELOXIDE_TOKEN is set
if (-not $env:TELOXIDE_TOKEN) {
    Write-Host "❌ ERROR: TELOXIDE_TOKEN environment variable is not set!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please set your Telegram bot token:" -ForegroundColor Yellow
    Write-Host '  $env:TELOXIDE_TOKEN="your_bot_token_here"' -ForegroundColor Yellow
    Write-Host ""
    exit 1
}

Write-Host "✅ Telegram token found" -ForegroundColor Green
Write-Host "🚀 Starting Veo Pipeline..." -ForegroundColor Green
Write-Host ""

# Run the application
cargo run --release
