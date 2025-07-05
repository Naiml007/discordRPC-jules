use sysinfo::System; // Removed ProcessExt, SystemExt, PidExt and Pid

#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::{BOOL, LPARAM, MAX_PATH, TRUE, HWND}, // Added HWND
    // Removed OpenProcess, PROCESS_QUERY_INFORMATION from System::Threading
    UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    },
};

#[cfg(windows)]
struct WindowInfo {
    pid: u32,
    hwnd: HWND, // Changed from usize to HWND
    title: String,
}

#[cfg(windows)]
extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let windows_info = &mut *(lparam.0 as *mut Vec<WindowInfo>); // LPARAM is a tuple struct (isize,), so use .0
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid); // HWND can be passed directly

        if pid != 0 {
            if IsWindowVisible(hwnd) == TRUE { // HWND can be passed directly
                let mut text: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
                let len = GetWindowTextW(hwnd, text.as_mut_ptr(), MAX_PATH as i32); // HWND can be passed directly
                if len > 0 {
                    let title = String::from_utf16_lossy(&text[..len as usize]);
                    if !title.is_empty() { // Ensure title is not empty
                        windows_info.push(WindowInfo { pid, hwnd, title });
                    }
                }
            }
        }
        TRUE
    }
}

#[cfg(windows)]
fn get_all_windows_with_titles() -> Vec<WindowInfo> {
    let mut windows_info: Vec<WindowInfo> = Vec::new();
    unsafe {
        EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut windows_info as *mut _ as isize), // Pass LPARAM as a tuple struct
        );
    }
    windows_info
}

/// Checks if a process named "Resolve.exe" is running.
pub fn is_resolve_running() -> bool {
    let s = System::new_all();
    for process in s.processes_by_name("Resolve.exe") {
        log::debug!("Found Resolve process: {:?}", process.name());
        return true;
    }
    false
}

/// Gets the window title of the main DaVinci Resolve window.
/// Returns None if Resolve is not running or the window title cannot be found.
#[cfg(windows)]
pub fn get_resolve_window_title() -> Option<String> {
    let s = System::new_all();
    let resolve_processes: Vec<_> = s.processes_by_name("Resolve.exe").collect();

    if resolve_processes.is_empty() {
        log::debug!("Resolve process not found.");
        return None;
    }

    // We might have multiple Resolve.exe processes (e.g. helper processes).
    // We need to find the one with the main window title.
    let all_windows = get_all_windows_with_titles();
    log::debug!("Found {} visible windows with titles.", all_windows.len());

    for p_info in resolve_processes {
        let pid = p_info.pid().as_u32();
        log::debug!("Checking PID: {}", pid);
        for window in &all_windows {
            if window.pid == pid {
                log::debug!("Found window for PID {}: '{}'", pid, window.title);
                // Further checks can be added here if "DaVinci Resolve" is too generic
                // e.g. ensuring it's the main application window, not a dialog.
                // For now, we assume any window from Resolve.exe containing "DaVinci Resolve" is a candidate.
                if window.title.contains("DaVinci Resolve") {
                    log::info!("Found Resolve window: {}", window.title);
                    return Some(window.title.clone());
                }
            }
        }
    }
    log::warn!("Resolve process is running, but no suitable window title found.");
    None
}

#[cfg(not(windows))]
pub fn get_resolve_window_title() -> Option<String> {
    log::warn!("Window title detection is only implemented for Windows currently.");
    // Fallback for non-Windows: if Resolve is running, return a generic title.
    if is_resolve_running() {
        Some("DaVinci Resolve".to_string()) // Generic title
    } else {
        None
    }
}

/// Parses the project name from the Resolve window title.
/// Example titles: "DaVinci Resolve - ProjectName", "DaVinci Resolve Studio - ProjectName", "DaVinci Resolve"
pub fn parse_project_name_from_title(title: &str) -> Option<String> {
    log::debug!("Parsing project name from title: {}", title);
    if title.contains(" - ") {
        let parts: Vec<&str> = title.splitn(2, " - ").collect();
        if parts.len() == 2 && !parts[1].is_empty() && parts[1] != "Untitled Project" { // Also check for "Untitled Project" if desired
            // Further check if parts[0] is "DaVinci Resolve" or "DaVinci Resolve Studio"
            if parts[0].starts_with("DaVinci Resolve") {
                 log::info!("Parsed project name: {}", parts[1]);
                return Some(parts[1].to_string());
            }
        }
    }
    log::debug!("No project name found in title, or it's an untitled project.");
    None // No project name found or it's the main screen without a project open
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_project_name_simple() {
        let title = "DaVinci Resolve - My Awesome Project";
        assert_eq!(parse_project_name_from_title(title), Some("My Awesome Project".to_string()));
    }

    #[test]
    fn test_parse_project_name_studio() {
        let title = "DaVinci Resolve Studio - Another Project_123";
        assert_eq!(parse_project_name_from_title(title), Some("Another Project_123".to_string()));
    }

    #[test]
    fn test_parse_project_name_no_project() {
        let title = "DaVinci Resolve";
        assert_eq!(parse_project_name_from_title(title), None);
    }

    #[test]
    fn test_parse_project_name_untitled() {
        let title = "DaVinci Resolve - Untitled Project";
        assert_eq!(parse_project_name_from_title(title), None);
    }

    #[test]
    fn test_parse_project_name_edge_case_extra_dashes() {
        let title = "DaVinci Resolve - Project - With - Dashes";
         assert_eq!(parse_project_name_from_title(title), Some("Project - With - Dashes".to_string()));
    }

    #[test]
    fn test_parse_project_name_not_resolve_title() {
        let title = "Some Other Application - My Project";
        assert_eq!(parse_project_name_from_title(title), None);
    }
}
