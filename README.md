# Setup
Requires a 'dev' or 'nightly' version of rust to run. Ex: `rustup override set nightly`

At this time it is required that a user provides their own Strava API app credentials. Those need to be created and placed inside 
a directory called auth in a file named secrets.json. To setup your own app follow the instructions [here]("https://developers.strava.com/docs/getting-started/").
```
// Example secrets.json
{
    "client_id" : <your_id>,
    "client_secret" : <your_secret>
}
```

# Overview
This strava analysis app enables users to get a few different overviews of activities from current day, current week or a specified date range.
Users can also get splits from an activity from the current day or a specified date range. When requesting laps they are calculated from the activities
data stream and stored in a JSON file named with the format `<month>-<day>-<activity_id>.json`. These can be used later for different types of analysis
whether implemented by me in future versions or users. 

# How to use
Start the program: `cargo run`
Choose an option from the menu
- 1 - Get an overview of your running activities today
- 2 - Get the splits from your running activities today
- 3 - Get the splits from activities in a date range (mm/dd/yyyy)
- 4 - Get an overview of activities in a date range (mm/dd/yyyy)
- 5 - Get an overview of this weeks totals
- q. Quit

# Updates
### Version 0.1.0
- Ability to authenticate the a user with the CLI
- Stores the users auth details for later use
- Gives a rough outline of activity details in the summary
- Retrieves the data streams and prints a count of how many data points are in each activity stream

### Version 0.1.1
- Implement `get_summary` to print overview of activity stats
- Format time h : m : s instead of m:s

### Version 0.2.0
- Restructure modules to create a strava crate. Expandable to encorporate more/all of the strava API
- Complete data stream analysis to get lap splits 
- Store activity laps in json file
- Option to get a summary of week activities

### Next steps
- Format summary with user selected distances
- Get splits from data stream using user defined distances

# Acknowledgments
Strava Authentication - https://francoisbest.com/posts/2019/strava-auth-cli-in-rust
Strava API - https://developers.strava.com/docs/reference/
