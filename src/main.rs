// Required for Rocket code generation to work
#![feature(decl_macro)]

use structopt::StructOpt;
mod server;
mod strava;

#[derive(Debug, StructOpt)]
enum Args {
    #[structopt(name = "setup", about = "Authenticates the user of the Args app")]
    Setup {},
    #[structopt(name = "summary", about = "Get the activities for the day")]
    Summary {},
    #[structopt(
        name = "splits",
        about = "Gets the data stream and calculates the splits"
    )]
    Splits {
        interval: String, // mile, 1k, etc
    },
}

fn main() {
    let args = Args::from_args();
    match args {
        Args::Setup {} => strava::auth_new_user(),
        Args::Summary {} => strava::get_summary(),
        Args::Splits { interval } => strava::get_splits(interval),
    }
}
