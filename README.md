# ![tori](https://raw.githubusercontent.com/LeoRiether/tori/master/assets/tori_64x60.png) tori
### The frictionless music player for the terminal

tori is a terminal-based music player and playlist manager that can play music from local files
and external URLs (supported by yt-dlp).

![tori screenshot](https://user-images.githubusercontent.com/8211902/233261347-f1cb6597-0d2f-41e5-88b0-32590de43946.png)

## Features
- Plays songs from local files and external URLs
- Configurable keybinds
- Filters songs by name, artist or filepath/URL
- Sorts songs by name or duration
- Spectrum visualizer

## Documentation
tori's documentation is hosted [here](https://leoriether.github.io/tori/). It includes a [Getting Started guide](https://leoriether.github.io/tori/#getting_started/) and [configuration instructions](https://leoriether.github.io/tori/#configuration/).

For code-related documentation, there's also a [docs.rs entry](https://docs.rs/tori).

## Installing
- Make sure you have the dependencies installed
- Run `cargo install tori`

### Dependencies
- [mpv](https://mpv.io/)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) (recommended) or youtube-dl
- [cava](https://github.com/karlstav/cava) (optional) for the visualizer

### yt-dlp
If you're using yt-dlp instead of youtube-dl, edit your `mpv.conf` and paste the following line:
```conf
script-opts=ytdl_hook-ytdl_path=yt-dlp
```

Either this or follow [the guide I followed :)](https://www.funkyspacemonkey.com/replace-youtube-dl-with-yt-dlp-how-to-make-mpv-work-with-yt-dlp)
## Alternatives
- [musikcube](https://github.com/clangen/musikcube) is what I used before writing tori.
  It's a great player, but only plays from local files.
- [cmus](https://cmus.github.io/)
- [yewtube](https://github.com/mps-youtube/yewtube)
