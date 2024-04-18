# Apple Studio Display Brightness Control

A small command line utility to get or set the brightness level for Apple
Studio Displays in node.

```typescript
import * as asdbctl from "asdbctl";

// Check if a Studio Display is connected
asdbctl.hasDisplay();

// Get the current brightness as a percent.
console.log(asdbctl.getBrightness()); // 100

//Set the brightness as a percent.
console.log(asdbctl.setBrightness(50));

console.log(asdbctl.getBrightness()); // 50
```

## Background

Dumping the USB traffic on macOS with Wireshark and setting the proper filters
will show the USB Control transfers to set the brightness::

    bmRequestType: 0x21
    bRequest     : 0x9
    wValue       : 0x0301
    wIndex:      : 0x000c
    wLength      : 0x7

with this data package when setting it to the minimum brightness value::

    [ 0x01, 0x90, 0x01, 0x00, 0x00, 0x00, 0x00 ]

The 0x90 and 0x01 are the brightness value encoded with the least significant
byte first (LSB).

Its possible to operate the Studio Display in 3 different USB configurations
and Linux will use the first. This means that the USB interface number for
controlling the brightness is not `0xc` (extracted from the dump) but `0x7`.
