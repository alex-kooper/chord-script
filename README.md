# chord-script
A CLI for clean, chord-first charts focused on form and structure

## Usage

The package is `chord-script`; the command it installs is `chords`.

```
chords song.chords                 # writes song.pdf next to the input
chords song.chords -o out.pdf      # explicit output; the extension selects the format
chords song.chords -o song.svg     # song.svg, or song-1.svg, song-2.svg, ... for several pages
chords *.chords                    # one PDF per chart
```

Parse errors are reported against the source on stderr. Compilation continues past a failing
chart, and the exit code is 1 if any chart failed.
