# colortty

colortty converts terminal emulator color schemes. It currently supports the following conversions:

- [mintty](https://github.com/mintty/mintty) -> [alacritty](https://github.com/jwilm/alacritty)
- iTerm 2 -> [alacritty](https://github.com/jwilm/alacritty)

## Usage

Convert:

```sh
colortty convert some-color.itermcolors
colortty convert some-color.minttyrc

colortty convert -i iterm some-color-theme
colortty convert -i mintty some-color-theme

cat some-color-theme | colortty convert -i iterm -
cat some-color-theme | colortty convert -i mintty -
```

List color schemes:

```sh
colortty list
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
