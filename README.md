# Setup
Requires a 'dev' or 'nightly' version of rust to run. Ex: `rustup override set nightly`

At this time it is required that a user provides their own Strava API app credentials. Those need to be created and placed inside 
a directory called auth in a file named secrets.json. secrets.json looks something like 
```
{
    "client_id" : <your_id>,
    "client_secret" : <your_secret>
}
```

# How to use
Start the program: `cargo run`
Choose an option from the menu
- 1. Get an overview of yoru running activities today
- 2. Get the splits from your running activities today (will add a interval option in the future)
- q. Quit

# Updates
### Release 1
- Ability to authenticate the a user with the CLI
- Stores the users auth details for later use
- Gives a rough outline of activity details in the summary
- Retrieves the data streams and prints a count of how many data points are in each activity stream

### Next steps
- Format summary with user selected distances
- Get splits from data stream using user defined distances

# Acknowledgments
Strava Authentication - https://francoisbest.com/posts/2019/strava-auth-cli-in-rust
