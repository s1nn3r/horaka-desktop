// Ohne Konsolenfenster unter Windows im Release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    horaka_desktop_lib::run()
}
