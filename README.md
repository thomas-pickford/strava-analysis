# Setup
Requires a 'dev' or 'nightly' version of rust to run. Ex: `rustup override set nightly`

# How to use
`cargo run -- <option>`

### Valid options
**setup:** Used to authenticate the user's strava account with the API app that powers this CLI
**summary:** Returns the summary of the running activites for the day
**splits <interval>:** Returns a json file with details about each lap interval (user specified) such as the distance, pace, time
- Valid interval examples
- mile
- 1k

# Updates
### Release 1
- Ability to authenticate the a user with the CLI
- Stores the users auth details for later use
- Placeholder functions for summary and splits options

# Acknowledgments
Strava Authentication - https://francoisbest.com/posts/2019/strava-auth-cli-in-rust
