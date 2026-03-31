use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;

#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: whenpathchanges <path> [command...]");
        std::process::exit(1);
    }
    let target_path = &args[0];
    let cmd_args = &args[1..];

    // Bridge the synchronous OS-native watcher callback to our tokio async world
    // mpsc=multiple producer, single consumer
    let (transmitter, mut receiver) = mpsc::unbounded_channel();
    let mut watcher = RecommendedWatcher::new(
        move |result| {
            // This closure is called by the OS-native notify thread.
            // Sending to an unbounded tokio channel is lock-free and instantaneous, 
            // ensuring we don't block the OS kernel event queue.
            let _ = transmitter.send(result);
        },
        Config::default(),
    )
    .expect("Failed to hook into OS filesystem events");

    watcher
        .watch(Path::new(target_path), RecursiveMode::Recursive)
        .unwrap_or_else(|e| {
            eprintln!("Failed to watch path '{}': {}", target_path, e);
            std::process::exit(1);
        })
    ;
    // Filesystem events are incredibly noisy. Saving a file in Vim/VSCode often 
    // generates 3 to 5 separate events instantly (Temp create, Modify, Rename, Chmod).
    // To remain "non-wasteful towards the CPU", we implement a micro-throttle to 
    // prevent spawning 4 instances of your command in the exact same millisecond.
    let mut last_trigger = Instant::now() - Duration::from_secs(1);
    let throttle_window = Duration::from_millis(50);
    loop {
        match receiver.recv().await {
            Some(Ok(event)) => {
                // Ignore pure "Access" (read) events to save CPU, we only care about mutations
                if let EventKind::Access(_) = event.kind {
                    continue;
                }
                let now = Instant::now();
                if now.duration_since(last_trigger) < throttle_window {
                    continue; // Skip the noisy sub-events of a single action
                }
                last_trigger = now;
                let changed_file = event
                    .paths
                    .first()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| target_path.to_string());

                if cmd_args.is_empty() {
                    println!("{}", changed_file);
                } else {
                    fire_command_without_blocking_event_loop(cmd_args, &changed_file);
                }
            }
            Some(Err(e)) => eprintln!("OS watcher error: {:?}", e),
            None => {
                eprintln!("OS event stream closed unexpectedly.");
                break;
            }
        }
    }
}

fn fire_command_without_blocking_event_loop(args: &[String], path: &str) {
    let replaced_args: Vec<String> = 
        args.iter().map(|arg| arg.replace("{}", path)).collect();
    let mut cmd = Command::new(&replaced_args[0]);
    if replaced_args.len() > 1 {
        cmd.args(&replaced_args[1..]);
    }
    // Prevent interactive shells (like zsh -i) from stealing the terminal input
    cmd.stdin(Stdio::null());
    // We use .spawn() instead of .output()!
    // Fork the process in microseconds and return immediately.
    if let Err(e) = cmd.spawn() {
        eprintln!("Failed to execute command: {}", e);
    }
}
