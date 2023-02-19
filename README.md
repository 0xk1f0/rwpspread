# rwpspread

WIP Wallpaper Multi-Monitor Utility written in Rust

Code quality is probably pretty poor as I'm new to Rust

## Building

```bash
git clone https://github.com/0xk1f0/rwpspread.git
cd rwpspread/
cargo build --release
```

## Usage

```bash
# it takes an input image,
# resolution of the primary and secondary monitor 
# and an offset (in pixels)
rwpspread <image> <primary> <secondary> <offset>
# for example
rwpspread wallpaper.png 2560x1440 1920x1080 100
```

## TODO

- wallpaper rearranging
- GUI?
