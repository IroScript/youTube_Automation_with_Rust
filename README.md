# Veo 3.1 Automated Video Pipeline

An automated video generation pipeline that converts ideas into videos using AI.

## Features

- 📱 Mobile control via Telegram bot
- 🤖 AI-powered prompt generation (Gemini)
- 🎬 Video generation with Veo 3.1 API (15-key fallback)
- 💾 SQLite state persistence
- ⚡ Background processing (fire-and-forget)
- 🏥 Health check endpoint

## Project Structure

```
veo_pipeline/
├── src/
│   └── main.rs           # Main application code
├── config/
│   └── api_keys.toml     # API keys configuration
├── videos/               # Downloaded videos (created at runtime)
├── Cargo.toml            # Rust dependencies
├── pipeline.db           # SQLite database (created at runtime)
└── README.md             # This file
```

## Setup

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Configure API Keys
Edit `config/api_keys.toml` and add your keys:
- Veo 3.1 API keys (at least 1)
- Gemini API key for prompt generation

### 3. Set Telegram Bot Token
Get a token from [@BotFather](https://t.me/BotFather) on Telegram.

**Mac/Linux:**
```bash
export TELOXIDE_TOKEN=your_telegram_bot_token
```

**Windows (PowerShell):**
```powershell
$env:TELOXIDE_TOKEN="your_telegram_bot_token"
```

### 4. Update Veo Endpoint
In `src/main.rs`, update the `veo_url` variable with your actual Veo 3.1 endpoint.

### 5. Run the Project
```bash
cargo run
```

## Usage

1. Open Telegram on your mobile
2. Send a message to your bot: `/idea A futuristic city at sunset`
3. The bot will reply instantly confirming your idea
4. Wait for the background processing to complete
5. The bot will notify you when the video is ready

## Manual Testing Checklist

### Phase 1-2: Server & Config
- [ ] Terminal prints `Loaded X Veo API keys.`
- [ ] Browser at `http://localhost:3000/health` shows `{"status":"ok"}`

### Phase 3-4: Telegram & AI Prompt
- [ ] Bot replies instantly: `✅ Idea accepted...`
- [ ] Terminal prints `📝 Prompt generated: ...`

### Phase 5-6: Veo Fallback & Download
- [ ] Terminal logs key fallback attempts
- [ ] Video file appears in `videos/` folder
- [ ] Video plays correctly in media player

### Phase 8: SQLite State
- [ ] Open `pipeline.db` with DB Browser for SQLite
- [ ] Check `jobs` table has correct data and status

## Next Steps

- **Phase 7:** YouTube OAuth2 integration (coming soon)
- **Phase 10:** Cloud deployment

## Tech Stack

- **Language:** Rust
- **Web Framework:** Axum
- **Async Runtime:** Tokio
- **Bot Framework:** Teloxide
- **Database:** SQLite (rusqlite)
- **HTTP Client:** reqwest
- **AI:** Google Gemini
- **Video:** Veo 3.1

## License

MIT
