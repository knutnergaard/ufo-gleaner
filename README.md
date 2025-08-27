# ufo-gleaner

[![PyPI - Version](https://img.shields.io/pypi/v/ufo-gleaner?logo=pypi&logoColor=white&style=for-the-badge)](https://pypi.org/project/ufo-gleaner/)
[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/ufo-gleaner?logo=python&logoColor=white&style=for-the-badge)](https://www.python.org)



High-performance UFO/GLIF parser for Python written in Rust

## Usage

### Install

Install with pip:

```bash
python -m pip install ufo-gleaner
```

### Quickstart

Parse all `.glif` files in one go as a dictionary mapping glyph names to their attributes:

```python
from ufo_gleaner import UfoGleaner, FileProvider

provider = FileProvider("/path/to/myfont.ufo")
gleaner = UfoGleaner(provider)

glyphs = gleaner.glean()
print(glyphs["A"]["width"])
```

### Custom Providers

`UfoGleaner` can be used with any Python object that implements a `read(path: str) -> bytes` method,
where `path` is relative to the UFO root. This lets you from both `.ufo` directories and `.ufoz` 
ZIP archives, for example:

```python
import zipfile
from ufo_gleaner import UfoGleaner

class UfozProvider:
    def __init__(self, root):
        self.zipfile = zipfile.ZipFile(root, "r")
    
    def read(self, path: str) -> bytes:
        return self.zipfile.read(path)

provider = UfozProvider("/path/to/myfont.ufoz")
gleaner = UfoGleaner(provider)

glyphs = gleaner.glean()
```