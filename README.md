## Description

Remindy is a little reminder app with vim inspired commands.
You can create and delete reminder, and do some things with them like renaming.

## Installation

There is currently a small install script in the remindy folder.
That script requires you to have 'cargo-deb' and 'dpkg' on your system.

## Repository structure

The repo holds a `remindy` folder with the main app.
This is, because there will be more clients in the future.

There is also a `.remindy` folder, that holds my example configuration.
The installscript will copy the folder to your home dir.

## More features

There is a small rest api build in, that lets you create new reminder and retreive a list of the current ones.
This exists, because of the plans for further clients, that can use the main remindy app as a server.

## Requirements

As of now, the following requirements are needed to install remindy with rust:
- openssl
- libssl-dev
- cmake
- gtk-3-dev

These came up during development. It could well be, that this list shrinks later on!
