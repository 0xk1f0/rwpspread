# rwpspread

[![Pipeline](https://github.com/0xk1f0/rwpspread/actions/workflows/build.yml/badge.svg)](https://github.com/0xk1f0/rwpspread/actions/workflows/build.yml)
![AUR](https://img.shields.io/aur/version/rwpspread?label=AUR%20rwpspread)
![AUR](https://img.shields.io/aur/version/rwpspread-git?label=AUR%20rwpspread-git)

Wallpaper Utility written in Rust

## Features

- Spans an input wallpaper across all monitors
- Works alongside any [`wlroots`](https://gitlab.freedesktop.org/wlroots/wlroots) based compositor f.E. [`Hyprland`](https://hyprland.org/)
- Color-Palette generation
- [`wpaperd`](https://github.com/danyspin97/wpaperd) wallpaper daemon integration
- [`swaylock`](https://github.com/swaywm/swaylock) integration

## Installing

![Arch Linux](https://img.shields.io/badge/Arch_Linux-via_AUR-grey?style=for-the-badge&logo=arch-linux&logoColor=white&labelColor=1793D1)

```bash
# stable
paru -S rwpspread
# git
paru -S rwpspread-git
```

![NixOS](https://img.shields.io/badge/NixOS-via_nixpkgs-grey?style=for-the-badge&logo=nixos&logoColor=white&labelColor=5277C3)

```bash
# On NixOS
nix-env -iA nixos.rwpspread
# On Non NixOS
nix-env -iA nixpkgs.rwpspread
```

## Building

```bash
git clone https://github.com/0xk1f0/rwpspread.git
cd rwpspread/
cargo build --release
```

## Usage

```bash
❯ rwpspread --help
Multi-Monitor Wallpaper Utility

Usage: rwpspread [OPTIONS] --image <IMAGE>

Options:
  -i, --image <IMAGE>  Image File Path
  -w, --wpaperd        Use wpaperd Integration
  -s, --swaylock       Generate swaylock file
  -p, --palette        Generate a color palette from Wallpaper
  -d, --daemon         Enable Daemon Watchdog mode, will resplit on Output changes
      --force-resplit  Force Resplit, skips all Image Cache checks
  -a, --align <ALIGN>  Do not downscale the Base Image, align the Layout instead [possible values: tl, tr, tc, bl, br, bc, rc, lc, c]
  -h, --help           Print help
  -V, --version        Print version
```

## Examples

```bash
# it takes an input image
# screens are automatically read
# if running a wlroots based compositor
rwpspread -i /some/path/wallpaper.png

# to align the layout if the input images
# is big enough, instead of resizing
# for example, to align it top-right
rwpspread -a tr -i /some/path/wallpaper.png

# to use the wpaperd integration
# this autogenerates the config file
# you will need to have wpaperd installed
rwpspread -wi /some/path/wallpaper.png

# if you want automatic resplits when
# connecting new monitors, start with
# daemon mode -> requires wpaperd
rwpspread -wdi /some/path/wallpaper.png
```

## `swaylock` Integration

A drop-in string for swaylock will be put in `/home/$USER/.cache/rwpspread/rwps_swaylock.conf` which can look something like:

```text
-i <image_path_1> -i <image_path_2>
```

This file can be sourced and used with your swaylock command, for exmaple:

```bash
#!/usr/bin/env bash

# source the command options
IMAGES=$(cat /home/$USER/.cache/rwpspread/rwps_swaylock.conf)
# execute with them
swaylock $IMAGES --scaling fill
```

## Save Locations

All generate files are stored in `/home/$USER/.cache/rwpspread/` with the `rwps_` prefix.

To get all files simply do:

```bash
ls /home/$USER/.cache/rwpspread/
```

> Note: If you are using the wpaperd, `rwpspread` will use its default config path `/home/$USER/.config/wpaperd/`.

## Troubleshooting

If you encounter issues after an update or with a new version please do the following:

```bash
# clear cached images
rm -r /home/$USER/.cache/rwpspread/
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
- [x] `swaylock` integration
- [x] parallel image processing
- [x] more alignment options if wallpaper is big enough
- [x] palette generation rework (broken in some cases)
- [ ] `swaybg` integration
- [ ] standalone support
- [ ] monitor bezel compensation

## Credits and Thanks

- [smithay-client-toolkit](https://github.com/Smithay/client-toolkit) - Rust Interaction with Wayland
- [wpaperd](https://github.com/danyspin97/wpaperd) - Excellent Wallpaper Daemon
- [nu-nu-ko](https://github.com/nu-nu-ko) - Nix Package Maintainer
