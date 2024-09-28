use std::io;
use std::string::String;
use std::fs::File;
use std::result::Result;
use clap::Parser;

#[derive(Parser)]
struct MyArgs {
    #[arg(short,long,required=true)]
    path: String 
}


fn main() {
    let args = MyArgs::parse();
    println!("INFO: provided path is {}", args.path);

    match read(args.path.as_str()) {
        Ok(_) => {
            std::process::exit(0);
        },
        Err(err) => {
            println!("ERR: {}", err.as_str());
            std::process::exit(1);
        },
    };
}


fn read(path:&str) -> Result<(), String> {
    let file:File = match File::open(path) {
        Ok(file) => file,
        Err(err) => return Err(format!("could not open file {}", err)), 
    };

    let reader = std::io::BufReader::new(file);

    // start at the end, we only want to read new lines
    let pos = std::io::SeekFrom::End(0);
    reader.seek(pos);

    Ok(())
}
