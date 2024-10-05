use std;
use std::fs;
use std::result::Result;
use std::io::BufReader;
use inotify::{Inotify, WatchMask, EventMask};

pub struct McLogsWatch {
    path: String,
}

pub fn new(path :&str) -> McLogsWatch {
    let new_watch = McLogsWatch{
        path: String::from(path),
    };
    return new_watch;
}

fn open_file(path:&str) -> Result<BufReader<fs::File>, std::io::Error>{
    let file:fs::File = fs::File::open(path)?;
    let reader = BufReader::new(file);
    return Ok(reader);
}

/*  
    This is how we can wrap the error with our own. Box<dyn> is not ideal.
    With Rust, usually, we would want strongly typed / defined errors

fn open_file(path:&str) -> Result<BufReader<fs::File>, Box<dyn std::error::Error>>{
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(err) => return Err(format!("could not open file. err: {}", err).into()),
    };
    let reader = BufReader::new(file);
    return Ok(reader);
}
*/


impl McLogsWatch {
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let file = open_file(self.path.as_str())?;

        let mut inotify = Inotify::init()
            .expect("Error while initializing inotify instance");

        let canonpath = std::fs::canonicalize(self.path.as_str())?;
        let p = std::path::Path::new(&canonpath);

        let currdir = std::env::current_dir().unwrap();
        let parentdir = p.parent().unwrap_or(&currdir);

        inotify.watches()
            .add(self.path.as_str(),WatchMask::MODIFY | WatchMask::DELETE_SELF | WatchMask::DELETE)
            .expect("Failed to add file watch");

        inotify.watches()
            .add(parentdir.to_str().unwrap(), WatchMask::DELETE | WatchMask::DELETE_SELF)
            .expect("Failed to add parent file dir watch");

        println!("Watching {}", self.path);

        let mut buffer = [0; 1024];
        loop {
            let events = inotify
                .read_events_blocking(&mut buffer)
                .expect("Failed to read inotify events");

            for event in events {
                let isdir = event.mask.contains(EventMask::ISDIR);
                let name = event.name.unwrap_or_default().to_str().unwrap_or_default();

                if name != p.file_name().unwrap() && name != "" { // empty for the direct file
                    println!("ignoring event on {}", name);
                    // ignore irrelevant files in dir
                    continue
                }

                if event.mask.contains(EventMask::DELETE) {
                    println!("received {:?} event, exitting", event.mask);
                    return Ok(());
                } else if event.mask.contains(EventMask::MODIFY) { 
                        println!("received MODIFY event");
                } else {
                    println!("WARN: unknown event mask received {:?}", event.mask);
                }

            }
        }
    } 
}
