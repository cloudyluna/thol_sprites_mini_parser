# THOL sprites mini parser

An ad-hoc CLI tool to parse a subset of THOL's data, mainly a list of object 
sprites and prints them as a JSON formatted text.

Tested to parse relevant subset of ***9617*** objects (for v20319 as of this writing).
Total referenced sprites from each object: ***60871***.

This tool does not cover transition, animation, sound, etc.

## Usage

- With `cargo`: `cargo run -- <objects directory path>`
- Install and run: `cargo install --path . && tsmp <objects directory path>`


## License

This software is released under the BSD-3-Clause license. See LICENSE file for
further details.