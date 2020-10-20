extern crate sysinfo;

use std::{process::Command, thread, time};
use sysinfo::{ProcessExt, ProcessStatus, RefreshKind, SystemExt};

fn main() {
  let trigger_process_name = "RainbowSix_Vulkan.exe";
  let target_process_path = "C:\\Program Files\\obs-studio\\bin\\64bit\\obs64.exe";
  let target_process_name = "obs64.exe";
  let r = RefreshKind::new();
  let mut s = sysinfo::System::new_with_specifics(r.with_processes());
  loop {
    s.refresh_processes();
    let processes = s.get_process_by_name(trigger_process_name);
    let target_processes = s.get_process_by_name(target_process_name);
    eprintln!("looking for process {}", trigger_process_name);
    if processes.len() > 0 {
      eprintln!("found process {}!", trigger_process_name);
      if target_processes.len() > 0 {
        let tp = target_processes[0]; // we're guaranteed to have at least one...
        #[cfg(target_os = "linux")]
        {
          match tp.status() {
            ProcessStatus::Run | ProcessStatus::Sleep | ProcessStatus::Idle => {
              eprintln!("target already running");
              thread::sleep(time::Duration::from_millis(5000));
              continue;
            }
            _ => {
              // any other case...
              eprintln!("spawning process {}", target_process_name);
              Command::new(target_process_name)
                .spawn()
                .expect("failed to start target process");
            }
          }
        }
        // Windows has only Run in the ProcessStatus enum
        #[cfg(target_os = "windows")]
        {
          match tp.status() {
            ProcessStatus::Run => {
              eprintln!(
                "target {} already running. {} instances already.",
                target_process_path,
                target_processes.len()
              );
              thread::sleep(time::Duration::from_millis(5000));
              continue;
            }
          }
        }
      } else {
        eprintln!("spawning process {}", target_process_path);
        let p = std::path::Path::new("C:\\Program Files\\obs-studio\\bin\\64bit");
        std::env::set_current_dir(&p);
        Command::new(target_process_path).args(&["--startreplaybuffer"])
          .spawn()
          .expect("failed to start target process");
        // so i don't blow my computer up with ~15GB of chrome instances starting as fast as possible
        if cfg!(debug_assertions) {
          thread::sleep(time::Duration::from_millis(1000));
        }
      }
    } else {
      thread::sleep(time::Duration::from_millis(5000));
    }
  }
}
