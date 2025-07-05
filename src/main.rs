use std::{env, thread, time::Duration}; // Removed SystemTime, UNIX_EPOCH
use log::{info, warn, error, debug};

mod detect;
mod discord;

const DEFAULT_DISCORD_APP_ID: &str = "YOUR_DISCORD_APP_ID_HERE"; // IMPORTANT: Replace with actual App ID
const POLLING_INTERVAL_SECONDS: u64 = 15;

fn main_loop(mut discord_client: discord::DiscordClient) {
    let mut last_known_project_name: Option<String> = None;
    let mut resolve_was_running = false;
    // Removed unused session_start_time variable


    loop {
        let resolve_running = detect::is_resolve_running();

        if resolve_running {
            if !resolve_was_running {
                info!("DaVinci Resolve has started.");
                // session_start_time was previously set here
                // Attempt to connect to Discord if not already connected
                if !discord_client.is_connected() {
                    match discord_client.connect() {
                        Ok(_) => info!("Successfully connected to Discord for new Resolve session."),
                        Err(e) => {
                            error!("Failed to connect to Discord: {}. Will retry.", e);
                            // No need to set resolve_was_running to false, loop will retry connection
                        }
                    }
                }
            }
            resolve_was_running = true;

            // Only try to set activity if connected
            if discord_client.is_connected() {
                let current_project_name = match detect::get_resolve_window_title() {
                    Some(title) => detect::parse_project_name_from_title(&title),
                    None => {
                        // This case might happen if Resolve is running but the main window isn't found (e.g. splash screen)
                        // Or if on a non-windows system and get_resolve_window_title provides a generic title without project info
                        debug!("Could not get Resolve window title, or no project name parsable from it.");
                        None
                    }
                };

                // Update activity if project name changed or if it's the first update for this session
                if last_known_project_name != current_project_name || current_project_name.is_none() && last_known_project_name.is_some() {
                     info!("Project changed from {:?} to {:?}. Updating Discord.", last_known_project_name, current_project_name);
                }

                // Pass the session_start_time to set_activity
                // The DiscordClient's internal start_time is for the connection, this one is for the current Resolve session
                // For simplicity, we'll let DiscordClient manage its own start_time for the presence timestamp for now.
                // If project specific time is needed, that would be a more complex change.
                // The current discord.rs sets its own start_time on connect.
                // We could potentially pass 'session_start_time' to a modified 'set_activity' if we want project-specific time.

                match discord_client.set_activity(current_project_name.as_deref()) {
                    Ok(_) => {
                        if last_known_project_name != current_project_name { // Log only on change
                             debug!("Discord activity updated for project: {:?}", current_project_name);
                        }
                        last_known_project_name = current_project_name;
                    }
                    Err(e) => {
                        warn!("Failed to set Discord activity: {}. Will attempt to reconnect if needed.", e);
                        // The discord_client.set_activity might have set itself to not connected.
                        // The next loop iteration will try to connect if resolve is still running.
                        last_known_project_name = None; // Reset to force update next time
                    }
                }
            } else if resolve_was_running { // Resolve is running, but discord is not connected
                 warn!("Resolve is running, but Discord client is not connected. Attempting to reconnect...");
                 if discord_client.connect().is_err() {
                     error!("Failed to reconnect to Discord. Will retry later.");
                 }
            }

        } else { // Resolve is not running
            if resolve_was_running {
                info!("DaVinci Resolve has exited.");
                if discord_client.is_connected() {
                    info!("Clearing Discord activity and closing connection.");
                    if let Err(e) = discord_client.clear_activity() {
                        warn!("Failed to clear Discord activity: {}", e);
                    }
                    if let Err(e) = discord_client.close() {
                        warn!("Failed to close Discord connection cleanly: {}", e);
                    }
                }
                last_known_project_name = None; // Reset project name
            }
            resolve_was_running = false;
        }

        thread::sleep(Duration::from_secs(POLLING_INTERVAL_SECONDS));
    }
}


fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting DaVinci Resolve Discord Rich Presence application...");
    info!("Polling interval: {} seconds", POLLING_INTERVAL_SECONDS);

    let app_id = env::var("DISCORD_APP_ID").unwrap_or_else(|_| {
        warn!("DISCORD_APP_ID environment variable not set. Using default: {}", DEFAULT_DISCORD_APP_ID);
        warn!("Please replace YOUR_DISCORD_APP_ID_HERE with your actual Discord Application ID.");
        DEFAULT_DISCORD_APP_ID.to_string()
    });

    if app_id == DEFAULT_DISCORD_APP_ID {
        error!("Critical: You must set a Discord Application ID, either via DISCORD_APP_ID environment variable or by changing the default in the code.");
        error!("Get an App ID from the Discord Developer Portal (https://discord.com/developers/applications)");
        // std::process::exit(1); // Consider exiting if no valid ID is provided. For now, let it run but fail on connect.
    }


    let discord_client = discord::DiscordClient::new(app_id.clone());

    // Graceful shutdown handler
    // This is a bit tricky because the main_loop is blocking.
    // For a simple CLI app, Ctrl+C is usually handled by the OS, which would terminate the process.
    // The Discord client has its own drop implementation that tries to close, but it might not run fully.
    // A more robust solution would involve `ctrlc` crate and channels.
    // For now, we rely on the OS to terminate and the fact that Discord RPC handles client disappearance.

    info!("Application started. Press Ctrl+C to exit.");

    // Initial connection attempt before loop, or let the loop handle it?
    // Let the loop handle it to simplify logic around Resolve starting/stopping.
    main_loop(discord_client);

    // Code here might not be reached if main_loop runs forever.
    // If we implement a shutdown signal, cleanup would go here.
    info!("Application shutting down...");
}
