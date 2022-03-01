# yabel

> **y**et **a**nother **be**ncode **l**ibrary

A really simple bencode library.

## Usage

Add this to your `Cargo.toml` dependencies:

```toml
[dependencies]
yabel = { git = "https://github.com/rctlmk/yabel" }
```

You can find some examples [here](examples/README.MD).

Though I recommend giving one of [these crates](https://crates.io/keywords/bencode) a go instead.

## Notes

This implementation will force sort all dictionaries (see [this example](examples/resume-dat.rs) for more details).

## License

Distributed under the terms of the MIT license, see [LICENSE](LICENSE) for details.