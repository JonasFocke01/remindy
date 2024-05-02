#!/bin/bash

cp dbg_db ../../.remindy -r
cat ../../.remindy/remindy.example.toml | vipe | tee ../../.remindy.toml
rm ../../.remindy/remindy.example.toml

sudo apt install openssl -y
sudo apt install libssl-dev -y
sudo apt install cmake -y
sudo apt install gtk-3-dev -y
sudo apt install libasound2-dev -y
sudo apt install libwebkit2gtk-4.0 -y

cargo build -r

ln -s /home/$(whoami)/repos/remindy/target/release/i3blocks_client /home/$(whoami)/.config/i3blocks/scripts/remindy/remindy
