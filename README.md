# colortty

colortty converts terminal emulator color schemes. It currently supports the following conversions:

- [mintty](https://github.com/mintty/mintty) -> [alacritty](https://github.com/jwilm/alacritty)
- iTerm 2 -> [alacritty](https://github.com/jwilm/alacritty)

## Usage

```sh
colortty some-color.itermcolors
colortty some-color.minttyrc

colortty -i iterm some-color-theme
colortty -i mintty some-color-theme

cat some-color-theme | colortty -i iterm -
cat some-color-theme | colortty -i mintty -
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
