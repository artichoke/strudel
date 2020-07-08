#!/usr/bin/env bash

set -euo pipefail

unset CDPATH

root="$(pwd)"
ruby_bin_strudel="${root}/build/rubies/2.6.3-strudel/bin/ruby"
ruby_bin_st_hash="${root}/build/rubies/2.6.3-st_hash/bin/ruby"

echo "strudel"
${ruby_bin_strudel} bench.rb

echo

echo "st_hash"
${ruby_bin_st_hash} bench.rb
