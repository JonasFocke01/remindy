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

- cargo build
- cd /target/debug
- mkdir deb_release_build
- mv remindy deb_release_build/bin
- mkdir deb_release_build/etc
- mdkir deb_release_build/DEBIAN
- touch deb_release_build/DEBIAN/control
- vim deb_release_build/DEBIAN/control
  - _File content:_
  - Package: Remindy
  - Version: 1.0
  - Section: utils
  - Priority: optional
  - Architecture: all
  - Maintainer: me <me@me.de>
  - Description: This is Remindy
- sudo dpkg --build deb_release_build/
- sudo dpkg -i deb_release_build.deb
