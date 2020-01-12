#/!/bin/bash

cargo build --release --target x86_64-pc-windows-gnu \
&& cp lib/x86_64-pc-windows-gnu/dll/*.dll target/x86_64-pc-windows-gnu/release

