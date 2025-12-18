// SPDX-License-Identifier: MPL-2.0
//! Build script for platform-specific resources.
//!
//! On Windows, this embeds the application icon into the executable
//! so it appears in the taskbar and file explorer.

fn main() {
    // Only run on Windows
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/branding/iced_lens.ico");
        res.compile().expect("Failed to compile Windows resources");
    }
}
