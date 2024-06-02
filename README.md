# lamzu-cfg

Lamzu mouse configuration tool for linux.

This CLI tool provides a way to configure Lamzu mice on linux (currently only
tested on the Atlantis Mini Pro). The configuration protocol has been
reverse-engineered from a Lamzu Atlantis Mini Pro, but may work on other Lamzu
devices.


## Table of Contents

- [Disclaimer](#disclaimer)
- [Install](#install)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)


## Disclaimer

This tool is not official Lamzu software, but is instead based on the
reverse-engineering of the configuration protocol used. As such, you take all
responsibility for any damages resulting from the use of this tool. Don't blame
me if your mouse explodes. It is recommended to backup your mouse configuration
before using this tool.


## Install

Make sure you have rust and cargo installed (stable or nightly is fine), then
run:

```sh
cargo install --git https://github.com/leadsun/lamzu-cfg
```


## Usage

### Supported Devices

`lamzu-cfg` has only been tested with the Lamzu Atlantis Mini Pro, but
other Lamzu mice may also work. `lamzu-cfg` will notify you if your mouse
appears to be a compatible but untested model, and will ask you to use the
`--force` flag to continue. If you then find that your mouse works well with
`lamzu-cfg`, then feel free to submit an issue to have your mouse marked as
tested.


### Permissions

`lamzu-cfg` requires access to `/dev/hidraw*` to function. Running as root is
the simplest option to provide this access:

```sh
sudo lamzu-cfg get --profile 1
```

Instead, if you would rather *not* run sketchy code from github as root, you can
temporarily alter the permissions. For each of the `/dev/hidraw` files that
`lamzu-cfg` complains about missing permissions for, set the permissions as
follows:

**Warning: This allows any user to read/write to your USB devices.**

```sh
sudo chmod o+r,o+w /dev/hidrawX # Replace X with the hidraw numbers printed.
```

These changes only persist until reboot or device disconnection, but it is still
recommended to change the permissions back when you're finished:

```sh
sudo chmod o-r,o-w /dev/hidraw*
```


### Active profile

Get and set the active profile on the mouse. Profiles are numbered from 1.

```sh
# Get currently active profile.
sudo lamzu-cfg get-active

# Switch to the second profile.
sudo lamzu-cfg set-active 2
```


### Profiles

Print profiles one-at-a-time, or all at once. Use `--json` flag for JSON output
instead of the default RON format.

```sh
# Print profile 3.
sudo lamzu-cfg get --profile 3
sudo lamzu-cfg get -p 3

# Get all profiles.
sudo lamzu-cfg get

# Save profiles for later.
sudo lamzu-cfg get > profiles.ron

# Print profile 4 in JSON format.
sudo lamzu-cfg get --profile 4 --json
```

Writing profiles works similarly. Input profile data can be provided inline,
from a file, or from standard input. Use `--json` for JSON input.

```sh
# Write profile 3 from file or STDIN.
sudo lamzu-cfg set --profile 3 --file profile3.ron
cat profile3.ron | sudo lamzu-cfg set --profile 3

# Provide part of a profile inline.
sudo lamzu-cfg set --profile 3 '(poll_rate: 500, debounce_ms: 2)'

# Write all profiles from one file.
sudo lamzu-cfg set -f profiles.ron

# Use JSON.
sudo lamzu-cfg set --profile 1 -f profile1.json --json
sudo lamzu-cfg set -f profiles.json --json
```


## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as below, without any additional terms or conditions.


## License

&copy; 2024 Contributors of Project lamzu-cfg.

This project is licensed under either of

- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) ([`LICENSE-APACHE`](LICENSE-APACHE))
- [MIT license](https://opensource.org/licenses/MIT) ([`LICENSE-MIT`](LICENSE-MIT))

at your option.

The [SPDX](https://spdx.dev) license identifier for this project is `MIT OR Apache-2.0`.
