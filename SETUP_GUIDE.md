# Step-by-Step Setup Guide

## Prerequisites

1. **Rust installed** - Visit https://rustup.rs/
2. **Telegram account** - For bot control
3. **API Keys:**
   - Gemini API key (for prompt generation)
   - Veo 3.1 API keys (at least 1, up to 15 for fallback)

## Step 1: Get Telegram Bot Token

1. Open Telegram and search for `@BotFather`
2. Send `/newbot` command
3. Follow the instructions to create your bot
4. Copy the token (looks like: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)

## Step 2: Get Gemini API Key

1. Go to https://makersuite.google.com/app/apikey
2. Click "Create API Key"
3. Copy the key

## Step 3: Configure the Project

1. Navigate to the project folder:
   ```bash
   cd veo_pipeline
   ```

2. Edit `config/api_keys.toml`:
   - Replace `YOUR_GEMINI_API_KEY` with your actual Gemini key
   - Replace `VEO_KEY_01`, etc. with your Veo API keys
   - Leave unused keys as empty strings or remove them

3. Set the Telegram token as environment variable:
   
   **Mac/Linux:**
   ```bash
   export TELOXIDE_TOKEN=your_telegram_bot_token
   ```
   
   **Windows (PowerShell):**
   ```powershell
   $env:TELOXIDE_TOKEN="your_telegram_bot_token"
   ```
   
   **Windows (CMD):**
   ```cmd
   set TELOXIDE_TOKEN=your_telegram_bot_token
   ```

## Step 4: Update Veo Endpoint (If Available)

If you have access to Veo 3.1 API:

1. Open `src/main.rs`
2. Find the line with `veo_url` (around line 95)
3. Replace `YOUR_PROJECT` with your actual Google Cloud project ID

## Step 5: Build and Run

```bash
cargo build
cargo run
```

You should see:
```
🚀 Server running on http://localhost:3000
🤖 Telegram bot listening for /idea commands...
Loaded X Veo API keys.
```

## Step 6: Test the Bot

1. Open Telegram on your phone
2. Find your bot (the name you gave it in BotFather)
3. Send: `/idea A cat playing piano in space`
4. You should get an instant reply: `✅ Idea accepted...`

## Step 7: Verify Each Phase

### Check Health Endpoint
Open browser: `http://localhost:3000/health`
Expected: `{"status":"ok"}`

### Check Logs
Watch the terminal for:
- `📝 Prompt generated: ...`
- `Trying Veo API Key #1...`
- `💾 Video saved to videos/...`

### Check Database
1. Download "DB Browser for SQLite"
2. Open `pipeline.db` in the project folder
3. Check the `jobs` table for your request

### Check Video
1. Go to `videos/` folder
2. Find the `.mp4` file
3. Play it with VLC or any media player

## Troubleshooting

### "TELOXIDE_TOKEN env var not set"
- Make sure you set the environment variable in the same terminal where you run `cargo run`

### "At least 1 Veo API key is required!"
- Check `config/api_keys.toml` has at least one non-empty key

### "Config error: No such file or directory"
- Make sure you're running `cargo run` from the `veo_pipeline` folder
- Check that `config/api_keys.toml` exists

### Bot doesn't respond
- Verify the token is correct
- Check your internet connection
- Look at terminal logs for errors

### Video generation fails
- If using dummy keys, this is expected (test the fallback logic)
- If using real keys, check the Veo endpoint URL is correct
- Verify your API keys have proper permissions

## Next Steps

Once everything works:
1. Test with multiple ideas
2. Verify the fallback logic (put a bad key first)
3. Check database persistence (restart server and check old jobs)
4. Prepare for Phase 7: YouTube upload integration
