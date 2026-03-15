use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

/// Helper to kill the sytter process on drop
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
fn test_cron_trigger_executes_periodically() {
  // Setup: Create a temporary file for test output
  let temp_dir = std::env::temp_dir();
  let output_file =
    temp_dir.join(format!("sytter_test_{}.txt", std::process::id()));

  // Clean up any previous test file
  let _ = fs::remove_file(&output_file);

  // Get the path to the test sytter config
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let config_path = manifest_dir.join("tests/fixtures/test_cron.toml");

  println!("Test output file: {}", output_file.display());
  println!("Config path: {}", config_path.display());

  // Use a random port to avoid conflicts with other tests or running instances
  let test_port = 18080 + (std::process::id() % 1000);

  // Start sytter with the test configuration
  let child = Command::new(env!("CARGO_BIN_EXE_sytter"))
    .arg("--sytters-path")
    .arg(&config_path)
    .arg("--log-level")
    .arg("debug")
    .env("SYTTER_TEST_OUTPUT", &output_file)
    .env("sytter_http_port", test_port.to_string())
    .spawn()
    .expect("Failed to start sytter");

  // Wrap in our cleanup helper
  let _process = SytterProcess::new(child);

  println!("Sytter started, waiting for health check...");

  // Wait for the health check to be ready (max 5 seconds)
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

  println!("Waiting for first cron execution...");

  // Wait for first execution (cron is set to */5 seconds, so wait up to 7 seconds)
  thread::sleep(Duration::from_secs(7));

  // Check that the file was created and has at least one entry
  assert!(
    output_file.exists(),
    "Output file should exist after first execution"
  );

  let contents =
    fs::read_to_string(&output_file).expect("Failed to read output file");
  let lines: Vec<&str> = contents.lines().collect();

  println!("After 7 seconds, output file has {} lines", lines.len());
  assert!(
    lines.len() >= 1,
    "Should have at least 1 execution after 7 seconds"
  );

  // Wait for second execution (another 5 seconds)
  println!("Waiting for second execution...");
  thread::sleep(Duration::from_secs(5));

  // Check that we have at least 2 executions now
  let contents =
    fs::read_to_string(&output_file).expect("Failed to read output file");
  let lines: Vec<&str> = contents.lines().collect();

  println!(
    "After 12 seconds total, output file has {} lines",
    lines.len()
  );
  println!("Output contents:\n{}", contents);

  assert!(
    lines.len() >= 2,
    "Should have at least 2 executions after 12 seconds, got {}",
    lines.len()
  );

  // Verify that the timestamps are increasing (each line is a unix timestamp)
  if lines.len() >= 2 {
    let first: i64 = lines[0]
      .trim()
      .parse()
      .expect("Failed to parse first timestamp");
    let second: i64 = lines[1]
      .trim()
      .parse()
      .expect("Failed to parse second timestamp");
    assert!(
      second > first,
      "Second timestamp should be after first timestamp"
    );
    assert!(
      second - first >= 4 && second - first <= 7,
      "Timestamps should be roughly 5 seconds apart (allowing for timing variance), got {} seconds",
      second - first
    );
  }

  // Cleanup
  let _ = fs::remove_file(&output_file);
  println!("Test completed successfully!");
}
