// Trên Windows, ẩn cửa sổ console ở bản dựng release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    git_plum_lib::run()
}
