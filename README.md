# strudel

[![GitHub Actions](https://github.com/artichoke/strudel/workflows/CI/badge.svg)](https://github.com/artichoke/strudel/actions)
[![Discord](https://img.shields.io/discord/607683947496734760)](https://discord.gg/QCe2tp2)
[![Twitter](https://img.shields.io/twitter/follow/artichokeruby?label=Follow&style=social)](https://twitter.com/artichokeruby)
<br>
[![Crate](https://img.shields.io/crates/v/strudel.svg)](https://crates.io/crates/strudel)
[![API](https://docs.rs/strudel/badge.svg)](https://docs.rs/strudel)
[![API trunk](https://img.shields.io/badge/docs-trunk-blue.svg)](https://artichoke.github.io/strudel/strudel/)

Drop-in replacement for `st_hash` originally written by Peter Moore @ UCB and
used in [Ruby](https://github.com/ruby/ruby)'s [implementation][st.c] of the
[`Hash`][hash] core class.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
strudel = "1.0"
```

## Crate features

All features are enabled by default.

- **capi** - Enables a C API suitable for embedding `strudel` with FFI. Linking
  in the `libstrudel` cdylib will implement the functions defined in [`include/st.h`](include/st.h).
  Disabling this drops the [`libc`] dependency.

## License

`strudel` is licensed under the [MIT License](LICENSE) (c) Ryan Lopopolo.

`strudel` is based on `st.h` and `st.c` from
[Ruby](https://github.com/ruby/ruby). See [`COPYING`](COPYING). These sources are
vendored in [`ruby`](ruby)

The `strudel` implementation in Ruby includes the following notice:

```
/* This is a public domain general purpose hash table package
   originally written by Peter Moore @ UCB.

   The hash table data strutures were redesigned and the package was
   rewritten by Vladimir Makarov <vmakarov@redhat.com>.  */
```

[st.c]: https://github.com/ruby/ruby/blob/v2_6_3/st.c
[hash]: https://ruby-doc.org/core-2.6.3/Hash.html
[`libc`]: https://crates.io/crates/libc
