# rwpspread

[![Pipeline](https://github.com/0xk1f0/rwpspread/actions/workflows/build.yml/badge.svg)](https://github.com/0xk1f0/rwpspread/actions/workflows/build.yml)
![AUR](https://img.shields.io/aur/version/rwpspread)

Wallpaper Utility written in Rust

## Features

- Spans an input wallpaper across all monitors
- Works alongside any wlroots based compositor f.E. [Hyprland](https://hyprland.org/)
- Uses [wpaperd](https://github.com/danyspin97/wpaperd) as wallpaper daemon

## Installing

On Archlinux via the [AUR](https://aur.archlinux.org/)

```bash
# stable
paru -S rwpspread
# git
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

# if you want automatic resplits when
# connecting new monitors, start with
# daemon mode -> pairs well with wpaperd
rwpspread -w -d -i /some/path/wallpaper.png

# for all commands
rwpspread --help
```

## Save Locations

All generate files are stored in `/home/$USER/.cache/` with the `rwps_` prefix.

To get all files simply do:

```bash
ls /home/$USER/.cache/rwps_*
```

> Note: If you are using the wpaperd, `rwpspread` will use its default config path `/home/$USER/.config/wpaperd/`.

## Troubleshooting

If you encounter issues after an update or with a new version please do the following:

```bash
# clear cached images
rm /home/$USER/.cache/rwps_*
# clear wpaperd config (if you use it)
rm /home/$USER/.config/wpaperd/wallpaper.toml
```

And try again.

If this doesn't fix your issue, feel free to open a PR and I'll look into it when I find the time.

## Checklist

- [x] splitting for dual screen layout
- [x] splitting for any screen layout (two or more screens)
- [x] Hyprland/wlroots Integration
- [x] wpaperd Integration
- [x] wallpaper caching (don't resplit if we don't need to)
- [x] color palette generation from wallpaper
- [x] watchdog auto-resplit on output change
- [x] center if wallpaper is big enough
- [ ] monitor bezel compensation
- [ ] more alignment options if wallpaper is big enough
