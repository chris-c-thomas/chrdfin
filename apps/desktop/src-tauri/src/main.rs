// Hide the Windows console window on release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    chrdfin_desktop_lib::run()
}
