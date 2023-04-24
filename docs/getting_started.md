# Getting Started

## Dependencies
- [mpv](https://mpv.io/)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) (recommended) or youtube-dl
- [cava](https://github.com/karlstav/cava) (optional) for the visualizer

## Installing
- Make sure you have the dependencies installed
- Run `cargo install tori`

### yt-dlp
If you want to use yt-dlp instead of youtube-dl, edit your `mpv.conf` and paste the following line:
```conf
script-opts=ytdl_hook-ytdl_path=yt-dlp
```

Either this or follow [the guide I followed :)](https://www.funkyspacemonkey.com/replace-youtube-dl-with-yt-dlp-how-to-make-mpv-work-with-yt-dlp)

