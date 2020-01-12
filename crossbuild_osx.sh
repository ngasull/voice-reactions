#!/bin/bash

if [ ! -e "$(pwd)/osxcross" ] ; then
  git clone https://github.com/tpoechtrager/osxcross
  cd osxcross
  wget https://s3.dockerproject.org/darwin/v2/MacOSX10.11.sdk.tar.xz
  mv MacOSX10.11.sdk.tar.xz tarballs/
  sed -i -e 's|-march=native||g' build_clang.sh wrapper/build_wrapper.sh
  UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh
fi

PATH="$(pwd)/osxcross/target/bin:$PATH" \
cargo build --release --target x86_64-apple-darwin
