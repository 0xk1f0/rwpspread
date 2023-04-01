# rwpspread

Wallpaper Utility written in Rust

## Features

- Spans an input wallpaper across all monitors
- Works alongside any wlroots based compositor f.E. [Hyprland](https://hyprland.org/)
- Uses [wpaperd](https://github.com/danyspin97/wpaperd) as wallpaper daemon
- Code quality is probably pretty poor as I'm new to Rust

## Installing

On Archlinux via the [AUR](https://aur.archlinux.org/)

```bash
paru -S rwpspread-git
```

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
# if running a wlroots based compositor
rwpspread -i <image>

# for example
rwpspread -i /some/path/wallpaper.png

# to use the wpaperd integration
# this autogenerates the config file
# you will need to have wpaperd installed
rwpspread -w -i /some/path/wallpaper.png

# for more info
rwpspread --help
```

## Checklist

- [x] splitting for dual screen layout
- [x] splitting for any screen layout (two or more screens)
- [x] Hyprland/wlroots Integration
- [x] wpaperd Integration
- [x] wallpaper caching (don't resplit if we don't need to)
- [ ] watchdog auto-resplit on output change
- [ ] restore standalone support
- [ ] alignment adjust if wallpaper is big enough
- [ ] monitor bezel compensation
