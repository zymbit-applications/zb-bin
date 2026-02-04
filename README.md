# zb-bin


## Installing Zymbit Bootware tools
```
curl -sSf https://raw.githubusercontent.com/zymbit-applications/zb-bin/main/install.sh | sudo bash
```

### To install non-interactively:

- Download the installer from the "releases" section of this repo or build it yourself
- Run:
```
./zb-install [--with-hardware-signing | --with-software-signing] [--zb-version <latest|VERSION_TAG>] [--rpi-model <MODELSPEC>]
```
- Note: specifying `--zb-version latest` will download and install the latest non-beta version of `zbcli`.

**OR**

Add the `-s` flag to `bash` and specify installer arguments:
```
curl -sSf \
    https://raw.githubusercontent.com/zymbit-applications/zb-bin/main/install.sh \
    | sudo bash -s -- <installer-args>
```
