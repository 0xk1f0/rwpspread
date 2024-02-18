# rwpspread

[![Pipeline](https://github.com/0xk1f0/rwpspread/actions/workflows/build.yml/badge.svg)](https://github.com/0xk1f0/rwpspread/actions/workflows/build.yml)
[![AUR](https://img.shields.io/aur/version/rwpspread?label=AUR%20rwpspread)](https://aur.archlinux.org/packages/rwpspread) 
[![AUR](https://img.shields.io/aur/version/rwpspread-git?label=AUR%20rwpspread-git)](https://aur.archlinux.org/packages/rwpspread-git)

Wallpaper Utility written in Rust

## Features

- Spans an input wallpaper across all monitors
- Works alongside any [`wlroots`](https://gitlab.freedesktop.org/wlroots/wlroots) based compositor f.E. [`Hyprland`](https://hyprland.org/)
- Color-Palette generation
- [`wpaperd`](https://github.com/danyspin97/wpaperd) wallpaper backend integration
- [`swaybg`](https://github.com/swaywm/swaybg) wallpaper backend integration
- [`swaylock`](https://github.com/swaywm/swaylock) integration

## Installing

[![Arch Linux](https://img.shields.io/badge/Arch_Linux-via_AUR-grey?style=for-the-badge&logo=arch-linux&logoColor=white&labelColor=1793D1)](https://aur.archlinux.org/packages/rwpspread)

```bash
# stable
paru -S rwpspread
# git
paru -S rwpspread-git
```

[![NixOS](https://img.shields.io/badge/NixOS-via_nixpkgs-grey?style=for-the-badge&logo=nixos&logoColor=white&labelColor=5277C3)](https://search.nixos.org/packages?channel=unstable&show=rwpspread&type=packages&query=rwpspread)

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
  -i, --image <IMAGE>      Image file path
  -a, --align <ALIGN>      Do not downscale the base image, align the layout instead [possible values: tl, tr, tc, bl, br, bc, rc, lc, c]
  -b, --backend <BACKEND>  Wallpaper setter backend [possible values: wpaperd, swaybg]
  -d, --daemon             Enable daemon mode, will watch and resplit on output changes
  -p, --palette            Generate a color palette from input image
  -s, --swaylock           Use swaylock integration
  -f, --force-resplit      Force resplit, skips all image cache checks
  -h, --help               Print help
  -V, --version            Print version
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

# to use f.E. the wpaperd integration
# this autogenerates the config file
# you will need to have wpaperd installed
rwpspread -b wpaperd -i /some/path/wallpaper.png

# if you want automatic resplits when
# connecting new monitors
# start with daemon mode
rwpspread -di /some/path/wallpaper.png
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

If used just to split images, output images are saved to the current working directory.

```bash
# output files in $PWD
rwpspread -i /some/path/wallpaper.png
```

When used with a backend, output images are stored in `/home/$USER/.cache/rwpspread/` with the `rwps_` prefix.

```bash
# output files in /home/$USER/.cache/rwpspread/
rwpspread -b swaybg -i /some/path/wallpaper.png
```

To get all files simply do:

```bash
ls /home/$USER/.cache/rwpspread/
```
> [!NOTE]
> If you are using the `wpaperd` backend, `rwpspread` will use its default config path `/home/$USER/.config/wpaperd/` for the auto-generated configuration.

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
- [x] `swaybg` integration
- [ ] standalone support
- [ ] monitor bezel compensation

## Credits and Thanks

- [nu-nu-ko](https://github.com/nu-nu-ko) - Nix Package Maintainer
- [smithay-client-toolkit](https://github.com/Smithay/client-toolkit) - Rust Interaction with Wayland
- [wpaperd](https://github.com/danyspin97/wpaperd) - Excellent Wallpaper Daemon
- [swaylock](https://github.com/swaywm/swaylock) - Screen Locking Utility
- [swaybg](https://github.com/swaywm/swaybg) - Wallpaper Utility
