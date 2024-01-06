## Description

Remindy is a little reminder app with vim inspired commands.  
You can create and delete reminder, and do some things with them like renaming.  
Remindy has multiple clients, that can all communicate to the same server.

## Installation

There is no *standard* way to install, as remindy is still in early access. Therefore no binarys are supplied.  
The easyest would be to utilize cargo-deb and install the generated package via dpgk (debian based).  
Disclaimer: You would also need to change the hardcoded ip in the [clients main file](https://github.com/JonasFocke01/remindy/blob/master/client/src/main.rs) (line 23 as of now). 

## Repository structure

The repository contains multiple rust workspaces:
- reminder: This holds structs and utilities for the other workspaces. This acts as a internal library.
- server: This holds the server app, that should be installed somewhere accessible.
- client: Currently, there is only one client, that runns in a terminal. This is the `vim based` component.

## Requirements

As of now, the following requirements are needed to install the client with cargo-deb:
- openssl
- libssl-dev
- cmake
- gtk-3-dev

These came up during development. It could well be, that this list shrinks later on!
