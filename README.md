# lamzu-cfg (WIP)

lamzu-cfg is currently a WIP and not usable yet.

This CLI tool provides a way to configure Lamzu mice on linux (currently only
tested on the Atlantis Mini Pro). The configuration protocol has been
reverse-engineered from a Lamzu Atlantis Mini Pro, but may work on other Lamzu
devices.


## Protocol

Reverse-engineered from a Lamzu Atlantis Mini Pro. All configuration is done via
HID reports with an ID of `0x08`. These reports are 17 bytes long including the
report ID byte, with a format as follows:

|Field                         |Length (Bytes)|Endian|
|------------------------------|--------------|------|
|Report ID (`0x08`)            |1             |      |
|[Command](#commands) ID       |1             |      |
|Error code (`0x00` is OK)     |1             |      |
|Address for R/W               |2             |Big   |
|Payload length (bytes, max 10)|1             |      |
|Payload                       |10            |      |
|Checksum                      |1             |      |

**Checksums**

Checksums are calculated as a sum complement with an initial value of 171 (or
181 for macros).

**Endianness**

The protocol uses mixed endianness for consistency and ease-of-use
🙃.


### Commands

Requests will generally trigger a response of the same command type. Check the
response error code field for errors (`0x00` for OK).

|Name              |ID    |Address            |Request Payload                |Response Payload               |
|------------------|------|-------------------|-------------------------------|-------------------------------|
|Unknown           |`0x02`|`0x0000`           |                               |                               |
|Unknown           |`0x03`|`0x0000`           |                               |                               |
|Unknown           |`0x04`|`0x0000`           |                               |                               |
|Set profile data  |`0x07`|`0x0000` - `0x1aff`|[Profile data](#profiles)      |                               |
|Get profile data  |`0x08`|`0x0000` - `0x1aff`|                               |[Profile data](#profiles)      |
|Unknown           |`0x0a`|`0x0000`           |                               |                               |
|Get active profile|`0x0e`|`0x0000`           |                               |Profile index (`0x00` - `0x03`)|
|Set active profile|`0x0f`|`0x0000`           |Profile index (`0x00` - `0x03`)|                               |
|Unknown           |`0x12`|`0x0000`           |                               |                               |
|Unknown           |`0x17`|`0x0000`           |                               |                               |


### Profiles

The Lamzu Atlantis supports 4 profiles, but only the active profile can be read
from / written to. The individual settings of the active profile are stored
sequentially and can be accessed as bytes. Up to 10 bytes can be read / written
at a time. The address field of the command is a byte offset from the start of
the profile data. Each setting has a checksum byte appended (except key combos
and macros which already have a checksum before their padding).

The protocol supports up to 8 DPI presets but the Lamzu desktop software only
uses 5. Up to 16 button mappings, combos, and macros are also supported even
though the Atlantis only has 6 buttons.

**Settings**

|Name                 |Length|Format                                                           |
|---------------------|------|-----------------------------------------------------------------|
|**Offset `0x0000`**  |      |                                                                 |
|Report rate          |2     |1000Hz (`0x01`), 500Hz (`0x02`), 250Hz (`0x04`), 125Hz (`0x08`)  |
|DPI count            |2     |`0x01` - `0x08`                                                  |
|Current DPI index    |2     |`0x00` - `0x07`                                                  |
|Unknown settings     |4     |2 x 2 byte settings                                              |
|Lift-off distance    |2     |1mm (`1`) or 2mm (`2`)                                     |
|DPI presets (x 8)    |4     |x DPI + y DPI + `0x00` (DPI in multiples of 50 with `0x00` = 50) |
|DPI colors (x 8)     |4     |Red + green + blue                                               |
|Unknown settings     |8     |4 x 2 byte settings                                              |
|Charging LED colour? |4     |Red + green + blue                                               |
|Unknown settings     |8     |4 x 2 byte settings                                              |
|Button actions (x 16)|4     |[Action](#actions) mapped to button                              |
|Unknown settings     |9     |7 byte setting + 2 byte setting                                  |
|Debounce ms          |2     |Time in ms: `0` - `15`                                      |
|Motion sync          |2     |Off (`0x00`) or on (`0x01`)                                      |
|Unknown setting      |2     |                                                                 |
|Angle snapping       |2     |Off (`0x00`) or on (`0x01`)                                      |
|Ripple control       |2     |Off (`0x00`) or on (`0x01`)                                      |
|Unknown setting      |2     |                                                                 |
|Peak performance     |2     |Off (`0x00`) or on (`0x01`)                                      |
|Peak performance time|2     |Multiples of 10s (`0x03`, `0x06`, `0x0c`, `0x1e`, `0x3c`, `0x5a`)|
|Performance mode     |2     |Low performance (`0x00`) or high performance (`0x01`)            |
|**Offset `0x0100`**  |      |                                                                 |
|Key combo (x 16)     |32    |[Key Combo](#macros) mapped to button                            |
|**Offset `0x0300`**  |      |                                                                 |
|Macro (x 16)         |384   |[Macro](#macros) mapped to button                                |


### Actions

Actions can be mapped to mouse buttons.

|Name          |ID / parameters                                           |
|--------------|----------------------------------------------------------|
|Disabled      |`0x000000`                                                |
|Left click    |`0x010100`                                                |
|Right click   |`0x010200`                                                |
|Middle click  |`0x010400`                                                |
|Back click    |`0x010800`                                                |
|Forward click |`0x011000`                                                |
|DPI loop      |`0x020100`                                                |
|DPI up        |`0x020200`                                                |
|DPI down      |`0x020300`                                                |
|Scroll left   |`0x030100`                                                |
|Scroll right  |`0x030200`                                                |
|Fire key      |`0x04` + interval (`10` - `255`), repeat times (`0` - `3`)|
|Key combo     |`0x050000`                                                |
|Macro         |`0x0605` + macro index (same as button index: `0` - `15`) |
|Poll rate loop|`0x070000`                                                |
|DPI lock      |`0x0a` + DPI value (`0x01` - `0x17`) + `0x00`             |
|Scroll up     |`0x0b0100`                                                |
|Scroll down   |`0x0b0200`                                                |

**Note**

Make sure the appropriate key combo or macro is set if mapping a button to one.


### Macros

**Key Events**

Key press or release events. Format:

> 3 bytes: 1 byte flags + 2 bytes little endian key data

Flags:

|7        |6          |5      |4      |3      |2        |1     |0  |
|---------|-----------|-------|-------|-------|---------|------|---|
|Key press|Key release|Unknown|Unknown|Unknown|Direction|HID CC|HID|

Key data:

- Least significant 3 flag bits are zero: Interpret as a modifer mask.
- HID flag bit: Interpret as USB HID code.
- HID CC flag bit: Interpret as USB HID consumer control code.
- Direction flag bit: Interpret as direction mask (left, right, middle, back, forward).

**Key Combos**

Combinations of up to 3 keys (6 press / release events) that can be mapped to
buttons. Format:

> 32 bytes: key event count (up to `6`) + 6 key events + checksum + 12 bytes padding

**Macro Events**

Key events with added delay in ms. Format:

> 5 bytes: key event + 2 byte big endian delay in ms

**Macros**

Named sequences of up to 70 macro events that can be mapped to buttons. Format:

> 384 bytes: name length (up to `30`) + 30 name characters + macro event count (up to `70`) + 70 macro events + checksum + 1 byte padding
