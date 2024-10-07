use std::string::String;
use clap::Parser;

mod watch;


#[derive(Parser)]
struct MyArgs {
    #[arg(short,long,required=true)]
    path: String 
}


fn main() {
    let args = MyArgs::parse();
    println!("INFO: provided path is {}", args.path);

    let discord_url = match std::env::var("DISCORD_URL") {
        Ok(var) => var,
        Err(err) => panic!("DISCORD_URL env var must be set: {}", err),
    };

    let mut watch = watch::new(args.path.as_str(), &discord_url);
    watch.run().expect("error running mc notify log watch");
}

/*
fn read(buf:&mut BufReader<File>){
    // start at the end, we only want to read new lines
    let pos_end = std::io::SeekFrom::End(0);
    let _nread = buf.seek(pos_end);
}

fn watch(path:&str) -> Result<(), notify::Error>{
    let mut watcher = notify::recommended_watcher(|res| {
        match res {
           Ok(event) => println!("event: {:?}", event),
           Err(e) => println!("watch error: {:?}", e),
        }
    })?;
    watcher.watch(Path::new(path), RecursiveMode::NonRecursive)?;

    return Ok(());
}


*/
