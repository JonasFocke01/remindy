# Remindy

Remindy is a little terminal tool to remind me of something.

# How to use

type in Terminal:
`remindy testmeeting 15m`
This will spawn a notification in 15 minutes for testmeeting.
or
`remindy testmeeting 15:00`
This will spawn a nofification at 15 O'Clock for testmeeting.
You must keep the terminal window open!

# How to compile with cargo

- run `cargo-deb`
- run `cd /target/debian` (or what other directory it is compiling to)
- sudo dpkg -i <Package_name>
