#!/bin/bash

cd remindy

cargo install cargo-deb

cargo-deb --install

cd ..
