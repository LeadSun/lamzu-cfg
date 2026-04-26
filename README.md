# lamzu-cfg

Lamzu mouse configuration tool for linux.

This CLI tool provides a way to configure Lamzu mice on linux (currently only
tested on Atlantis / Thorn). The configuration protocol has been
reverse-engineered from a Lamzu Atlantis Mini Pro, but may work on other Lamzu
devices.


## Table of Contents

- [Disclaimer](#disclaimer)
- [Install](#install)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)


## Disclaimer

This is not official Lamzu software, but is instead based on the
reverse-engineering of the configuration protocol used. Don't blame me if your
mouse explodes. Backing up your config would be a good idea.


## Install

Make sure you have rust and cargo installed (stable or nightly is fine), then
run:

```sh
cargo install --git https://github.com/leadsun/lamzu-cfg
```


## Usage

### Supported Devices

`lamzu-cfg` has only been tested with Lamzu Atlantis / Thorn mice, but other
Lamzu mice may also work. `lamzu-cfg` will notify you if your mouse appears to
be an untested model, and will ask you to use the `--force` flag to continue.
**Use this flag at your own risk.** If you then find that your mouse works well
with `lamzu-cfg`, then feel free to submit an issue to have your mouse marked as
tested.


### Permissions

`lamzu-cfg` requires access to `/dev/hidraw*` to function. Running as root is
the simplest option to provide this access:

```sh
sudo lamzu-cfg get --profile 1
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


### Profile Example

```ron
(
    poll_rate: 1000,
    current_resolution_index: 2,
    lift_off_distance: 1,
    debounce_ms: 8,
    motion_sync: true,
    angle_snapping: false,
    ripple_control: false,
    peak_performance: true,
    peak_performance_time: 30,
    high_performance: false,
    resolutions: [
        (
            x: 400,
            y: 400,
        ),
        (
            x: 800,
            y: 800,
        ),
        (
            x: 1600,
            y: 1600,
        ),
        (
            x: 3200,
            y: 3200,
        ),
        (
            x: 6400,
            y: 6400,
        ),
    ],
    resolution_colors: [
        (
            red: 255,
            green: 0,
            blue: 0,
        ),
        (
            red: 0,
            green: 255,
            blue: 255,
        ),
        (
            red: 0,
            green: 255,
            blue: 0,
        ),
        (
            red: 255,
            green: 255,
            blue: 255,
        ),
        (
            red: 255,
            green: 255,
            blue: 0,
        ),
    ],
    button_map: {
        Left: LeftClick,
        Right: RightClick,
        Middle: MiddleClick,
        Back: Fire(
            interval: 50,
            repeat: 2,
        ),
        Forward: Combo(
            events: [
                (
                    key: UsA,
                    state: Pressed,
                ),
                (
                    key: UsA,
                    state: Released,
                ),
                (
                    key: UsB,
                    state: Pressed,
                ),
                (
                    key: UsB,
                    state: Released,
                ),
                (
                    key: UsC,
                    state: Pressed,
                ),
                (
                    key: UsC,
                    state: Released,
                ),
            ]
        ),

        Bottom: Macro(
            name: "example macro",
        ),
    },
    macros: {
        "example macro": (
            mode: Repeat(1),
            events: [
                (
                    key_event: (
                        key: UsA,
                        state: Pressed,
                    ),
                    delay_ms: 100,
                ),
                (
                    key_event: (
                        key: UsA,
                        state: Released,
                    ),
                    delay_ms: 100,
                ),
                (
                    key_event: (
                        key: UsB,
                        state: Pressed,
                    ),
                    delay_ms: 100,
                ),
                (
                    key_event: (
                        key: UsB,
                        state: Released,
                    ),
                    delay_ms: 100,
                ),
                (
                    key_event: (
                        key: UsC,
                        state: Pressed,
                    ),
                    delay_ms: 100,
                ),
                (
                    key_event: (
                        key: UsC,
                        state: Released,
                    ),
                    delay_ms: 100,
                ),
            ],
        ),
    },
)
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
