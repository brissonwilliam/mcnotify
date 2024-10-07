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
