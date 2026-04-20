# Quick Start Guide

## 🚀 Running the Pipeline

### Step 1: Set Your Telegram Bot Token

```powershell
$env:TELOXIDE_TOKEN="your_telegram_bot_token_here"
```

### Step 2: Configure API Keys

Edit `config/api_keys.toml`:
- Add your Gemini API key
- Add your Veo 3.1 API keys (at least 1)

### Step 3: Run the Application

**Option A: Using the script**
```powershell
.\run.ps1
```

**Option B: Direct command**
```powershell
cargo run --release
```

## 📊 What You'll See

### Console Output
```
╔═══════════════════════════════════════════════════════════╗
║         VEO PIPELINE - NEW SESSION STARTED               ║
║         2026-04-20 22:22:47                              ║
╚═══════════════════════════════════════════════════════════╝

═══════════════════════════════════════════════════════════
🚀 [STARTUP] Initializing Veo Pipeline System
═══════════════════════════════════════════════════════════
🔄 [CONFIG] Loading configuration from config/api_keys.toml...
✅ [CONFIG] Loaded 3 Veo API keys (keys masked for security)
✅ [CONFIG] Prompt API key loaded
🔄 [DATABASE] Initializing SQLite database...
✅ [DATABASE] Connected to pipeline.db
✅ [DATABASE] Jobs table ready
🔄 [STARTUP] Initializing Telegram bot...
✅ [STARTUP] Telegram bot initialized
═══════════════════════════════════════════════════════════
✅ [STARTUP] ALL SYSTEMS OPERATIONAL
═══════════════════════════════════════════════════════════
🚀 Server: http://localhost:3000/health
🤖 Telegram: Listening for /idea commands
📝 Logs: logs/pipeline.log
═══════════════════════════════════════════════════════════
```

### When You Send `/idea A cat in space`

```
📨 [TELEGRAM] New /idea command received from chat 123456789
📝 [TELEGRAM] Idea: 'A cat in space'
✅ [TELEGRAM] Confirmation sent to user
🔄 [TELEGRAM] Spawning background pipeline task...
✅ [TELEGRAM] Background task spawned

═══════════════════════════════════════════════════════════
🚀 [PIPELINE] NEW JOB STARTED
═══════════════════════════════════════════════════════════
📝 [PIPELINE] Idea: 'A cat in space'
👤 [PIPELINE] Chat ID: 123456789
🔄 [DATABASE] Creating new job entry...
✅ [DATABASE] Job #1 created with status: PROMPTING

───────────────────────────────────────────────────────────
📝 [PIPELINE] STEP 1: PROMPT GENERATION
───────────────────────────────────────────────────────────
🔄 [PROMPT] Starting prompt generation for idea: 'A cat in space'
🔄 [PROMPT] Sending request to Gemini API...
✅ [PROMPT] Received response from Gemini API (Status: 200)
✅ [PROMPT] Successfully generated prompt: 'A cinematic 8-second shot...'
🔄 [DATABASE] Updating job #1 status to GENERATING
✅ [DATABASE] Job #1 updated with generated prompt

───────────────────────────────────────────────────────────
🎬 [PIPELINE] STEP 2: VIDEO GENERATION
───────────────────────────────────────────────────────────
🔄 [VEO] Starting video generation with 3 API keys available
🔑 [VEO] Attempting with API Key #1/3
🔄 [VEO] Key #1: Preparing request payload...
🔄 [VEO] Key #1: Sending request to Veo 3.1 API (timeout: 90s)...
✅ [VEO] Key #1: Request successful! Status: 200
✅ [VEO] Key #1: Video URL extracted: https://...
🎉 [VEO] Video generation completed successfully with Key #1

───────────────────────────────────────────────────────────
💾 [PIPELINE] STEP 3: VIDEO DOWNLOAD
───────────────────────────────────────────────────────────
🔄 [DOWNLOAD] Starting video download for Job #1
🔄 [DOWNLOAD] Sending GET request to video URL...
✅ [DOWNLOAD] Response received (Status: 200)
🔄 [DOWNLOAD] Downloading video bytes (streaming)...
✅ [DOWNLOAD] Download complete! Size: 5242880 bytes (5.00 MB)
🔄 [DOWNLOAD] Writing video to disk: videos/video_job_1.mp4
✅ [DOWNLOAD] Video saved successfully!
💾 [DOWNLOAD] File: videos/video_job_1.mp4 | Size: 5242880 bytes

═══════════════════════════════════════════════════════════
🎉 [PIPELINE] JOB #1 COMPLETED SUCCESSFULLY!
═══════════════════════════════════════════════════════════
```

## 📝 Log Files

All logs are saved to `logs/pipeline.log`

Each time you restart the application:
- Old logs are archived to `logs/pipeline_archive_YYYYMMDD_HHMMSS.log`
- New logs start fresh at the top of `pipeline.log`

## 🧪 Testing

1. **Test Health Endpoint:**
   ```powershell
   curl http://localhost:3000/health
   ```
   Expected: `{"status":"ok"}`

2. **Test Telegram Bot:**
   - Open Telegram on your phone
   - Send: `/idea A futuristic city at sunset`
   - Watch the console for detailed logs

3. **Check Database:**
   - Open `pipeline.db` with DB Browser for SQLite
   - View the `jobs` table

## 🐛 Troubleshooting

### "TELOXIDE_TOKEN env var not set"
Set the token in your current PowerShell session:
```powershell
$env:TELOXIDE_TOKEN="your_token"
```

### "At least 1 Veo API key is required!"
Edit `config/api_keys.toml` and add at least one valid key.

### Bot doesn't respond
- Check your internet connection
- Verify the bot token is correct
- Look at the console logs for errors

### All API keys fail
- If using dummy keys, this is expected (tests the fallback logic)
- If using real keys, check the Veo endpoint URL in `src/main.rs`
