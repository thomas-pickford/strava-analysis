// Required for Rocket code generation to work
#![feature(decl_macro)]

use std::io::{self, Write};

mod server;
mod strava;
mod auth;

fn main() {
    if strava::is_setup() {
        strava::greet_user();
    }
    else {
        auth::auth_new_user()
    }
    
    println!("\nHow can I help you today?");
    println!("1. Get an overview of your running activities today");
    println!("2. Get the splits from your running activities today");
    println!("q. Quit");
    
    loop {

        let mut input = String::new();
        print!("> ");
        io::stdout().flush().expect("Failed to flush stdout");
        io::stdin().read_line(&mut input).expect("failed to read input");

        let trimmed_input = input.trim();

        match trimmed_input {
            "1" | "2" => {
                println!("Pick formatting interval (mile, 1k):");
                loop {
                    let mut interval_size = String::new();
                    print!("interval> ");
                    io::stdout().flush().expect("Failed to flush stdout");
                    io::stdin().read_line(&mut interval_size).expect("failed to read input");
                    let upper = interval_size.trim().to_ascii_uppercase();
                    match upper.as_str() {
                        "MILE" | "1K" => {
                            match trimmed_input {
                                "1" => strava::get_summary(upper),
                                "2" => strava::get_splits(upper),
                                _ => todo!(),
                            }
                            break;
                        },
                        "Q" => {
                            println!("Cancelled request");
                            break;
                        },
                        _ => println!("Unsupported distance. Please choose from the following (mile, 1k)")
                    }
                }
            },
            "q" => {
                println!("Quitting the app. Goodbye!");
                break;
            }
            _ => println!("Invalid option. Try again."),
        }
    }
}
