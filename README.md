# squirrel

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
