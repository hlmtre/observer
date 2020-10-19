extern crate sysinfo;

use std::process::Command;
use std::{thread, time};
use sysinfo::ProcessStatus;
use sysinfo::{ProcessExt, RefreshKind, SystemExt};

fn main() {
  let trigger_process_name = "htop";
  let target_process_name = "pavucontrol";
  let r = RefreshKind::new();
  let mut s = sysinfo::System::new_with_specifics(r.with_processes());
  loop {
    s.refresh_processes();
    let processes = s.get_process_by_name(trigger_process_name);
    let target_processes = s.get_process_by_name(target_process_name);
    println!("looking for process {}", trigger_process_name);
    if processes.len() > 0 {
      println!("found process {}!", trigger_process_name);
      if target_processes.len() > 0 {
        for tp in target_processes {
          match tp.status() {
            ProcessStatus::Run | ProcessStatus::Sleep | ProcessStatus::Idle => {
              // our target is already running
              println!("target already running");
              thread::sleep(time::Duration::from_millis(5000));
              continue;
            }
            _ => {
              // any other case...
              println!("spawning process {}", target_process_name);
              Command::new(target_process_name)
                .spawn()
                .expect("failed to start target process");
            }
          }
        }
      } else {
        println!("spawning process {}", target_process_name);
        Command::new(target_process_name)
          .spawn()
          .expect("failed to start target process");
      }
    } else {
      thread::sleep(time::Duration::from_millis(5000));
    }
  }
}
