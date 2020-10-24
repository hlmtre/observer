extern crate sysinfo;

use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::{BufRead, BufReader};
use std::{fs::File, process};
use std::{process::Command, thread, time};
use sysinfo::{ProcessExt, ProcessStatus, RefreshKind, SystemExt};
use time::Duration;

#[derive(Default, Debug)]
struct Obs {
  trigger_process_name: String,
  target_process_path: String,
  target_process_name: String,
  target_args: String,
}

impl Obs {
  fn is_valid(&self) -> bool {
    if self.target_process_name.len() > 0 && self.trigger_process_name.len() > 0 {
      return true;
    }
    return false;
  }
}

fn main() {
  let mut args: Vec<String> = std::env::args().collect();
  if args.len() < 2 {
    // assume it's here, and it's called observer.conf
    args.push("observer.conf".to_string());
  }
  let obs_or_err = open_config(args[1].as_str());
  let o = match obs_or_err {
    Ok(k) => k,
    Err(e) => {
      eprintln!("error within or opening ./observer.conf. error: {}", e);
      process::exit(1);
    }
  };

  // all this string type juggling is to
  // prevent temporary value dropped while borrowed
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
    let processes = s.get_process_by_name(tr);
    let target_processes = s.get_process_by_name(ta);
    if cfg!(debug_assertions) {
      eprintln!("looking for process {}", tr);
    }
    if processes.len() > 0 {
      if cfg!(debug_assertions) {
        eprintln!("found process {}!", tr);
      }
      if target_processes.len() > 0 {
        let tp = target_processes[0]; // we're guaranteed to have at least one...
        #[cfg(target_os = "linux")]
        {
          match tp.status() {
            ProcessStatus::Run | ProcessStatus::Sleep | ProcessStatus::Idle => {
              if cfg!(debug_assertions) {
                eprintln!("target already running");
              }
              thread::sleep(Duration::from_millis(5000));
              continue;
            }
            _ => {
              // any other case...
              if cfg!(debug_assertions) {
                eprintln!("spawning process {}", target_process_name);
              }
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
              if cfg!(debug_assertions) {
                eprintln!(
                  "target {} already running. {} instances already.",
                  target_process_path,
                  target_processes.len()
                );
              }
              thread::sleep(Duration::from_millis(5000));
              continue;
            }
          }
        }
      } else {
        if cfg!(debug_assertions) {
          eprintln!("spawning process {}", target_process_path);
        }
        let mut a: Vec<&str> = Vec::new();
        let pdir = std::path::Path::new(target_process_path.as_str())
          .parent()
          .unwrap();
        let _ = std::env::set_current_dir(pdir);
        let mut c = Command::new(target_process_path.clone());
        // if arguments are specified in the config file
        if o.target_args.len() > 0 {
          let s = o.target_args.replace("\"", "");
          a.append(s.split(" ").collect::<Vec<&str>>().as_mut());
          // remove that initial empty string item
          // some programs see the empty string and get mad
          a.retain(|&i| i.len() > 0);
          c.args(a);
        }
        c.spawn().expect("ohtehnoes");
        thread::sleep(Duration::from_millis(1000));
      }
    } else {
      thread::sleep(Duration::from_millis(5000));
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
    } else if a.starts_with("target_process_path") {
      let el: Vec<&str> = a.split("=").collect();
      o.target_process_path = el[1].to_string();
    } else if a.starts_with("target_args") {
      let el: Vec<&str> = a.split("=").collect();
      o.target_args = el[1].to_string();
    }
  }
  if o.target_process_path.len() > 0 {
    o.target_process_name = o.target_process_path.clone()
  }
  if !o.is_valid() {
    return Err(Error::new(ErrorKind::Other, "invalid config!"));
  }
  Ok(o)
}
