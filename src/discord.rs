use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::time::{SystemTime, UNIX_EPOCH};
use log::{debug, error, info, warn};

const RESOLVE_LOGO_KEY: &str = "resolve_logo"; // Replace with your actual asset key in Discord Developer Portal
const EDITING_ICON_KEY: &str = "editing_icon"; // Replace with your actual asset key

pub struct DiscordClient {
    client: Option<DiscordIpcClient>,
    app_id: String,
    connected: bool,
    last_project_name: Option<String>,
    start_time: Option<i64>,
}

impl DiscordClient {
    pub fn new(app_id: String) -> Self {
        DiscordClient {
            client: None,
            app_id,
            connected: false,
            last_project_name: None,
            start_time: None,
        }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        if self.connected && self.client.is_some() {
            info!("Discord client already connected.");
            return Ok(());
        }

        info!("Connecting to Discord with App ID: {}", self.app_id);
        let mut client = match DiscordIpcClient::new(&self.app_id) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to create Discord IPC client: {:?}", e);
                return Err(format!("Failed to create Discord IPC client: {:?}", e));
            }
        };

        if let Err(e) = client.connect() {
            error!("Failed to connect to Discord IPC: {:?}", e);
            // Attempt to close if connection failed mid-way
            let _ = client.close();
            return Err(format!("Failed to connect to Discord IPC: {:?}", e));
        }

        info!("Successfully connected to Discord IPC.");
        self.client = Some(client);
        self.connected = true;
        self.start_time = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64);
        Ok(())
    }

    pub fn set_activity(&mut self, project_name: Option<&str>) -> Result<(), String> {
        if !self.connected || self.client.is_none() {
            warn!("Discord client not connected. Call connect() first.");
            return Err("Client not connected".to_string());
        }

        let current_project_name_str = project_name.map(|s| s.to_string());

        // Only update if project name has changed
        // The is_ready() check was removed as the method doesn't exist in this version of the crate.
        // Error handling on set_activity will manage disconnections.
        if self.last_project_name == current_project_name_str {
            debug!("Project name unchanged ('{:?}'), skipping redundant update.", current_project_name_str);
            // If client.set_activity fails later, it will handle reconnection logic.
            // We could add a check here to see if self.client is Some, but set_activity itself checks.
            return Ok(());
        }

        info!("Setting Discord activity. Project: {:?}", project_name);

        let mut assets = activity::Assets::new();
        assets = assets.large_image(RESOLVE_LOGO_KEY);
        // assets = assets.large_text("DaVinci Resolve"); // Optional: text for large image

        let details_text;
        if let Some(name) = project_name {
            details_text = format!("Editing: {}", name);
            assets = assets.small_image(EDITING_ICON_KEY);
            // assets = assets.small_text("Actively Editing"); // Optional: text for small image
        } else {
            details_text = "Idle".to_string();
        }

        let mut activity_payload = activity::Activity::new()
            .details(&details_text)
            .assets(assets);

        if let Some(start_time) = self.start_time {
            activity_payload = activity_payload.timestamps(activity::Timestamps::new().start(start_time));
        }

        // Example button
        // activity_payload = activity_payload.buttons(vec![
        //     activity::Button::new("Learn More", "https://www.blackmagicdesign.com/products/davinciresolve/")
        // ]);

        match self.client.as_mut().unwrap().set_activity(activity_payload) {
            Ok(_) => {
                info!("Discord activity updated successfully for project: {:?}", project_name);
                self.last_project_name = current_project_name_str;
            }
            Err(e) => {
                error!("Failed to set Discord activity: {:?}", e);
                // If setting activity fails, it might mean the connection dropped.
                // We could try to reconnect here or mark as disconnected.
                // For now, we'll log the error and let the main loop handle re-connection attempt if Resolve is still running.
                self.connected = false; // Mark as potentially disconnected
                let _ = self.client.as_mut().unwrap().close(); // Attempt to close cleanly
                self.client = None;
                return Err(format!("Failed to set Discord activity: {:?}", e));
            }
        }
        Ok(())
    }

    pub fn clear_activity(&mut self) -> Result<(), String> {
        if !self.connected || self.client.is_none() {
            info!("Discord client not connected or already cleared.");
            return Ok(());
        }
        info!("Clearing Discord activity.");
        match self.client.as_mut().unwrap().clear_activity() {
            Ok(_) => {
                info!("Discord activity cleared successfully.");
                self.last_project_name = None;
            }
            Err(e) => {
                error!("Failed to clear Discord activity: {:?}", e);
                 // Similar to set_activity error, connection might be an issue.
                self.connected = false;
                let _ = self.client.as_mut().unwrap().close();
                self.client = None;
                return Err(format!("Failed to clear Discord activity: {:?}", e));
            }
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), String> {
        if !self.connected || self.client.is_none() {
            info!("Discord client already closed or was not connected.");
            self.connected = false;
            self.client = None; // Ensure client is None
            return Ok(());
        }
        info!("Closing Discord client connection.");
        let res = self.client.as_mut().unwrap().close();
        self.client = None;
        self.connected = false;
        self.last_project_name = None;
        self.start_time = None;

        if let Err(e) = res {
            error!("Error while closing Discord client: {:?}", e);
            return Err(format!("Error while closing Discord client: {:?}", e));
        }
        info!("Discord client closed successfully.");
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        // is_ready() was removed. Connection status is based on self.connected flag
        // and whether the client object exists. Actual readiness is tested by operations.
        self.connected && self.client.is_some()
    }
}

// Basic tests could be added here, but they would likely require a running Discord client
// or mocking the DiscordIpcClient, which is more involved.
// For now, testing will primarily be through manual execution with a Discord client.
#[cfg(test)]
mod tests {
    // use super::*;
    // It's tricky to unit test this module without a live Discord instance or complex mocking.
    // We'll rely on integration testing via main.rs for now.
}
