extern crate sysinfo;

use std::fs::File;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::{BufRead, BufReader};
use std::{process::Command, thread, time};
use sysinfo::{ProcessExt, ProcessStatus, RefreshKind, SystemExt};

#[derive(Default, Debug)]
struct Obs {
  trigger_process_name: String,
  target_process_path: String,
  target_process_name: String,
}

impl Obs {
  fn is_valid(&self) -> bool {
    if self.trigger_process_name.len() > 0
      && self.target_process_name.len() > 0
      && self.target_process_path.len() > 0
    {
      return true;
    }
    return false;
  }
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let o: Obs = open_config(args[1].as_str()).unwrap();
  let trigger_process_name = o.trigger_process_name.replace("\"", "");
  let f = o.target_process_path.replace("\"", "");
  let target_process_path = f.trim().to_string();
  let target_process_name = o.target_process_name.replace("\"", "");
  let tr: &str = trigger_process_name.trim();
  let ta: &str = target_process_name.trim();
  let r = RefreshKind::new();
  let mut s = sysinfo::System::new_with_specifics(r.with_processes());
  loop {
    s.refresh_processes();
    // prevent temporary value dropped while borrowed
    let processes = s.get_process_by_name(tr);
    let target_processes = s.get_process_by_name(ta);
    eprintln!("looking for process {}", tr);
    if processes.len() > 0 {
      eprintln!("found process {}!", tr);
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
              Command::new(target_process_name.clone())
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
        //let p = std::path::Path::new(target_process_path.as_str());
        //std::env::set_current_dir(&p);
        eprintln!("{:#?}", target_process_path);
        Command::new(target_process_path.clone())
          //.args(&["--startreplaybuffer"])
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

fn open_config(path: &str) -> Result<Obs, io::Error> {
  let mut o = Obs::default();
  let f = File::open(path)?;
  let b = BufReader::new(f);
  for l in b.lines() {
    let a = l.unwrap();
    if a.starts_with("trigger_process_name") {
      let el: Vec<&str> = a.split("=").collect();
      o.trigger_process_name = el[1].to_string();
    } else if a.starts_with("target_process_name") {
      let el: Vec<&str> = a.split("=").collect();
      o.target_process_name = el[1].to_string();
    } else if a.starts_with("target_process_path") {
      let el: Vec<&str> = a.split("=").collect();
      o.target_process_path = el[1].to_string();
    }
  }
  if !o.is_valid() {
    return Err(Error::new(ErrorKind::Other, "invalid config!"));
  }
  return Ok(o);
}
