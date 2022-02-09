#!/usr/bin/env bash

set -eux

unset CDPATH

root="$(pwd)"

cargo build --workspace --release

mkdir -p build

pushd build

if [ ! -d ruby ]; then
  git clone git@github.com:ruby/ruby.git ruby
fi

pushd ruby
git checkout -- .
git clean -fd
git checkout v2_6_3
popd

mkdir -p rubies
mkdir -p rubies/2.6.3-st_hash
mkdir -p rubies/2.6.3-strudel

if [ ! -d ruby-strudel-build-root ]; then
  cp -r ruby ruby-strudel-build-root
  pushd ruby-strudel-build-root

  git apply "${root}/strudelify-mri.patch"
  git add .
  git commit -m "strudel hash map backend"

  aclocal
  autoconf
  export cflags="-DSTRUDEL"
  export LDFLAGS="-L${root}/target/release/"
  export LIBS="-lstrudel_st"
  ./configure --prefix="${root}/build/rubies/2.6.3-strudel" --with-baseruby="$(which ruby)"
  unset cflags
  unset LDFLAGS
  unset LIBS

  popd
fi

pushd ruby-strudel-build-root
make miniruby
# make
# make install
popd

if [ ! -d ruby-st_hash-build-root ]; then
  cp -r ruby ruby-st_hash-build-root
  pushd ruby-st_hash-build-root
  autoconf
  ./configure --prefix="${root}/build/rubies/2.6.3-st_hash"
  make
  make install
  popd
fi

