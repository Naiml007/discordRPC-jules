# DaVinci Resolve Discord Rich Presence

This application provides Discord Rich Presence integration for DaVinci Resolve, showing your current project status on Discord.

## Features

- Detects when DaVinci Resolve is running on Windows.
- Extracts the current project name from the Resolve window title.
- Updates your Discord status with "Editing: [Project Name]" or "Idle".
- Includes a timestamp for how long you've been in the current Resolve session.
- Automatically connects/disconnects when Resolve starts/exits.
- Refreshes presence periodically.

## Prerequisites

1.  **Rust:** Install Rust and Cargo from [rustup.rs](https://rustup.rs/).
2.  **Discord Desktop Client:** The application communicates with your running Discord client. Ensure it's open.
3.  **DaVinci Resolve:** This application is designed for DaVinci Resolve.

## Setup

### 1. Create a Discord Application

You need to create a Discord Application to get an Application ID and manage Rich Presence assets.

1.  Go to the [Discord Developer Portal](https://discord.com/developers/applications).
2.  Click "**New Application**" in the top right corner.
3.  Give your application a name (e.g., "DaVinci Resolve Presence") and click "**Create**".
4.  Navigate to the "**Rich Presence**" tab in the left sidebar.
    *   Under "**Rich Presence Assets**", you can upload images that will be displayed in your status.
        *   Add a large image asset, for example, named `resolve_logo`. This key is used in `src/discord.rs` as `RESOLVE_LOGO_KEY`.
        *   Optionally, add a small image asset, for example, named `editing_icon`. This key is used as `EDITING_ICON_KEY`.
        *   **Important:** The keys you use for your assets here *must* match the constants `RESOLVE_LOGO_KEY` and `EDITING_ICON_KEY` in `src/discord.rs` if you want them to display correctly. You can change the constants in the code to match your asset names.
    *   You don't need to fill out much else here unless you want to customize further.
5.  Navigate to the "**OAuth2**" -> "**General**" tab.
    *   Copy your "**Client ID**" (this is your Application ID). You will need this in the next step.

### 2. Configure the Application ID

There are two ways to provide the Application ID to this program:

*   **Environment Variable (Recommended):**
    Set an environment variable named `DISCORD_APP_ID` to your Client ID.
    *   On Windows (PowerShell):
        ```powershell
        $env:DISCORD_APP_ID="YOUR_CLIENT_ID_HERE"
        ```
    *   On Windows (Command Prompt):
        ```cmd
        set DISCORD_APP_ID=YOUR_CLIENT_ID_HERE
        ```
    *   On Linux/macOS:
        ```bash
        export DISCORD_APP_ID="YOUR_CLIENT_ID_HERE"
        ```
    You might want to add this to your shell's profile file (like `.bashrc`, `.zshrc`, or PowerShell profile) to set it automatically.

*   **Edit `src/main.rs` (Not Recommended for sharing):**
    You can directly replace the placeholder in `src/main.rs`:
    ```rust
    const DEFAULT_DISCORD_APP_ID: &str = "YOUR_DISCORD_APP_ID_HERE";
    // Change to:
    // const DEFAULT_DISCORD_APP_ID: &str = "YOUR_ACTUAL_CLIENT_ID";
    ```
    And also update the asset keys in `src/discord.rs` if you named your assets differently in the Discord Developer Portal:
    ```rust
    const RESOLVE_LOGO_KEY: &str = "your_resolve_logo_asset_key";
    const EDITING_ICON_KEY: &str = "your_editing_icon_asset_key";
    ```

## Building the Application

1.  Clone this repository or download the source code.
2.  Open a terminal in the project's root directory.
3.  Run the build command:
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/release/resolve_discord_presence.exe` (on Windows).

## Running the Application

1.  Ensure your Discord desktop client is running.
2.  Ensure DaVinci Resolve is installed.
3.  Run the executable from the `target/release/` directory:
    *   On Windows: `resolve_discord_presence.exe`
    *   (If compiled for Linux/macOS): `./resolve_discord_presence`

The application will run in your terminal, logging its activity.

## Testing Steps

1.  **Initial State (Resolve NOT running):**
    *   Run the `resolve_discord_presence` application.
    *   Check your Discord profile. No Rich Presence status related to Resolve should be visible.
    *   The terminal output should indicate it's polling and Resolve is not found.

2.  **Start DaVinci Resolve (No Project Open):**
    *   Start DaVinci Resolve.
    *   Wait for up to 15-30 seconds (due to polling interval).
    *   Check your Discord profile. You should see a status like "DaVinci Resolve" with "Idle" or similar, along with the application name you set in the Discord Developer Portal and the logo if configured.
    *   The terminal output should show that Resolve was detected and Discord presence was updated.

3.  **Open a Project in Resolve:**
    *   In DaVinci Resolve, open or create a project (e.g., "My Test Project").
    *   Wait for up to 15-30 seconds.
    *   Check your Discord profile. The status should update to something like "Editing: My Test Project".
    *   The terminal output should show the detected project name.

4.  **Switch Projects in Resolve:**
    *   In DaVinci Resolve, switch to a different project (e.g., "Another Cool Edit").
    *   Wait for up to 15-30 seconds.
    *   Check your Discord profile. The status should update to "Editing: Another Cool Edit".

5.  **Close Project (Return to Project Manager in Resolve):**
    *   In DaVinci Resolve, close the current project to return to the project manager screen.
    *   Wait for up to 15-30 seconds.
    *   Check your Discord profile. The status should revert to "Idle" or similar.

6.  **Close DaVinci Resolve:**
    *   Completely close DaVinci Resolve.
    *   Wait for up to 15-30 seconds.
    *   Check your Discord profile. The Rich Presence status for Resolve should disappear.
    *   The terminal output should indicate Resolve has exited and the Discord connection was closed.

7.  **Check Logs for Errors:**
    *   Throughout testing, monitor the terminal output for any `WARN` or `ERROR` messages.
    *   If you want more detailed logs, you can run the application with the `RUST_LOG` environment variable:
        ```bash
        RUST_LOG=debug ./target/release/resolve_discord_presence
        # (or on Windows PowerShell)
        # $env:RUST_LOG="debug"; ./target/release/resolve_discord_presence.exe; $env:RUST_LOG=""
        ```

## Troubleshooting

*   **No Presence Shown:**
    *   Ensure your Discord App ID is correctly set (environment variable or in code).
    *   Ensure your Discord client is running and you are logged in.
    *   Check the "Activity Status" settings in Discord (User Settings -> Activity Settings -> Activity Status) to ensure "Display current activity as a status message." is enabled.
    *   Check the terminal logs for errors related to Discord connection.
    *   Make sure the asset keys in `src/discord.rs` match the names you gave them in the Discord Developer Portal if images aren't showing.
*   **Incorrect Project Name:**
    *   The project name is parsed from the window title. If Resolve's window title format changes or is unusual, parsing might fail. The current parsing logic in `src/detect.rs` looks for "DaVinci Resolve - Project Name".
*   **Application Crashes:**
    *   Check terminal logs for panic messages or errors. Please report them as issues if they seem to be bugs in this application.

This README provides a good starting point for users to set up, build, and test the application.Tool output for `create_file_with_block`:
