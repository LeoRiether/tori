# tori
## Terminal-based music player

![tori](https://user-images.githubusercontent.com/8211902/228040682-7888b4f9-0dc3-425e-95ad-166a1737f388.png)
## Configuration
Configuration can be defined in $CONFIG_DIR/tori.yaml, where $CONFIG_DIR is, depending on your operating system:

|Platform | Value                                 | Example                                  |
| ------- | ------------------------------------- | ---------------------------------------- |
| Linux   | `$XDG_CONFIG_HOME` or `$HOME`/.config | /home/alice/.config                      |
| macOS   | `$HOME`/Library/Application Support   | /Users/Alice/Library/Application Support |
| Windows | `{FOLDERID_LocalAppData}`             | C:\Users\Alice\AppData\Local             |

Here's an example of a configuration file:
```yaml
playlists_dir: /home/leonardo/Music/tori
normal:
  C-c: Quit
  C-d: Quit
  j: SelectNext
  k: SelectPrev
  '>': NextSong 
  '<': PrevSong 
  q: QueueSong
  A-enter: QueueShown
  S-right: SeekForward
  S-left: SeekBackward
  o: OpenInBrowser
  y: CopyUrl # y for 'yank', like in vim
  t: CopyTitle
  ' ': TogglePause
  A-up: VolumeUp
  A-down: VolumeDown
  m: Mute
  p: PlayFromModal
  a: Add
  R: Rename
  X: Delete
  ',': Shuffle
```

## Dependencies
- [mpv](https://mpv.io/)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) (or youtube-dl)

## yt-dlp
If you want to use yt-dlp instead of youtube-dl, edit your `mpv.conf` and paste the following line:
```
script-opts=ytdl_hook-ytdl_path=yt-dlp
```

Either this or follow [the guide I followed :)](https://www.funkyspacemonkey.com/replace-youtube-dl-with-yt-dlp-how-to-make-mpv-work-with-yt-dlp)

## Alternatives
- [musikcube](https://github.com/clangen/musikcube) is what I used before writing tori.
  It's a great player, but it only plays from local files.
- [cmus](https://cmus.github.io/)
- [yewtube](https://github.com/mps-youtube/yewtube)
