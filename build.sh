#!/bin/bash

## must be using nightly of rustc to use this 
cargo build --release --out-dir=bin -Z unstable-options