# Squirrel

sq (squirrel) cli app for more better command

## Installing

```sh
sudo rm -rf /usr/local/bin/sq && \
sudo curl -L https://github.com/andypangaribuan/squirrel/releases/latest/download/sq-`uname -s`-`uname -m` -o /usr/local/bin/sq && \
sudo chmod +x /usr/local/bin/sq
```

## Usage

```sh
sq --help
```

## Third party package

This sq cli using 3rd party package, you can install using brew:

- watch (brew install watch)
- expect (brew install expect), this for unbuffer cli
- kubectl (brew install kubernetes-cli)

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/andypangaribuan/squirrel/tags).

## Contributions

Feel free to contribute to this project.

If you find a bug or want a feature, but don't know how to fix/implement it, please fill an [`issue`](https://github.com/andypangaribuan/squirrel/issues).  
If you fixed a bug or implemented a feature, please send a [`pull request`](https://github.com/andypangaribuan/squirrel/pulls).

## MIT License

Copyright (c) 2025 Andy Pangaribuan

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
