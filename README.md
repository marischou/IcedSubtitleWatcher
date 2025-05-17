# IcedSubtitleWatcher
View (or watch) subtitle files even when you have no video to accompany it!

# What?
Inspired by vinceman's [subtitle-buddy application](https://github.com/vincemann/subtitle-buddy), I decided that it would be fun to try and recreate it in Iced. It's very rough around the edges, but the basic functionality works. Intentionally only supports .ass/.ssa/.srt files only because of unfinished input metadata sanitization.

# How to run?
```
git clone https://github.com/marischou/IcedSubtitleWatcher.git
cd IcedSubtitleWatcher
cargo run
```
# Basic functionality
- Press Space for pause and play
- Press Esc for toggling transparency on the whole application except the text, does not work for the window though, have to deal with that using your window manager
- Increase subtitle font size (or decrease)
- Change themes to predefined iced themes
- Change font if it's available on your system (subtitle file-defined font (like in srt) usage not yet implemented)
- Offset input to help shift your playback timing to match the player, for example when your subtitles are in separate file per episode, but your media is all in one continous playback
- Reset button for resetting the playback back to start
- Fast forward or reverse by 5 seconds
