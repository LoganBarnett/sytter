// Device connection trigger integration test.
//
// STATUS: The device trigger implementation is COMPLETE and FUNCTIONAL.
//
// TESTING APPROACH:
// This test requires manual interaction - you must physically plug in a USB device
// when prompted. The test monitors IOUSBHostDevice (modern macOS USB device class)
// and uses FirstPublish notification type to catch device registration early.
//
// The trigger configuration is in tests/fixtures/test_device.toml:
// - device_class: IOUSBHostDevice (modern USB devices)
// - notification_type: FirstPublish (early device registration)
// - events: [Add] (monitors device addition)
//
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

/// Helper to kill the sytter process on drop.
struct SytterProcess {
  child: Child,
}

impl SytterProcess {
  fn new(child: Child) -> Self {
    SytterProcess { child }
  }
}

impl Drop for SytterProcess {
  fn drop(&mut self) {
    let _ = self.child.kill();
    let _ = self.child.wait();
  }
}

#[test]
#[ignore] // Manual test - requires physically plugging in a USB device
fn test_device_trigger_manual() {
  // Setup: Create a temporary file for test output.
  let temp_dir = std::env::temp_dir();
  let output_file =
    temp_dir.join(format!("sytter_device_test_{}.txt", std::process::id()));

  // Clean up any previous test file.
  let _ = fs::remove_file(&output_file);

  // Get the path to the test sytter config.
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let config_path = manifest_dir.join("tests/fixtures/test_device.toml");

  println!("Test output file: {}", output_file.display());
  println!("Config path: {}", config_path.display());

  // Use a random port to avoid conflicts with other tests or running instances.
  let test_port = 18080 + (std::process::id() % 1000);

  // Start sytter with the test configuration.
  // The config file specifies:
  // - device_class: IOUSBHostDevice
  // - notification_type: FirstPublish
  let child = Command::new(env!("CARGO_BIN_EXE_sytter"))
    .arg("--sytters-path")
    .arg(&config_path)
    .arg("--log-level")
    .arg("debug")
    .env("SYTTER_TEST_OUTPUT", &output_file)
    .env("sytter_http_port", test_port.to_string())
    .spawn()
    .expect("Failed to start sytter");

  // Wrap in our cleanup helper.
  let _process = SytterProcess::new(child);

  println!("Sytter started, waiting for health check...");

  // Wait for the health check to be ready (max 5 seconds).
  let health_url = format!("http://localhost:{}/health", test_port);
  let start = Instant::now();
  let max_wait = Duration::from_secs(5);
  let mut healthy = false;

  while start.elapsed() < max_wait {
    match reqwest::blocking::get(&health_url) {
      Ok(response) if response.status().is_success() => {
        println!("Health check passed after {:?}", start.elapsed());
        healthy = true;
        break;
      }
      _ => {
        thread::sleep(Duration::from_millis(100));
      }
    }
  }

  assert!(
    healthy,
    "Health check did not return success within {} seconds",
    max_wait.as_secs()
  );

  // Give the device listener a moment to fully initialize.
  thread::sleep(Duration::from_secs(2));

  println!("\n===========================================");
  println!("MANUAL TEST: Please plug in a USB device now!");
  println!("===========================================\n");
  println!("The test will wait 15 seconds for you to plug in a device...");

  // Wait for the user to plug in a device
  thread::sleep(Duration::from_secs(15));

  // Check that the file was created and has at least one entry.
  assert!(
    output_file.exists(),
    "Output file should exist after device trigger. Did you plug in a USB device?"
  );

  let contents =
    fs::read_to_string(&output_file).expect("Failed to read output file");
  let lines: Vec<&str> = contents.lines().collect();

  println!("Output file has {} lines", lines.len());
  println!("Output contents:\n{}", contents);

  assert!(
    lines.len() >= 1,
    "Should have at least 1 execution after device connection"
  );

  // Verify that the output is a valid timestamp.
  if lines.len() >= 1 {
    let timestamp: i64 =
      lines[0].trim().parse().expect("Failed to parse timestamp");
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;

    assert!(
      timestamp > 0 && timestamp <= now,
      "Timestamp should be a valid unix timestamp"
    );
  }

  // Cleanup.
  let _ = fs::remove_file(&output_file);
  println!("Test completed successfully!");
}
