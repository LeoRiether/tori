# Troubleshooting

## [E] pw.loop [loop.c:67 pw_loop_new()] 0x7fb768010570: can't make support.system handle: No such file or directory

This happens when mpv, tori's audio backend, doesn't find PipeWire installed in
your system.

You can fix this either by installing PipeWire or by setting
`mpv_ao: <your preferred audio output>` in [the configuration file](/tori/configuration).
`mpv_ao: pulse` and `mpv_ao: alsa` are popular choices. The available
outputs can be listed by running `mpv --ao=help` in the terminal.

## Linking error with xcb (ld returned 1 exit status)
```
/usr/bin/ld: cannot find -lxcb: No such file or directory         
/usr/bin/ld: cannot find -lxcb-render: No such file or directory
/usr/bin/ld: cannot find -lxcb-shape: No such file or directory 
/usr/bin/ld: cannot find -lxcb-xfixes: No such file or directory
collect2: error: ld returned 1 exit status          
```

If you're running a Debian-based distribution like Ubuntu, you can fix this by running
```bash
sudo apt install -y libxcb-shape0-dev libxcb-xfixes0-dev
```

## VersionMismatch
```
Error: VersionMismatch { linked: 131072, loaded: 65645 }
```

Your version of mpv is too old. You can fix this by installing a newer version of mpv, like v0.35.1

## The visualizer doesn't show up

This may happen for a few reasons:

1. [cava](https://github.com/karlstav/cava) is not installed
2. [cava](https://github.com/karlstav/cava) is not on your `PATH`.
3. [cava](https://github.com/karlstav/cava) is throwing an error. If tori does
   not show you the error for some reason, you may be able to see it by running
   `cava` in the terminal.

If the visualizer still does not show up after checking the above, please open [an issue!](https://github.com/LeoRiether/tori/issues).
