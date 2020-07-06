# st_hash

[![GitHub Actions](https://github.com/artichoke/st_hash/workflows/CI/badge.svg)](https://github.com/artichoke/st_hash/actions)
[![Discord](https://img.shields.io/discord/607683947496734760)](https://discord.gg/QCe2tp2)
[![Twitter](https://img.shields.io/twitter/follow/artichokeruby?label=Follow&style=social)](https://twitter.com/artichokeruby)
<br>
[![Crate](https://img.shields.io/crates/v/st_hash.svg)](https://crates.io/crates/st_hash)
[![API](https://docs.rs/st_hash/badge.svg)](https://docs.rs/st_hash)
[![API trunk](https://img.shields.io/badge/docs-trunk-blue.svg)](https://artichoke.github.io/st_hash/st_hash/)

Drop-in replacement for `st_hash` originally written by Peter Moore @ UCB and
used in [Ruby](https://github.com/ruby/ruby)'s implementation of the
[`Hash`][hash] core class.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
st_hash = "1.0"
```

## Crate features

All features are enabled by default.

- **capi** - Enables a C API suitable for embedding `st_hash` with FFI.
  Disabling this drops the [`libc`] dependency.

## License

`st_hash` is licensed under the [MIT License](LICENSE) (c) Ryan Lopopolo.

`st_hash` is based on `st.h` and `st.c` from
[Ruby](https://github.com/ruby/ruby). See [`COPYING`](COPYING).

The `st_hash` implementation in Ruby includes the following notice:

```
/* This is a public domain general purpose hash table package
   originally written by Peter Moore @ UCB.

   The hash table data strutures were redesigned and the package was
   rewritten by Vladimir Makarov <vmakarov@redhat.com>.  */
```

[hash]: https://ruby-doc.org/core-2.6.3/Hash.html
[`libc`]: https://crates.io/crates/libc
