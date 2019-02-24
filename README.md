# colortty

colortty is a utility to generate color schemes for alacritty. It also supports the following conversions:

- iTerm 2 -> [alacritty](https://github.com/jwilm/alacritty)
- [mintty](https://github.com/mintty/mintty) -> [alacritty](https://github.com/jwilm/alacritty)

## Usage

List color schemes at [mbadolato/iTerm2-Color-Schemes](https://github.com/mbadolato/iTerm2-Color-Schemes):

```sh
colortty list
```

Get color scheme from [mbadolato/iTerm2-Color-Schemes](https://github.com/mbadolato/iTerm2-Color-Schemes) in Alacritty format:

```sh
colortty get <color scheme name>
```

Convert:

```sh
# Convert with implicit input type
colortty convert some-color.itermcolors
colortty convert some-color.minttyrc

# Convert with explicit input type
colortty convert -i iterm some-color-theme
colortty convert -i mintty some-color-theme

# Convert stdin (explicit input type is necessary)
cat some-color-theme | colortty convert -i iterm -
cat some-color-theme | colortty convert -i mintty -"
```

## Development

Install:

```sh
cargo install --path .
```

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
