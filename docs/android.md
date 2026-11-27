# Android CLI

The `wordcraft-cli` Android binary uses a portable `.wordcraft/` directory beside the
executable:

```text
.wordcraft/
├── cache/     XDG_CACHE_HOME
├── config/    XDG_CONFIG_HOME and XDG_CONFIG_DIRS
├── data/      XDG_DATA_HOME
├── state/     XDG_STATE_HOME
├── tmp/       TMPDIR
├── home/      HOME
├── fonts/     user fonts
├── hosts      reserved for optional extra hostname mappings
└── timezone   optional time zone, such as Asia/Shanghai
```

Non-empty values you set yourself for `XDG_CACHE_HOME`, `XDG_CONFIG_HOME`,
`XDG_CONFIG_DIRS`, `XDG_DATA_HOME`, `XDG_STATE_HOME`, `TMPDIR`, and `HOME` always win.
Non-empty `TZ` and `TZDIR` values are preserved; the binary's directory is prepended to `PATH`.

Fonts are read from `/system/fonts`, `.wordcraft/fonts`, and `$HOME/fonts`, in that order.
Hunspell dictionaries go in `.wordcraft/data/wordcraft/dictionaries` (the same
`$XDG_DATA_HOME/wordcraft/dictionaries` location used by Linux builds), or in a folder named
by `WORDCRAFT_DICTIONARIES`. Android does not have `/usr/share/hunspell`.
The `hosts` file is reserved for future use; WordCraft does not currently read mappings from it.

The binary must be in a directory that is both writable and executable, because WordCraft creates
the portable directory beside it. `/sdcard` is mounted `noexec` and cannot be used for the binary.

Language-pack and font downloads need a TLS trust store. `rustls-platform-verifier` needs a JVM
`Context` on Android, which a plain binary does not have, so those downloads may fail; when they
do, WordCraft reports the actual error rather than succeeding silently. At startup WordCraft
invokes `getprop` to read the time zone and, when locale environment variables are unset, the
system locale. If `getprop` is unavailable or fails, time-zone and paper selection fall back to
UTC and A4.
