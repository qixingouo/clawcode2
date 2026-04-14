//! Simple test binary to verify the desktop web UI server works.
use std::thread;
use std::time::Duration;

fn main() {
    println!("Starting Claw Code Desktop server test...");

    let result = desktop::start_web_ui();
    match result {
        Ok(url) => {
            println!("✅ Desktop web UI server started at: {}", url);
            println!("   Health check: {}/api/health", url);
            println!("   Status check: {}/api/status", url);
            println!("\n   Opening browser in 1 second...");
            desktop::open_browser(&url);
            println!("✅ Browser opened!");
            println!("\n   Server is running. Press Ctrl+C to stop.");
            println!("   Or test with: curl {}/api/health", url);

            // Keep alive for a bit to allow testing
            thread::sleep(Duration::from_secs(5));

            // Verify it's actually running
            if desktop::is_running() {
                println!("\n✅ Verified: server is running!");
            } else {
                println!("\n❌ Error: server stopped unexpectedly");
            }

            desktop::stop_web_ui();
            println!("\n✅ Server stopped.");
        }
        Err(e) => {
            println!("❌ Failed to start server: {}", e);
        }
    }
}
