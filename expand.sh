#!/bin/bash

rustup run nightly -- cargo rustc --profile=check -- -Zunpretty=expanded 2>&1 >/tmp/expand.rs | tee /tmp/err.rs