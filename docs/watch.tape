# VHS tape — records the `watch` TUI for the README.
# Regenerate with: vhs docs/watch.tape
Output docs/watch.gif

Set FontSize 13
Set Width 1220
Set Height 740
Set Padding 12

Sleep 500ms
Type "./target/release/backpack-battles watch --seed 42 --speed 3"
Enter
Sleep 2s
Screenshot docs/watch.png
Sleep 6s
