#!/usr/bin/env bash

set -euxo pipefail

unset CDPATH

root="$(pwd)"
ruby_bin="${root}/build/rubies/2.6.3-strudel/bin/ruby"

if [ ! -f "${ruby_bin}" ]; then
  echo 1>&2 "${ruby_bin} not built!"
  echo 1>&2 "Invoke build.sh before running tests."
  exit 1
fi

pushd build

if [ ! -d mspec ]; then
  git clone git@github.com:ruby/mspec.git mspec
fi

if [ ! -d spec ]; then
  git clone git@github.com:ruby/spec.git spec
fi

if [ ! -d rails ]; then
  git clone git@github.com:rails/rails.git rails
fi

./mspec/bin/mspec -t "${ruby_bin}" spec/core/hash/
./mspec/bin/mspec -t "${ruby_bin}" spec/core/
