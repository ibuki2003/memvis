# MemVis: visualize memory space

Container format files such as `ELF` or `uf2` has complex structure so we cannot comprehend memory layout by reading hexdump.
MemVis visualizes memory space with colored symbols/segments so we can know how are data placed.

## help
```
Usage: memvis [OPTIONS] <FILENAME>

Arguments:
  <FILENAME>

Options:
  -c <COLS>              Bytes per line [default: 16]
  -b, --break-on-bounds  Break on section boundaries
  -e, --hide-empty       Hide 0-byte sections
  -d, --demangle         Demangle symbols
  -h, --help             Print help
  -V, --version          Print version
```

## example

![example on ELF file](/image1.png)
