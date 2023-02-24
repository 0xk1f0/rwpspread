# rwpspread

WIP Wallpaper Utility written in Rust

## Features

- Spans an input wallpaper across all monitors
- Intended to work alongside [Hyprland](https://hyprland.org/)
- Code quality is probably pretty poor as I'm new to Rust

## Building

```bash
git clone https://github.com/0xk1f0/rwpspread.git
cd rwpspread/
cargo build --release
```

## Usage

```bash
# it takes an input image
# screens are automatically read
# if running a Hyprland session
rwpspread -i <image>
# for example
rwpspread -i /some/path/wallpaper.png
```

## Checklist

- [x] splitting for dual screen layout
- [x] splitting for any screen layout (two or more screens)
- [x] Hyprland Integration
- [ ] restore standalone support
- [ ] wpaperd Integration
