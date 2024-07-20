#!/bin/bash

cargo makedocs -e log -e thiserror
cargo doc -p foa --no-deps --all-features
