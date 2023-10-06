#!/bin/bash

cd remindy

cargo-deb

sudo dpkg -i target/debian/remindy_*_amd64.deb

cd ..
