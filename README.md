# colortty

colortty converts terminal emulator color schemes. It currently supports the following conversions:

- [mintty](https://github.com/mintty/mintty) -> [alacritty](https://github.com/jwilm/alacritty)

## Usage

```sh
cat some-color.minttyrc | colortty
```

## Development

Build:

```sh
cargo build
```

Test:

```sh
cargo test
```
