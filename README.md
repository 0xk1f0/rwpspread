<div align="center" style="text-decoration: none;">
  <h1>rwpspread</h1>
  <a href="https://github.com/0xk1f0/rwpspread/releases/latest">
    <img src="https://img.shields.io/github/v/release/0xk1f0/rwpspread?style=for-the-badge&color=blue" />
  </a>
  <a href="https://crates.io/crates/rwpspread">
    <img src="https://img.shields.io/crates/v/rwpspread?style=for-the-badge&color=orange" />
  </a>
  <a href="https://aur.archlinux.org/packages/rwpspread">
    <img src="https://img.shields.io/aur/version/rwpspread?style=for-the-badge&color=1793D1" />
  </a>
  <br><br>
  <a href="https://wallhaven.cc/w/l8q5k2">
    <img width=70% height=75% src="https://github.com/user-attachments/assets/836131fe-0f8e-449e-993d-9df2ebd33865"></img>
  </a>
</div>

## Features

- Wallpaper spanning across all monitors
- Monitor hotplugging detection
- Color palette generation
- Monitor bezel compensation
- Support for various wallpaper setters
  - [`wpaperd`](https://github.com/danyspin97/wpaperd)
  - [`swaybg`](https://github.com/swaywm/swaybg)
  - [`hyprpaper`](https://github.com/hyprwm/hyprpaper)
- Configuration generation for lockers
  - [`swaylock`](https://github.com/swaywm/swaylock)
  - [`hyprlock`](https://github.com/hyprwm/hyprlock)

## Installing

[![Arch Linux](https://img.shields.io/badge/Arch_Linux-via_AUR-grey?style=for-the-badge&logo=arch-linux&color=1793D1)](https://aur.archlinux.org/packages/rwpspread)

```bash
# stable
paru -S rwpspread
# git
paru -S rwpspread-git
```

[![Nix](https://img.shields.io/badge/nix-via%20nixpkgs-grey?style=for-the-badge&logo=Nixos&color=5277C3)](https://search.nixos.org/packages?query=rwpspread)

```bash
# try it out
nix run nixpkgs#rwpspread
# or add to any user/system package list as
pkgs.rwpspread
# master
take `rwpspread` from this repos flake directly.
```

[![Crates.io](https://img.shields.io/badge/crates.io-via_cargo-grey?style=for-the-badge&logo=rust&color=FFC933)](https://crates.io/crates/rwpspread)

```bash
# globally
cargo install rwpspread
```

## Building

```bash
git clone https://github.com/0xk1f0/rwpspread.git
cd rwpspread/
cargo build --release
```

## Usage

```text
rwpspread 0.5.0 - Multi-Monitor Wallpaper Spanning Utility

Usage:
  rwpspread [OPTIONS] <--image <IMAGE>|--info>

Options:
  -i, --image <IMAGE>           Image file or directory path
      --info                    Show detectable information
  -o, --output <OUTPUT>         Output directory path
  -a, --align <ALIGN>           Do not downscale the base image, align the layout instead [possible values: tl, tr, tc, bl, br, bc, rc, lc, ct]
  -b, --backend <BACKEND>       Wallpaper setter backend [possible values: wpaperd, swaybg, hyprpaper]
  -l, --locker <LOCKER>         Lockscreen implementation to generate for [possible values: swaylock, hyprlock]
      --bezel <BEZEL>           Bezel amount in pixels to compensate for
  -m, --monitors <MONITORS>...  List of monitor containing their diagonal in inches [format: "<NAME>:<INCHES>"]
      --experimental-ppi        Compensate for different monitor ppi values
  -d, --daemon                  Enable daemon mode and resplit on output changes
  -p, --palette                 Generate a color palette from input image
      --pre <PRE>               Script to execute before splitting
      --post <POST>             Script to execute after splitting
  -w, --watch                   Watch for wallpaper source changes and resplit on changes
  -f, --force-resplit           Force resplit, skips all image cache checks
  -h, --help                    Print help
  -V, --version                 Print version
```

## Examples

```bash
# it takes an input image
# screens are automatically read
# if running a wlroots based compositor
rwpspread -i /some/path/wallpaper.png

# you can also specify a directory
# as input and rwpspread will choose
# an image from it randomly
# supported formats: jpg, jpeg, png
rwpspread -i /some/wallpaper/dir/

# to align the layout if the input image
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

> [!NOTE]  
> `rwpspread` will try to force close any backend instances already running, this may fail in some cases and prevent it from setting any wallpapers at all. See Issue https://github.com/0xk1f0/rwpspread/issues/100
> 
> Make sure `rwpspread` is the first to start any `swaybg`, `hyprpaper` or `wpaperd` process, although the two latter ones may not be affected.

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

## `hyprlock` Integration

Just include `/home/$USER/.cache/rwpspread/rwps_hyprlock.conf` in your normal `hyprlock.conf` like this:

```text
# include generate rwpspread
source=/home/$USER/.cache/rwpspread/rwps_hyprlock.conf
```

This allows you to configure additional `hyprlock` stuff after the import statement.

## Custom Scripts

You can specify custom scripts or programs to execute before and after splitting takes place.

```bash
# before splitting
rwpspread --pre /some/pre/script.sh -di /some/path/wallpaper.png
# after splitting
rwpspread --post /some/post/script.sh -di /some/path/wallpaper.png
# or both
rwpspread --pre /some/pre/script.sh --post /some/post/script.sh -di /some/path/wallpaper.png
```

When in `daemon` mode, these script will also execute on re-splits f.E. monitor hotplugs.

> [!NOTE]  
> `rwpspread` will wait for these scripts to finish executing before continuing its own execution
> 
> So make sure you don't supply scripts that block execution indefinitely

## Save Locations

If used just to split images, output images are saved to the current working directory.

```bash
# output files in $PWD
rwpspread -i /some/path/wallpaper.png
```

When used with the backend or daemon option, output images are stored in `/home/$USER/.cache/rwpspread/` with the `rwps_` prefix.

```bash
# output files in /home/$USER/.cache/rwpspread/
rwpspread -b swaybg -i /some/path/wallpaper.png
# output files in /home/$USER/.cache/rwpspread/
rwpspread -di /some/path/wallpaper.png
```

To get all files simply do:

```bash
ls /home/$USER/.cache/rwpspread/
```
> [!NOTE]
> If you are using the `wpaperd` backend, `rwpspread` will use its default config path `/home/$USER/.config/wpaperd/` for the auto-generated configuration.

If you want to customize the output folder. use the `-o` option:

```bash
# output files in /some/other/dir/
rwpspread -o /some/other/dir/ -i /some/path/wallpaper.png
```

> [!NOTE]
> Be aware that `rwpspread` will take full control of this folder and potentially delete files you may not want to be deleted!

### Pretty filenames

In general the split files that `rwpspread` stores are not constant, they changed based on the configuration it receives. This includes what type of options it was run with and how many monitors are currently attached. Files are formatted in a specific way.

```bash
# actual output file
rwps_<monitor-name>_<config-hash>.png
```

This can make these file a bit cumbersome to use in external tools or wallpaper setters. This is why rwpspread also creates additional symlinks that have a predictable name, which point to the output file. It is important to note that it will do this only if one of either `-b` or `-d` are specified.

```bash
# symlink to actual file
rwps_<monitor-name>.png
```

You can use this in any other tool that uses the output files of `rwpspread` without worrying about changing names.

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

## Credits and Thanks

- [fsnkty](https://github.com/fsnkty) - Nix Package Maintainer
- [smithay-client-toolkit](https://github.com/Smithay/client-toolkit) - Rust Interaction with Wayland
- [wpaperd](https://github.com/danyspin97/wpaperd) - Excellent Wallpaper Daemon
- [swaylock](https://github.com/swaywm/swaylock) - Screen Locking Utility
- [swaybg](https://github.com/swaywm/swaybg) - Wallpaper Utility
- [hyprpaper](https://github.com/hyprwm/hyprpaper) - Hypr Wallpaper Daemon
- [hyprlock](https://github.com/hyprwm/hyprlock) - Hypr Screen Locker
- [material-colors](https://github.com/Aiving/material-colors) - Material Color Generation
