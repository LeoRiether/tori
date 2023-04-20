# ![tori](https://raw.githubusercontent.com/LeoRiether/tori/assets/assets/tori_64x60.png) tori
### The frictionless music player for the terminal

tori is a terminal-based music player and playlist manager that can play music from local files
and external URLs (supported by yt-dlp).

![tori](https://user-images.githubusercontent.com/8211902/230677856-02e4886e-84bf-4d21-ad70-0a625df4f24a.jpg)

## Features
- Plays songs from local files and external URLs
- Configurable keybinds
- Filters songs by name, artist or filepath/URL
- Sorts songs by name or duration
- Spectrum visualizer

## Dependencies
- [mpv](https://mpv.io/)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) (recommended) or youtube-dl
- [cava](https://github.com/karlstav/cava) (optional) for the visualizer

## Installing
- Make sure you have the dependencies installed
- Run `cargo install tori`

### yt-dlp
If you want to use yt-dlp instead of youtube-dl, edit your `mpv.conf` and paste the following line:
```
script-opts=ytdl_hook-ytdl_path=yt-dlp
```

Either this or follow [the guide I followed :)](https://www.funkyspacemonkey.com/replace-youtube-dl-with-yt-dlp-how-to-make-mpv-work-with-yt-dlp)

## Configuration
Configuration can be defined in $CONFIG_DIR/tori.yaml, where $CONFIG_DIR is,
depending on your operating system:

| Platform | Value                                 | Example                                  |
| -------  | ------------------------------------- | ---------------------------------------- |
| Linux    | `$XDG_CONFIG_HOME` or `$HOME`/.config | /home/alice/.config                      |
| macOS    | `$HOME`/Library/Application Support   | /Users/Alice/Library/Application Support |
| Windows  | `{FOLDERID_LocalAppData}`             | C:\Users\Alice\AppData\Local             |

The default directory tori uses to store playlists also depends on your OS:

| Platform | Value                   | Example                   |
| -------  | ------------------      | --------------------      |
| Linux    | `XDG_MUSIC_DIR`/tori    | /home/alice/Music/tori    |
| macOS    | `$HOME`/Music/tori      | /Users/Alice/Music/tori   |
| Windows  | `{FOLDERID_Music}`/tori | C:\Users\Alice\Music\tori |

Here's the default configuration file:
```yaml
playlists_dir: {audio_dir describe in the above table}
visualizer_gradient:
  - [46, 20, 66]
  - [16, 30, 71]
normal:
  '?': OpenHelpModal
  C-c: Quit
  C-d: Quit
  ">": NextSong
  "<": PrevSong
  " ": TogglePause
  S-right: SeekForward
  S-left: SeekBackward
  o: OpenInBrowser
  y: CopyUrl
  t: CopyTitle
  A-up: VolumeUp
  A-down: VolumeDown
  m: Mute
  v: ToggleVisualizer
  s: NextSortingMode
  R: Rename
  X: Delete
  S-down: SwapSongDown
  S-up: SwapSongUp
  J: SwapSongDown
  K: SwapSongUp
  ",": Shuffle
  h: SelectLeft
  j: SelectNext
  k: SelectPrev
  l: SelectRight
  a: Add
  q: QueueSong
  A-enter: QueueShown
  p: PlayFromModal
  E: OpenInEditor
```

You can override shortcuts in your config file, or remove some by binding them to `Nop` like so:
```yaml
    A-enter: Nop
```

## Alternatives
- [musikcube](https://github.com/clangen/musikcube) is what I used before writing tori.
  It's a great player, but only plays from local files.
- [cmus](https://cmus.github.io/)
- [yewtube](https://github.com/mps-youtube/yewtube)
