# ufo-gleaner

![PyPI - Version](https://img.shields.io/pypi/v/ufo-gleaner)
![PyPI - Python Version](https://img.shields.io/pypi/pyversions/ufo-gleaner)

High-performance UFO/GLIF parser for Python written in Rust

## Usage

### Install

Install with pip:

```console
$ python -m pip install ufo-gleaner
```

### Quickstart

`UfoGleaner` can be used with any filesystem provider that implements the `FileProvider` interface:

```python
from ufo_gleaner import UfoGleaner, FileProvider

provider = FileProvider("/path/to/myfont.ufo")
gleaner = UfoGleaner(provider)

gleaner.glean()
```