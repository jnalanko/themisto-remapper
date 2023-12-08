# Themisto remapper

Compresses the color range of Themisto output by removing all colors with less than a given number of total hits.

# Compiling

Install Rust. Then: `cargo build --release`. This generates the binary to `./target/release/themisto-remapper`.

# Usage

```
Usage: themisto-remapper --input <input> --output <output> --mapping-file <mapping-file> --min-hits <min-hits>

Options:
  -i, --input <input>                Themisto pseudoalignment file
  -o, --output <output>              Output pseudoalignment file
  -m, --mapping-file <mapping-file>  Mapping output file with pairs of new and old colors, one per line, separated by a tab.
  -n, --min-hits <min-hits>          Minimum number of hits for a color to be included in the output
  -h, --help                         Print help
```
