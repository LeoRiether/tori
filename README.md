# tori
## Terminal-based music player

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
- [yewtube](https://github.com/mps-youtube/yewtube)
