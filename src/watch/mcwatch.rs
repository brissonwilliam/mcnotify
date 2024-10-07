use std::{self, io};
use std::error::Error;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::result::Result;
use std::io::{BufRead, BufReader, Seek};
use reqwest;
use inotify::{Inotify, WatchMask, EventMask};
use super::event::{McLogEvent};
use super::event;


pub(crate) struct McLogsWatch {
    path: String,
    discord_url: String,
    filepos: u64,
    buf: Option<BufReader<fs::File>>,
}

pub fn new(path :&str, discord_url :&str) -> McLogsWatch {
    let new_watch = McLogsWatch{
        path: String::from(path),
        filepos: 0,
        discord_url: String::from(discord_url),
        buf: None,
    };
    return new_watch;
}

/* Not really using this anymore, but keeping jsut as reference on how custom errors should be made
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
*/

const SEEK_EOF:io::SeekFrom = io::SeekFrom::End(0);

impl McLogsWatch {

    fn push_notif(&self, event:McLogEvent) -> Result<(), Box<dyn Error>> {
        println!("DBUG: pushing to {}", self.discord_url);

    let body = format!(r#"
{{
  "content": "",
  "embeds": [{{
    "title": "mfsbuilds MC server event",
    "description": "{}",
    "color": "{}"
  }}]
}}
"#, event.msg, event.color);

       let client = reqwest::blocking::Client::new();
       let resp = client.post(self.discord_url.as_str())
        .body(body)
        .header("Content-Type", "application/json") 
        .send()?;

        println!("DEBUG: got response {}", resp.status());
       /*
        .method("POST")
        .uri(self.discord_url.as_str())
        .body(body); 
       */



        return Ok(())
    }

    fn on_modify(&mut self) -> Result<(), Box<dyn Error>> {
        let md = fs::metadata(self.path.as_str())?;

        if self.filepos == 0 || self.filepos > md.size() { 
            println!("DEBUG: filepos is 0 or would overflow, reopening to end of file");
            self.open_file_at(SEEK_EOF)?;
        }

        let mut reader = self.buf.take().unwrap();

        let mut n = 1;
        while n > 0 {
            let mut str = String::new();
            n = reader.read_line(&mut str)?;

            if n == 0 {
                break
            }

            self.filepos += n as u64;
            println!("DEBUG: read line of {} bytes. filepos is now {}", n, self.filepos);

            let log_event = match event::get_log_event(str.as_str()) {
                Some(event) => event,
                None => continue,
            };

            match self.push_notif(log_event){
                Ok(_) => {},
                Err(err) => {
                    println!("ERR: could not push notification: {}", err);
                    continue
                },
            };
        }
        self.buf = Some(reader);

        return Ok(());
    }

    fn open_file_at(&mut self, pos:io::SeekFrom) -> Result<(), io::Error> {
        println!("DEBUG: Opening a buffer for {}", self.path.as_str());

        let file:fs::File = fs::File::open(self.path.as_str())?;
        let mut reader = BufReader::new(file);

        self.filepos = reader.seek(pos)?;
        self.buf = Some(reader);
        println!("DEBUG: succesfully opened buffer at filepos {}", self.filepos);

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

        println!("INFO: Watching {}", self.path);

        self.open_file_at(SEEK_EOF)?;

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
                    println!("DEBUG: received {:?} event, stopping watch descriptor #{}", event.mask, event.wd.get_watch_descriptor_id());
                    inotify.watches().remove(event.wd)?;

                } else if event.mask.contains(EventMask::CREATE) && name == p.file_name().unwrap() {
                    // CREATE will happen if the file is moved, which happens with vim and
                    // potentially in othe ways
                    // We have re-add the watch and re open the file
                    println!("DEBUG: received {:?} event on watched path, adding new watch", event.mask);
                    inotify.watches()
                        .add(self.path.as_str(), file_flags)
                        .expect("Failed to add file watch");

                    let pos = io::SeekFrom::Start(self.filepos); // try to keep the latest post
                    self.open_file_at(pos)?;

                } else if event.mask.contains(EventMask::MODIFY) || event.mask.contains(EventMask::CLOSE_WRITE) {  
                    println!("DEBUG: received {:?} event", event.mask);
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
