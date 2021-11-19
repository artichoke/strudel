# strudel

[![GitHub Actions](https://github.com/artichoke/strudel/workflows/CI/badge.svg)](https://github.com/artichoke/strudel/actions)
[![Discord](https://img.shields.io/discord/607683947496734760)](https://discord.gg/QCe2tp2)
[![Twitter](https://img.shields.io/twitter/follow/artichokeruby?label=Follow&style=social)](https://twitter.com/artichokeruby)
<br>
[![Crate](https://img.shields.io/crates/v/strudel.svg)](https://crates.io/crates/strudel)
<!-- markdown-link-check-disable-next-line -->
[![API](https://docs.rs/strudel/badge.svg)](https://docs.rs/strudel)
[![API trunk](https://img.shields.io/badge/docs-trunk-blue.svg)](https://artichoke.github.io/strudel/strudel/)

Insertion-ordered hash table suitable for embedding via FFI.

Status: Work in progress.

Strudel was conceived as a drop-in replacement for `st_hash`, a hash map
implemented in C originally written by Peter Moore @ UCB and used in [Ruby]'s
[implementation][st.c] of the [`Hash`][hash] core class.

This crate is an exercise in implementing an insertion-ordered hash map in Rust
that cannot use the built-in `Hasher` infrastructure. The implementation uses
[Ruby's `Hash` backend][sthash-notes] and [Python's dict][pydict-notes] as prior
art.

`StHashMap` is designed to implement the `st_hash` C API and be FFI-friendly.

`StHashMap` is built on top of the high performance [`HashMap`] and [`Vec`] in
Rust `std`.

`StHashMap`, and `StHashSet` which builds on top of it, support in-place updates
of hash keys. No mutable iterators are provided.

The optional `api` and `capi` modules in `strudel` build on top of `StHashMap`
to implement a compatible C API to `st_hash`. This API includes support for
iterating over a mutable map and in-place updates of `(key, value)` pairs. These
features distinguish it from the [`HashMap`] in Rust `std`.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
strudel = "1.0"
```

## Crate features

All features are enabled by default.

- **api** - Enables a Rust API that closely mirrors the C API defined in
  `ruby/st.h`. Disabling this feature drops the [`libc`] dependency.
- **capi** - Enables a C API suitable for embedding `strudel` with FFI. Linking
  in the `libstrudel` cdylib will implement the functions defined in
  `ruby/st.h`. Disabling this feature drops the [`fnv`] dependency.
- **capi-specialized-init** - Enables additional `st_init_table` C APIs with
  known `st_hash_type`s for tables with numeric and string keys.

## Building Ruby with Strudel

Strudel exports most of the symbols implemented by `st.c` in MRI 2.6.3. The
[included patch](strudelify-mri.patch) and some
[`configure` arguments](build.sh) can build the bootstrapping phase of MRI 2.6.3
with Strudel as the hash backend.

To build `miniruby` with Strudel, run:

```sh
./build.sh
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

`strudel` is based on `st.h` and `st.c` from [Ruby]. See [`COPYING`](COPYING).
These sources are vendored in [`ruby`](ruby) source directory.

The `st_hash` implementation in Ruby includes the following notice:

```
/* This is a public domain general purpose hash table package
   originally written by Peter Moore @ UCB.

   The hash table data strutures were redesigned and the package was
   rewritten by Vladimir Makarov <vmakarov@redhat.com>.  */
```

[ruby]: https://github.com/ruby/ruby
[st.c]: https://github.com/ruby/ruby/blob/v2_6_3/st.c
[hash]: https://ruby-doc.org/core-2.6.3/Hash.html
[sthash-notes]: https://github.com/ruby/ruby/blob/v2_6_3/st.c#L1-L101
[pydict-notes]:
  https://github.com/python/cpython/blob/v3.8.4/Objects/dictobject.c#L1-L110
[`hashmap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
[`vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
[`libc`]: https://crates.io/crates/libc
[`fnv`]: https://crates.io/crates/fnv
