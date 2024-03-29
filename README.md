# strudel

[![GitHub Actions](https://github.com/artichoke/strudel/workflows/CI/badge.svg)](https://github.com/artichoke/strudel/actions)
[![Discord](https://img.shields.io/discord/607683947496734760)](https://discord.gg/QCe2tp2)
[![Twitter](https://img.shields.io/twitter/follow/artichokeruby?label=Follow&style=social)](https://twitter.com/artichokeruby)
<br>
[![Crate](https://img.shields.io/crates/v/strudel.svg)](https://crates.io/crates/strudel)
[![API](https://docs.rs/strudel/badge.svg)](https://docs.rs/strudel)
[![API trunk](https://img.shields.io/badge/docs-trunk-blue.svg)](https://artichoke.github.io/strudel/strudel/)

Insertion-ordered hash table suitable for embedding via FFI.

Status: Work in progress.

Strudel was conceived as a drop-in replacement for `st_hash`, a hash map
implemented in C originally written by Peter Moore @ UCB and used in [Ruby]'s
[implementation][`st.c`] of the [`Hash`][hash] core class.

[ruby]: https://github.com/ruby/ruby
[hash]: https://ruby-doc.org/core-2.6.3/Hash.html

This crate is an exercise in implementing an insertion-ordered hash map in Rust
that cannot use the built-in `Hasher` infrastructure. The implementation uses
[Ruby's `Hash` backend][sthash-notes] and [Python's dict][pydict-notes] as prior
art.

[sthash-notes]: https://github.com/ruby/ruby/blob/v2_6_3/st.c#L1-L101
[pydict-notes]:
  https://github.com/python/cpython/blob/v3.8.4/Objects/dictobject.c#L1-L110

This crate exports two types:

- `StHashMap` is a hash map built on top of the high performance [`HashMap`] and
  [`Vec`] in Rust `std`. It is designed to implement the `st_hash` C API and be
  FFI-friendly. This map supports in-place updates of hash keys. No mutable
  iterators are provided.
- `StHashSet` is a set that wraps an `StHashMap` like `HashSet` does in `std`.

[`hashmap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
[`vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
[`hashse`]: https://doc.rust-lang.org/std/collections/struct.HashSet.html

The `api` and `capi` modules in `strudel` build on top of `StHashMap` to
implement a compatible C API to `st_hash`. This API includes support for
iterating over a mutable map and in-place updates of `(key, value)` pairs. These
features distinguish it from the [`HashMap`] in Rust `std`.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
strudel = "1.0"
```

## Building Ruby with Strudel

Strudel exports most of the symbols implemented by `st.c` in MRI 2.6.3. The
[included patch] and some [`configure` arguments] can build the bootstrapping
phase of MRI 2.6.3 with Strudel as the hash backend.

[included patch]: strudelify-mri.patch
[`configure` arguments]: build.sh

To build `miniruby` with Strudel, run:

```sh
./build.sh
```

`build.sh` requires autoconf 2.69. On macOS with Homebrew, this can be done
with:

```sh
brew install autoconf@2.69
PATH="/usr/local/opt/autoconf@2.69/bin:$PATH" ./build.sh
```

The resulting Ruby is in `./build/ruby-strudel-build-root/miniruby`. `miniruby`
can run simple scripts involving `Hash`, for example:

```console
$ ./build/ruby-strudel-build-root/miniruby -e 'h = {}' -e '1000.times { |i| h[i] = i }' -e 'puts h.length'
1000
```

`miniruby` successfully executes the benchmarks in [`benches`](benches).

NOTE: Strudel cannot build a full Ruby due to bugs in the implementation of the
`st_hash` API.

## License

`strudel` is licensed under the [MIT License](LICENSE) (c) Ryan Lopopolo.

This repository includes a vendored copy of [`st.h`] and [`st.c`] from Ruby
2.6.3, which is licensed under the [Ruby license] or [BSD 2-clause license]. See
[`vendor/README.md`] for more details. These sources are not distributed on
[crates.io].

[`st.h`]: vendor/ruby-2.6.3/st.h
[`st.c`]: vendor/ruby-2.6.3/st.c
[ruby license]: vendor/ruby-2.6.3/COPYING
[bsd 2-clause license]: vendor/ruby-2.6.3/BSDL
[`vendor/readme.md`]: vendor/README.md
[crates.io]: https://crates.io/

The `st_hash` implementation in Ruby includes the following notice:

```
/* This is a public domain general purpose hash table package
   originally written by Peter Moore @ UCB.

   The hash table data strutures were redesigned and the package was
   rewritten by Vladimir Makarov <vmakarov@redhat.com>.  */
```
