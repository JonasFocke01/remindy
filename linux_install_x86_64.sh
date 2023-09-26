#!/bin/bash

cd remindy

cargo-deb

sudo dpkg -i target/debian/remindy_0.1.0_amd64.deb

cd ..
