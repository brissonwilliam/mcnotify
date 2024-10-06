use std::borrow::Borrow;
use std::{self, io};
use std::error::{Error};
use std::fs;
use std::fmt;
use std::os::unix::fs::MetadataExt;
use std::result::Result;
use std::io::{BufRead, BufReader, Seek};
use inotify::{Inotify, WatchMask, EventMask};

pub struct McLogsWatch {
    path: String,
    lastsize: u64,
    buf: Option<BufReader<fs::File>>,
}

pub fn new(path :&str) -> McLogsWatch {
    let new_watch = McLogsWatch{
        path: String::from(path),
        lastsize: 0,
        buf: None,
    };
    return new_watch;
}

#[derive(Debug)]
pub struct ErrModify {
    err: Box<dyn Error>,
}

fn err_modify(source:Box<dyn Error>) -> ErrModify {
    return ErrModify{err: source};
}

impl std::fmt::Display for ErrModify {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "modify event error: {}", self.err)
    }
}
impl Error for ErrModify {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        return Some(self.err.borrow())
    }
}

impl McLogsWatch {
    fn open_file(&mut self) -> Result<(), io::Error> {
        println!("Opening {} to end of file", self.path.as_str());

        let md = fs::metadata(self.path.as_str())?;
        let file:fs::File = fs::File::open(self.path.as_str())?;
        let mut reader = BufReader::new(file);

        let eof = io::SeekFrom::End(0);
        reader.seek(eof)?;

        self.buf = Some(reader);
        self.lastsize = md.size();

        return Ok(());
    }

    fn on_modify(&mut self) -> Result<(), Box<dyn Error>> {
        let md = fs::metadata(self.path.as_str())?;

        if self.lastsize == 0 || md.size() < self.lastsize { 
            println!("File has changed size, reopening");
            self.open_file()?;
        }

        let mut reader = self.buf.take().unwrap();

        let mut n = 1;
        while n > 0 {
            let mut str = String::new();
            n = reader.read_line(&mut str)?;

            if n == 0 {
                break
            }

            print!("read {}", str.as_str());
        }
        self.buf = Some(reader);

        return Ok(());
    }


    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut inotify = Inotify::init()
            .expect("Error while initializing inotify instance");

        let canonpath = std::fs::canonicalize(self.path.as_str())?;
        let p = std::path::Path::new(&canonpath);

        let currdir = std::env::current_dir().unwrap();
        let parentdir = p.parent().unwrap_or(&currdir);

        let file_flags = WatchMask::DELETE_SELF | WatchMask::DELETE | WatchMask::MODIFY | WatchMask::CLOSE_WRITE | WatchMask::MOVE_SELF;
        inotify.watches()
            .add(self.path.as_str(), file_flags)
            .expect("Failed to add file watch");

        inotify.watches()
            .add(parentdir.to_str().unwrap(), WatchMask::DELETE | WatchMask::DELETE_SELF | WatchMask::CREATE)
            .expect("Failed to add parent file dir watch");

        println!("Watching {}", self.path);

        self.open_file()?;

        let mut buffer = [0; 1024];
        loop {
            let events = inotify
                .read_events_blocking(&mut buffer)
                .expect("Failed to read inotify events");

            for event in events {
                // let isdir = event.mask.contains(EventMask::ISDIR);
                let name = event.name.unwrap_or_default().to_str().unwrap_or_default();

                if name != p.file_name().unwrap() && name != "" { // empty for the direct file
                    // ignore irrelevant files in dir
                    // println!("ignoring event on {}", name);
                    continue
                }

                if event.mask.contains(EventMask::DELETE) || event.mask.contains(EventMask::DELETE_SELF) || event.mask.contains(EventMask::MOVE_SELF) {
                    // the inotify watch will become invalid and unusable, so remove it
                    // move will happen with vim
                    println!("received {:?} event, stopping {}", event.mask, event.wd.get_watch_descriptor_id());
                    inotify.watches().remove(event.wd)?;
                    self.lastsize = 0;

                } else if event.mask.contains(EventMask::CREATE) {
                    println!("received {:?} event on watched path, adding a watch", event.mask);
                    inotify.watches()
                        .add(self.path.as_str(), file_flags)
                        .expect("Failed to add file watch");
                    self.lastsize = 0;

                } else if event.mask.contains(EventMask::MODIFY) || event.mask.contains(EventMask::CLOSE_WRITE) {  
                    println!("received {:?} event", event.mask);
                    match self.on_modify() {
                        Ok(_) => (),
                        Err(err) => println!("received error on modify event: {}", err),
                    };

                } else {
                    println!("WARN: unknown event mask received {:?}", event.mask);
                }

            }
        }
    } 
}
