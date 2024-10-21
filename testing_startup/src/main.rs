use std::process::Command;
use std::time::Duration;
use std::thread::sleep;
use std::env;

fn main() {
    println!("Starting YugabyteDB Docker container...");
    match Command::new("docker")
        .args(&[
            "run", "-d", "--name", "yugabyte",
            "-p7001:7000", "-p9000:9000", "-p15433:15433",
            "-p5433:5433", "-p9042:9042",
            "yugabytedb/yugabyte:2024.1.2.0-b77",
            "bin/yugabyted", "start", "--background=false"
        ])
        .status()
    {
        Ok(status) => {
            if !status.success() {
                shutdown("Failed to run YugabyteDB Docker container.");
                return;
            }
        },
        Err(_) => { shutdown("Failed to run YugabyteDB Docker container."); return; }
    }
    println!("YugabyteDB started, waiting to initialize (10 seconds)...");
    sleep(Duration::from_secs(10));

    env::set_current_dir("server").expect("Failed to change directory to server");

    println!("Setting and populating Diesel database...");
    match Command::new("diesel")
        .arg("database")
        .arg("reset")
        .status()
    {
        Ok(status) => {
            if !status.success() {
                shutdown("Failed to reset Diesel database.");
                return;
            }
        },
        Err(_) => { shutdown("Failed to reset Diesel database."); return; }
    }
    sleep(Duration::from_secs(5));


    println!("Running tests...");
    match Command::new("cargo")
        .arg("test")
        .status()
    {
        Ok(status) => {
            if !status.success() {
                shutdown("Failed to run the server.");
                return;
            }
        },
        Err(_) => { shutdown("Failed to run the server."); return; }
    }

    shutdown("Successful Testing");
}

fn shutdown(comment: &str){
    eprintln!("{}", comment);
    let stop_container = Command::new("docker")
        .args(&["stop", "yugabyte"])
        .status()
        .expect("Failed to stop YugabyteDB Docker container");

    if !stop_container.success() {
        eprintln!("Failed to stop YugabyteDB Docker container.");
    } else {
        let remove_container = Command::new("docker")
            .args(&["rm", "yugabyte"])
            .status()
            .expect("Failed to remove YugabyteDB Docker container");

        if !remove_container.success() {
            eprintln!("Failed to remove YugabyteDB Docker container.");
        } else {
            println!("YugabyteDB Docker container stopped and removed successfully.");
        }
    }
}
