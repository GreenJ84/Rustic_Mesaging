use std::process::Command;
use std::time::Duration;
use std::thread::sleep;
use std::env;

fn main() {
    // Step 1: Run the Docker container for YugabyteDB
    println!("Starting YugabyteDB Docker container...");
    let yugabyte = Command::new("docker")
        .args(&[
            "run", "-d", "--name", "yugabyte",
            "-p7001:7000", "-p9000:9000", "-p15433:15433",
            "-p5433:5433", "-p9042:9042",
            "yugabytedb/yugabyte:2024.1.2.0-b77",
            "bin/yugabyted", "start", "--background=false"
        ])
        .status()
        .expect("Failed to start YugabyteDB Docker container");

    if !yugabyte.success() {
        eprintln!("Failed to run YugabyteDB Docker container.");
        return;
    }

    // Wait for 20 seconds to let YugabyteDB initialize
    println!("Waiting for YugabyteDB to initialize (10 seconds)...");
    sleep(Duration::from_secs(10));

    // Step 2: Move to Server directory
    env::set_current_dir("server").expect("Failed to change directory to server");

    // Step 3: Reset the Diesel database
    println!("Resetting Diesel database...");
    let diesel_reset = Command::new("diesel")
        .arg("database")
        .arg("reset")
        .status()
        .expect("Failed to reset the Diesel database");

    if !diesel_reset.success() {
        eprintln!("Failed to reset Diesel database.");
        return;
    }

    // Wait for 10 seconds
    println!("Waiting for 5 seconds...");
    sleep(Duration::from_secs(5));

    // Step 4: Run the server
    println!("Starting the server...");
    let server = Command::new("cargo")
        .arg("run")
        .spawn()
        .expect("Failed to run the server");

    println!("Server running in background (PID: {:?})", server.id());

    // Step 5: Move to Client directory
    println!("Serving the client...");
    env::set_current_dir("../client").expect("Failed to change directory to client");

    // Step 6: Serve the client
    let trunk_serve = Command::new("trunk")
        .arg("serve")
        .status()
        .expect("Failed to serve the client");

    if !trunk_serve.success() {
        eprintln!("Failed to serve the client.");
        return;
    }

    println!("All commands executed successfully.");
}
