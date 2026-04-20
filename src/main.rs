use axum::{routing::get, Router};
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::Mutex;
use tracing::{error, info, warn, debug};

mod logging;

// ==========================================
// 1. CONFIG & API KEYS
// ==========================================
#[derive(Debug, Deserialize)]
struct AppConfig {
    veo_api_keys: Vec<String>,
    prompt_api_key: String,
}

impl AppConfig {
    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        info!("🔄 [CONFIG] Loading configuration from config/api_keys.toml...");
        let contents = std::fs::read_to_string("config/api_keys.toml")?;
        let config: AppConfig = toml::from_str(&contents)?;
        
        // Filter out empty strings
        let valid_keys: Vec<String> = config.veo_api_keys.into_iter()
            .filter(|k| !k.trim().is_empty())
            .collect();

        if valid_keys.is_empty() {
            error!("❌ [CONFIG] No valid Veo API keys found!");
            return Err("At least 1 Veo API key is required!".into());
        }

        info!("✅ [CONFIG] Loaded {} Veo API keys (keys masked for security)", valid_keys.len());
        info!("✅ [CONFIG] Prompt API key loaded");
        Ok(AppConfig {
            veo_api_keys: valid_keys,
            prompt_api_key: config.prompt_api_key,
        })
    }
}

// ==========================================
// 2. DATABASE (SQLite)
// ==========================================
fn init_db() -> Result<Connection, rusqlite::Error> {
    info!("🔄 [DATABASE] Initializing SQLite database...");
    let conn = Connection::open("pipeline.db")?;
    info!("✅ [DATABASE] Connected to pipeline.db");
    
    info!("🔄 [DATABASE] Creating jobs table if not exists...");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            idea TEXT NOT NULL,
            prompt TEXT,
            status TEXT NOT NULL,
            video_path TEXT,
            created_at TEXT NOT NULL
        )",
        [],
    )?;
    info!("✅ [DATABASE] Jobs table ready");
    Ok(conn)
}

// ==========================================
// 3. PROMPT GENERATOR (AI)
// ==========================================
async fn generate_prompt(api_key: &str, idea: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!("🔄 [PROMPT] Starting prompt generation for idea: '{}'", idea);
    
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
        api_key
    );

    info!("🔄 [PROMPT] Sending request to Gemini API...");
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": format!("Create a detailed video generation prompt for Veo 3.1 about: {}. The prompt should be 1-2 sentences, cinematic, 8 seconds long. Only return the prompt.", idea)
            }]
        }]
    });

    debug!("🔄 [PROMPT] Request payload: {}", serde_json::to_string_pretty(&payload).unwrap_or_default());

    let res = client.post(&url).json(&payload).send().await?;
    info!("✅ [PROMPT] Received response from Gemini API (Status: {})", res.status());
    
    let json: serde_json::Value = res.json().await?;
    debug!("🔄 [PROMPT] Response JSON: {}", serde_json::to_string_pretty(&json).unwrap_or_default());

    let prompt = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .ok_or("Failed to parse prompt from AI response")?
        .to_string();

    info!("✅ [PROMPT] Successfully generated prompt: '{}'", prompt);
    Ok(prompt)
}

