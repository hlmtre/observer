extern crate sysinfo;

use std::path::Path;
use std::{
  fs::File,
  io::{self, BufRead, BufReader, Error, ErrorKind},
  process::{self, Command},
  thread, time,
};
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
    if self.target_process_path.len() > 0 && self.trigger_process_name.len() > 0 {
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
  let r = RefreshKind::new();
  let mut s = sysinfo::System::new_with_specifics(r.with_processes());
  loop {
    s.refresh_processes();
    let processes = s.get_process_by_name(o.trigger_process_name.as_str());
    let target_processes = s.get_process_by_name(o.target_process_name.as_str());
    if cfg!(debug_assertions) {
      eprintln!(
        "looking for process {}; would spawn {}",
        o.trigger_process_name, o.target_process_name
      );
    }
    if processes.len() > 0 {
      if cfg!(debug_assertions) {
        eprintln!("found process {}!", o.trigger_process_name);
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
                eprintln!("spawning process {}", o.target_process_name);
              }
              Command::new(o.target_process_path.clone())
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
                  o.target_process_path,
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
          eprintln!("spawning process {}", o.target_process_path);
        }
        let mut a: Vec<&str> = Vec::new();
        let pdir = std::path::Path::new(o.target_process_path.as_str())
          .parent()
          .unwrap();
        let _ = std::env::set_current_dir(pdir);
        let mut c = Command::new(o.target_process_path.clone());
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
  let mut trigger_process_name = String::new();
  let mut target_process_path = String::new();
  let mut target_args = String::new();
  for l in b.lines() {
    let a = l.unwrap();
    if a.starts_with("trigger_process_name") {
      let el: Vec<&str> = a.split("=").collect();
      trigger_process_name = el[1].to_string();
    } else if a.starts_with("target_process_path") {
      let el: Vec<&str> = a.split("=").collect();
      target_process_path = el[1].to_string();
    } else if a.starts_with("target_args") {
      let el: Vec<&str> = a.split("=").collect();
      target_args = el[1].to_string();
    }
  }
  let trigger_process_name = trigger_process_name.replace("\"", "");
  let target_process_exe = std::ffi::OsString::from(
    Path::new(target_process_path.replace("\"", "").trim())
      .file_name()
      .unwrap(),
  )
  .into_string()
  .unwrap();
  o.trigger_process_name = trigger_process_name.trim().to_string();
  o.target_process_path = target_process_path.replace("\"", "").trim().to_string();
  o.target_process_name = target_process_exe.clone();
  o.target_args = target_args;

  if cfg!(debug_assertions) {
    eprintln!(
      "Config: 
        trigger process: {:#?},
        target process path: {:#?},
        target process name: {:#?}",
      o.trigger_process_name, o.target_process_path, o.target_process_name
    );
  }

  if !o.is_valid() {
    return Err(Error::new(ErrorKind::Other, "invalid config!"));
  }
  Ok(o)
}
