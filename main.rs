use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    let mut args = env::args().skip(1).peekable();
    let mut paths_to_watch = Vec::new();
    let cmd_args;
    let mut force = false;
    let mut paths_file = None;
    while let Some(arg) = args.peek() {
        if arg == "--force" {
            force = true;
            args.next();
        } else if arg == "--pathsfile" {
            args.next();
            paths_file = Some(args.next().unwrap_or_else(|| {
                eprintln!("Error: --pathsfile requires a file path argument");
                std::process::exit(1);
            }));
        } else {
            break;
        }
    }
    let raw_args: Vec<String> = args.collect();
    if let Some(file) = paths_file {
        let content = fs::read_to_string(&file).unwrap_or_else(|_| {
            eprintln!("Error: Failed to read pathsfile '{}'", file);
            std::process::exit(1);
        });
        for line in content.lines() {
            let pth = line.trim();
            if !pth.is_empty() {
                paths_to_watch.push(pth.to_string());
            }
        }
        cmd_args = raw_args;
    } else {
        if raw_args.is_empty() {
            eprintln!("Example Usage: whenpathchanges <path> [command...]");
            std::process::exit(1);
        }
        paths_to_watch.push(raw_args[0].clone());
        cmd_args = raw_args[1..].to_vec();
    }

    let mut exact_targets: Vec<PathBuf> = Vec::new();
    for path_str in &paths_to_watch {
        let p = Path::new(path_str);
        let abs_path = if p.is_absolute() { 
            p.to_path_buf() 
        } else { env::current_dir().unwrap().join(p) };
        exact_targets.push(abs_path);
    }
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

    for target in &exact_targets {
        if !target.exists() && !force {
            eprintln!("Error: Path does not exist {:?}", target);
            std::process::exit(1);
        }
        let watch_target = if target.is_dir() {
            target.clone()
        } else if let Some(parent) = target.parent() {
            parent.to_path_buf()
        } else {
            target.clone()
        };
        let mode = if target.is_dir() { 
            RecursiveMode::Recursive 
        } else { RecursiveMode::NonRecursive };
        if let Err(e) = watcher.watch(&watch_target, mode) {
            if !force {
                eprintln!("Failed to watch path {:?}: {}", watch_target, e);
                std::process::exit(1);
            }
        }
    }

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
                let mut matched_path = None;

                for ev_path in &event.paths {
                    let ev_abs = if ev_path.is_absolute() { 
                        ev_path.clone() 
                    } else { 
                        env::current_dir().unwrap().join(ev_path) 
                    };
                    for target in &exact_targets {
                        if target.is_dir() && ev_abs.starts_with(target) {
                            matched_path = Some(ev_abs.to_string_lossy().to_string());
                            break;
                        } else if ev_abs == *target {
                            matched_path = Some(ev_abs.to_string_lossy().to_string());
                            break;
                        }
                    }
                    if matched_path.is_some() { break; }
                }

                let changed_file = match matched_path {
                    Some(p) => p,
                    None => continue, 
                };
                let now = Instant::now();
                if now.duration_since(last_trigger) < throttle_window {
                    continue; // Skip the noisy sub-events of a single action
                }
                last_trigger = now;
                if cmd_args.is_empty() {
                    println!("{}", changed_file);
                } else {
                    fire_command_without_blocking_event_loop(&cmd_args, &changed_file);
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