// ==========================================
// 4. VIDEO GENERATOR (Veo Fallback)
// ==========================================
async fn generate_video_with_fallback(
    client: &reqwest::Client,
    api_keys: &[String],
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!("🔄 [VEO] Starting video generation with {} API keys available", api_keys.len());
    info!("🔄 [VEO] Prompt to use: '{}'", prompt);
    
    // Replace with your actual Veo 3.1 endpoint
    let veo_url = "https://us-central1-aiplatform.googleapis.com/v1/projects/YOUR_PROJECT/locations/us-central1/publishers/google/models/veo-3.1:predict";

    for (index, key) in api_keys.iter().enumerate() {
        let key_num = index + 1;
        info!("🔑 [VEO] Attempting with API Key #{}/{}", key_num, api_keys.len());
        info!("🔄 [VEO] Key #{}: Preparing request payload...", key_num);
        
        let payload = serde_json::json!({
            "instances": [{ "prompt": prompt, "duration": 8, "resolution": "1080p" }],
            "parameters": { "sampleCount": 1 }
        });

        debug!("🔄 [VEO] Key #{}: Request payload: {}", key_num, serde_json::to_string_pretty(&payload).unwrap_or_default());
        info!("🔄 [VEO] Key #{}: Sending request to Veo 3.1 API (timeout: 90s)...", key_num);

        let res = client
            .post(veo_url)
            .header("Authorization", format!("Bearer {}", key))
            .json(&payload)
            .timeout(std::time::Duration::from_secs(90))
            .send()
            .await;

        match res {
            Ok(response) if response.status().is_success() => {
                info!("✅ [VEO] Key #{}: Request successful! Status: {}", key_num, response.status());
                let json: serde_json::Value = response.json().await?;
                debug!("🔄 [VEO] Key #{}: Response JSON: {}", key_num, serde_json::to_string_pretty(&json).unwrap_or_default());
                
                if let Some(video_url) = json["predictions"][0]["video_url"].as_str() {
                    info!("✅ [VEO] Key #{}: Video URL extracted: {}", key_num, video_url);
                    info!("🎉 [VEO] Video generation completed successfully with Key #{}", key_num);
                    return Ok(video_url.to_string());
                } else {
                    warn!("⚠️ [VEO] Key #{}: Response successful but no video_url found in JSON", key_num);
                    warn!("⚠️ [VEO] Key #{}: Skipping to next key...", key_num);
                }
            }
            Ok(response) => {
                let status = response.status();
                warn!("❌ [VEO] Key #{}: Request failed with HTTP status: {}", key_num, status);
                if let Ok(body) = response.text().await {
                    warn!("❌ [VEO] Key #{}: Error response body: {}", key_num, body);
                }
                if key_num < api_keys.len() {
                    info!("🔄 [VEO] Key #{}: Moving to next API key...", key_num);
                }
            }
            Err(e) => {
                warn!("❌ [VEO] Key #{}: Network/timeout error: {}", key_num, e);
                if key_num < api_keys.len() {
                    info!("🔄 [VEO] Key #{}: Moving to next API key...", key_num);
                }
            }
        }
    }

    error!("❌ [VEO] All {} API keys exhausted. Video generation failed.", api_keys.len());
    Err("All Veo API keys failed or exhausted.".into())
}

// ==========================================
// 5. VIDEO DOWNLOADER & VERIFIER
// ==========================================
async fn download_and_verify(client: &reqwest::Client, video_url: &str, job_id: u64) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!("🔄 [DOWNLOAD] Starting video download for Job #{}", job_id);
    info!("🔄 [DOWNLOAD] Video URL: {}", video_url);
    
    info!("🔄 [DOWNLOAD] Sending GET request to video URL...");
    let res = client.get(video_url).send().await?;
    info!("✅ [DOWNLOAD] Response received (Status: {})", res.status());
    
    info!("🔄 [DOWNLOAD] Downloading video bytes (streaming)...");
    let bytes = res.bytes().await?;
    info!("✅ [DOWNLOAD] Download complete! Size: {} bytes ({:.2} MB)", bytes.len(), bytes.len() as f64 / 1_048_576.0);

    if bytes.len() == 0 {
        error!("❌ [DOWNLOAD] Downloaded video is 0 bytes! Aborting.");
        return Err("Downloaded video is 0 bytes!".into());
    }

    let filename = format!("videos/video_job_{}.mp4", job_id);
    info!("🔄 [DOWNLOAD] Writing video to disk: {}", filename);
    std::fs::write(&filename, &bytes)?;
    
    info!("✅ [DOWNLOAD] Video saved successfully!");
    info!("💾 [DOWNLOAD] File: {} | Size: {} bytes", filename, bytes.len());
    Ok(filename)
}

