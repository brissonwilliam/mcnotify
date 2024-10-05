use std;
use std::fs;
use std::path::Path;
use std::result::Result;
use std::io::{BufReader, Seek};
use std::time;
use std::thread;
use notify::{PollWatcher, RecommendedWatcher, Watcher};

pub struct McLogsWatch {
    path: String,
    stop: bool,
}

pub fn new(path :&str) -> McLogsWatch {
    let mut stop = false;
    let new_watch = McLogsWatch{
        path: String::from(path),
        stop: stop,
    };
    return new_watch;
}

/*
fn open_file(path:&str) -> Result<BufReader<fs::File>, std::io::Error>{
    let file:fs::File = fs::File::open(path)?;
    let reader = BufReader::new(file);
    return Ok(reader);
}
*/


fn open_file(path:&str) -> Result<BufReader<fs::File>, Box<dyn std::error::Error>>{
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(err) => return Err(format!("could not open file. err: {}", err).into()),
    };
    let reader = BufReader::new(file);
    return Ok(reader);
}


const POLL_FREQ:time::Duration = time::Duration::from_secs(10);

impl McLogsWatch {

    fn handle_event(&mut self, res:Result<notify::Event, notify::Error>) {
        let event = match res {
           Ok(event) => event,
           Err(err) => {
                println!("watch poll event error: {}", err);
                return
           },
        };


        match event.kind {
            notify::EventKind::Any => (),
            notify::EventKind::Access(access_kind) => (),
            notify::EventKind::Create(create_kind) => (),
            notify::EventKind::Modify(modify_kind) => {

            },
            notify::EventKind::Remove(remove_kind) => self.stop = true,
            notify::EventKind::Other => (),
        }
    }


    pub fn run(&mut self) -> Result<(), Box<dyn core::error::Error>> {

        let file = match open_file(self.path.as_str()) {
            Ok(it) => it,
            Err(err) => return Err(err),
        };


        // let mut watcher = notify::RecommendedWatcher::new(handle_event, config)?;

        // create a new watcher

        let mut stop = false;

        let config = notify::Config::default().with_manual_polling();

        let mut watcher = notify::PollWatcher::new(|res| {
            let event: notify::Event = match res {
                Ok(event) => event,
                Err(err) => {
                    println!("event handler error: {}", err);
                    return;
                },
            };

            match event.kind {
                notify::EventKind::Any => (),
                notify::EventKind::Access(access_kind) => (),
                notify::EventKind::Create(create_kind) => (),
                notify::EventKind::Modify(modify_kind) => {

                },
                notify::EventKind::Remove(remove_kind) => stop = false,
                notify::EventKind::Other => (),
            }
        }, config)?;

        // add the actual file path to the watch list
        watcher.watch(Path::new(self.path.as_str()), notify::RecursiveMode::NonRecursive)?;

        loop {
            if self.stop {
                println!("stop received, exitting");
                break;
            }

            let start = time::Instant::now();

            println!("tick, polling");
            watcher.poll().map_err(|err| {
                println!("poll error: {}", err);
                return err;
            })?;

            let end = time::Instant::now();
            let elapsed = time::Instant::duration_since(&end, start);

            let wait_time = POLL_FREQ - elapsed;        
            thread::sleep(wait_time);
        }


        Ok(())
    } 
}
