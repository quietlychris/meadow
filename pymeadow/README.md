# PyMeadow

An *incomplete* Python binding for the Meadow library. Most Python types can be published, but operates on `String` types under the hood, which will need to parsed after `request()` methods both on the Rust and Python sides.

```python
import pymeadow as meadow
import time
import unittest
    
# Create a Meadow Host
host = meadow.Host()

# Create Node that operates over Tcp with all the default settings
# The topic is `gps`
node = meadow.TcpNode("gps")

# Publish a value, which can be any Python type
node.publish(6)
time.sleep(1)
# Request methods always return a String, so we'll need to do that conversion explicitly
result = node.request()
```

## Development

To get started with `maturin`-based development, follow the documentation [here](https://www.maturin.rs/installation.html)

```sh
$ cd pymeadow
# Create a virtual environment if it doesn't already exist
$ python3 -m venv .venv
$ source .venv/bin/activate
$ pip3 install maturin
$ maturin develop
$ python3 test.py
```

## License

This library is licensed under the Mozilla Public License, version 2.0 (MPL-2.0)