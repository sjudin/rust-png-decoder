# A (very) basic PNG decoder written in Rust
## Introduction
Implementation of a decoder that decodes .png images following the [Portable Network Graphics (PNG) Specification (Second Edition)](www.abs.com)

Mainly meant as a fun project to learn some Rust

## Getting started
### Requirements
* Rust (cargo and rustc)
* Python >3.8
### Installing
Clone repository, `cd` into the repo directory and run
```
./install.sh
```

## Building
Source the virtual python environment created by the installation script (if you
have not done so)
```
source venv/bin/activate
```
Then build
```
./build.sh
```

## Usage
Decode and print a png image to your terminal (requires truecolor support)
```
target/debug/png_reader <path/to/a/png>
```
You might have to reduce the font size of your terminal quite a lot for larger images

### Using Python bindings
Source the environment created by `install.sh` and run the python plotting script
which plots the decoded image together with the same image read using matplotlib imread
```
source venv/bin/activate
python plotter.py <path/to/a/png>
```

## Limitations
Does not support images that use [interlacing](https://www.w3.org/TR/2003/REC-PNG-20031110/#8Interlace)

Supported formats are:
* Indexed color
* Truecolor
* Truecolor with alpha (alpha channel will be ignored)
* Grayscale

if you try to decode an image which is not of this format an error will be raised. You can convert any .png to
index-colored format using [magick](https://imagemagick.org/index.php)
```
magick <path/to/input/png> -type palette -colors 255 <path/to/output/png>
```

