## Description

Remindy is a little reminder app with vim inspired commands.  
You can create and delete reminder, and do some things with them like renaming.  
Remindy has multiple clients, that can all communicate to the same server. Disclaimer a client cannont run in standallone mode, it needs a server.

## Installation

There is no *standard* way to install, as remindy is still in early access. Therefore no binarys are supplied.  
The easyest would be to utilize cargo-deb and install the generated package via dpgk (debian based).  
You also need to add a folder in your home directory named `.remindy`. Refer to the `debug_db` folder in the repo for the content.
Important: Build remindy in release mode. Remindy also exposes some features like support for `i3`, `msg-boxes`...

## Repository structure

The repository contains multiple rust workspaces:
- reminder: This holds structs and utilities for the other workspaces. This acts as a internal library.
- config: This holds the buisnesslogic for loading the config files.
- dbg_db: This holds the config files and a db file. (No need to alter this, as this gets overwritten).
- server: This holds the server app, that should be installed somewhere accessible.
- client: There is a terminal based client. This is the `vim-based` component.
- simple_js_frontend: This is a web client, that can display reminders and add new ones. It has no additional functionality as of now.

## Requirements

As of now, the following linux specific requirements are needed to install the client with cargo-deb:
- openssl
- libssl-dev
- cmake
- gtk-3-dev

These came up during development. It could well be, that this list shrinks later on, or allready shrank. (just play around if you want)

## OS

Only linux is in active development, so do not expect the client to run flawless on Windows or Macos.
The server on the other hand has no reason not to work on all OSses. (allthough untested)
