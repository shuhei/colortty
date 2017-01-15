# colortty

colortty converts terminal emulator color schemes. It currently supports the following conversions:

- [mintty](https://github.com/mintty/mintty) -> [alacritty](https://github.com/jwilm/alacritty)
- iTerm 2 -> [alacritty](https://github.com/jwilm/alacritty)

## Usage

```sh
cat some-color.minttyrc | colortty -i mintty
# or
cat some-color.itermcolors | colortty -i iterm2
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

## License

MIT
