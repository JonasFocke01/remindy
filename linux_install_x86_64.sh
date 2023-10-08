#!/bin/bash

cd remindy

cargo install cargo-deb

cargo-deb --install

cp dbg_db $HOME/.remindy -r

cd ..