// ==========================================
// 6. BACKGROUND WORKER (The Pipeline)
// ==========================================
async fn run_pipeline(
    bot: Bot,
    chat_id: ChatId,
    idea: String,
    config: Arc<AppConfig>,
    db: Arc<Mutex<Connection>>,
) {
    info!("═══════════════════════════════════════════════════════════");
    info!("🚀 [PIPELINE] NEW JOB STARTED");
    info!("═══════════════════════════════════════════════════════════");
    info!("📝 [PIPELINE] Idea: '{}'", idea);
    info!("👤 [PIPELINE] Chat ID: {}", chat_id);
    
    let client = reqwest::Client::new();
    let created_at = chrono::Local::now().to_rfc3339();

    // Step 6.1: Insert into DB
    info!("🔄 [DATABASE] Creating new job entry...");
    let db_conn = db.lock().await;
    db_conn.execute(
        "INSERT INTO jobs (idea, status, created_at) VALUES (?1, ?2, ?3)",
        params![idea, "PROMPTING", created_at],
    ).unwrap();
    let job_id = db_conn.last_insert_rowid() as u64;
    drop(db_conn);
    info!("✅ [DATABASE] Job #{} created with status: PROMPTING", job_id);

    // Step 6.2: Generate Prompt
    info!("───────────────────────────────────────────────────────────");
    info!("📝 [PIPELINE] STEP 1: PROMPT GENERATION");
    info!("───────────────────────────────────────────────────────────");
    match generate_prompt(&config.prompt_api_key, &idea).await {
        Ok(prompt) => {
            info!("� [DATABASE] Updating job #{} status to GENERATING", job_id);
            let db_conn = db.lock().await;
            db_conn.execute("UPDATE jobs SET prompt = ?1, status = ?2 WHERE id = ?3", params![prompt, "GENERATING", job_id]).unwrap();
            drop(db_conn);
            info!("✅ [DATABASE] Job #{} updated with generated prompt", job_id);

            // Step 6.3: Generate Video (Fallback)
            info!("───────────────────────────────────────────────────────────");
            info!("🎬 [PIPELINE] STEP 2: VIDEO GENERATION");
            info!("───────────────────────────────────────────────────────────");
            match generate_video_with_fallback(&client, &config.veo_api_keys, &prompt).await {
                Ok(video_url) => {
                    info!("🔄 [DATABASE] Updating job #{} status to DOWNLOADING", job_id);
                    let db_conn = db.lock().await;
                    db_conn.execute("UPDATE jobs SET status = ?1 WHERE id = ?2", params!["DOWNLOADING", job_id]).unwrap();
                    drop(db_conn);
                    info!("✅ [DATABASE] Job #{} status updated", job_id);

                    // Step 6.4: Download & Verify
                    info!("───────────────────────────────────────────────────────────");
                    info!("💾 [PIPELINE] STEP 3: VIDEO DOWNLOAD");
                    info!("───────────────────────────────────────────────────────────");
                    match download_and_verify(&client, &video_url, job_id).await {
                        Ok(file_path) => {
                            info!("🔄 [DATABASE] Updating job #{} status to UPLOADED", job_id);
                            let db_conn = db.lock().await;
                            db_conn.execute("UPDATE jobs SET video_path = ?1, status = ?2 WHERE id = ?3", params![file_path, "UPLOADED", job_id]).unwrap();
                            drop(db_conn);
                            info!("✅ [DATABASE] Job #{} marked as UPLOADED", job_id);

                            // TODO: Phase 7 - Add YouTube Upload logic here. 
                            // Once uploaded, delete local file: std::fs::remove_file(&file_path)?;

                            info!("═══════════════════════════════════════════════════════════");
                            info!("🎉 [PIPELINE] JOB #{} COMPLETED SUCCESSFULLY!", job_id);
                            info!("═══════════════════════════════════════════════════════════");
                            let _ = bot.send_message(chat_id, format!("🎉 Video is ready! Saved at: {}\n(YouTube upload coming in Phase 7)", file_path)).await;
                        }
                        Err(e) => {
                            let error_msg = e.to_string();
                            error!("❌ [DOWNLOAD] Failed: {}", error_msg);
                            info!("🔄 [DATABASE] Marking job #{} as FAILED", job_id);
                            let db_conn = db.lock().await;
                            db_conn.execute("UPDATE jobs SET status = ?1 WHERE id = ?2", params!["FAILED", job_id]).unwrap();
                            drop(db_conn);
                            error!("═══════════════════════════════════════════════════════════");
                            error!("❌ [PIPELINE] JOB #{} FAILED AT DOWNLOAD STEP", job_id);
                            error!("═══════════════════════════════════════════════════════════");
                            let _ = bot.send_message(chat_id, format!("❌ Download failed: {}", error_msg)).await;
                        }
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    error!("❌ [VEO] Video generation failed: {}", error_msg);
                    info!("🔄 [DATABASE] Marking job #{} as FAILED", job_id);
                    let db_conn = db.lock().await;
                    db_conn.execute("UPDATE jobs SET status = ?1 WHERE id = ?2", params!["FAILED", job_id]).unwrap();
                    drop(db_conn);
                    error!("═══════════════════════════════════════════════════════════");
                    error!("❌ [PIPELINE] JOB #{} FAILED AT VIDEO GENERATION STEP", job_id);
                    error!("═══════════════════════════════════════════════════════════");
                    let _ = bot.send_message(chat_id, format!("❌ All API keys failed: {}", error_msg)).await;
                }
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            error!("❌ [PROMPT] Prompt generation failed: {}", error_msg);
            info!("🔄 [DATABASE] Marking job #{} as FAILED", job_id);
            let db_conn = db.lock().await;
            db_conn.execute("UPDATE jobs SET status = ?1 WHERE id = ?2", params!["FAILED", job_id]).unwrap();
            drop(db_conn);
            error!("═══════════════════════════════════════════════════════════");
            error!("❌ [PIPELINE] JOB #{} FAILED AT PROMPT GENERATION STEP", job_id);
            error!("═══════════════════════════════════════════════════════════");
            let _ = bot.send_message(chat_id, format!("❌ AI Prompt failed: {}", error_msg)).await;
        }
    }
}

// ==========================================
// 7. MAIN FUNCTION (Telegram Bot + Axum)
// ==========================================
fn main() {
    // Initialize logging system FIRST before anything else
    std::fs::create_dir_all("logs").ok();
    logging::rotate_log_file();
    
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║         VEO PIPELINE - NEW SESSION STARTED               ║");
    println!("║         {}                              ║", timestamp);
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    logging::setup_tracing();
    
    // Run the async runtime
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async_main());
}

async fn async_main() {
    // NO tracing_subscriber::fmt::init() here — already initialized in main()

    info!("═══════════════════════════════════════════════════════════");
    info!("🚀 [STARTUP] Initializing Veo Pipeline System");
    info!("═══════════════════════════════════════════════════════════");

    // Load Config
    let config = match AppConfig::load() {
        Ok(c) => Arc::new(c),
        Err(e) => {
            error!("❌ [STARTUP] Config error: {}", e);
            error!("═══════════════════════════════════════════════════════════");
            error!("❌ [STARTUP] SYSTEM FAILED TO START");
            error!("═══════════════════════════════════════════════════════════");
            return;
        }
    };

    // Init DB
    let db = match init_db() {
        Ok(c) => Arc::new(Mutex::new(c)),
        Err(e) => {
            error!("❌ [STARTUP] DB init error: {}", e);
            error!("═══════════════════════════════════════════════════════════");
            error!("❌ [STARTUP] SYSTEM FAILED TO START");
            error!("═══════════════════════════════════════════════════════════");
            return;
        }
    };

    // Init Telegram Bot
    info!("🔄 [STARTUP] Initializing Telegram bot...");
    let bot_token = std::env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN env var not set");
    let bot = Bot::new(bot_token);
    info!("✅ [STARTUP] Telegram bot initialized");

    // Axum Health Check Server (for deployment/monitoring)
    info!("🔄 [STARTUP] Starting Axum health check server...");
    let app = Router::new().route("/health", get(|| async { "{\"status\":\"ok\"}" }));
    
    // Spawn Axum server in background
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        info!("✅ [STARTUP] Axum server listening on http://localhost:3000");
        axum::serve(listener, app).await.unwrap();
    });

    info!("═══════════════════════════════════════════════════════════");
    info!("✅ [STARTUP] ALL SYSTEMS OPERATIONAL");
    info!("═══════════════════════════════════════════════════════════");
    info!("🚀 Server: http://localhost:3000/health");
    info!("🤖 Telegram: Listening for /idea commands");
    info!("📝 Logs: logs/pipeline.log");
    info!("═══════════════════════════════════════════════════════════\n");

    // Telegram Bot Handler using Dispatcher
    use teloxide::prelude::*;
    
    let handler = Update::filter_message().branch(
        dptree::filter(|msg: Message| msg.text().map_or(false, |t| t.starts_with("/idea")))
            .endpoint(|bot: Bot, msg: Message, config: Arc<AppConfig>, db: Arc<Mutex<Connection>>| async move {
                if let Some(text) = msg.text() {
                    let idea = text.strip_prefix("/idea").unwrap_or("").trim();
                    
                    if idea.is_empty() {
                        warn!("⚠️ [TELEGRAM] Empty idea received from chat {}", msg.chat.id);
                        bot.send_message(msg.chat.id, "Please provide an idea. Usage: `/idea A cat in space`").await?;
                        return Ok::<_, teloxide::RequestError>(());
                    }

                    info!("📨 [TELEGRAM] New /idea command received from chat {}", msg.chat.id);
                    info!("📝 [TELEGRAM] Idea: '{}'", idea);
                    
                    bot.send_message(msg.chat.id, format!("✅ Idea accepted: '{}'\n⏳ Processing in background...", idea)).await?;
                    info!("✅ [TELEGRAM] Confirmation sent to user");

                    let bot_clone = bot.clone();
                    info!("🔄 [TELEGRAM] Spawning background pipeline task...");
                    tokio::spawn(run_pipeline(bot_clone, msg.chat.id, idea.to_string(), config, db));
                    info!("✅ [TELEGRAM] Background task spawned\n");
                }
                
                Ok(())
            })
    );
    
    let mut dispatcher = teloxide::dispatching::Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![config, db])
        .build();
    
    dispatcher.dispatch().await;
}
