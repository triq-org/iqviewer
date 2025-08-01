# IQViewer -- SDR I/Q data file viewer app

A somewhat minimal hack of an I/Q data file browser and viewer app.

| OS | Arch |
|---|---|
| Linux   | [x86_64](https://github.com/triq-org/iqviewer/releases/download/0.6/IQViewer-Linux-amd64.zip) &nbsp; [ARM64](https://github.com/triq-org/iqviewer/releases/download/0.6/IQViewer-Linux-arm64.zip) &nbsp; [ARMv7](https://github.com/triq-org/iqviewer/releases/download/0.6/IQViewer-Linux-armv7.zip)|
| NetBSD  | [x86_64](https://github.com/triq-org/iqviewer/releases/download/0.6/IQViewer-NetBSD-amd64.zip) |
| FreeBSD | [x86_64](https://github.com/triq-org/iqviewer/releases/download/0.6/IQViewer-FreeBSD-amd64.zip) |
| Windows | [x64](https://github.com/triq-org/iqviewer/releases/download/0.6/IQViewer-Windows-x64.zip) |
| macOS   | [intel + arm](https://github.com/triq-org/iqviewer/releases/download/0.6/IQViewer.dmg) |

![Screenshot](web/IQViewer.png)

Supports files and folders as CLI arguments and drag and drop of files and folders.

Folders added as CLI argument, opened or dropped will be watched for changes.

For ideas on more features, feedback and discussion see this [Issue](https://github.com/triq-org/iqviewer/issues/1)

## Supported file types
- `.cu4`,
- `.cs4`,
- `.cu8`, `.data`., `.complex16u`,
- `.cs8`, `.complex16s`,
- `.cu12`,
- `.cs12`,
- `.cu16`,
- `.cs16`,
- `.cu32`,
- `.cs32`,
- `.cu64`,
- `.cs64`,
- `.cf32`, `.cfile`, `.complex`,
- `.cf64`,
- `.sigmf`,

## Controls and Hotkeys

### Browser
- <kbd>o</kbd> → open files
- <kbd>O</kbd> → open and watch folder
- <kbd>x</kbd> → clear list and watches
- <kbd>DEL</kbd> → remove item
- <kbd>d</kbd> → mark file for delete
- <kbd>f</kbd> → mark file for move
- <kbd>m</kbd> → mark file for move
- <kbd>D</kbd> → delete marked
- <kbd>M</kbd> → move marked
- <kbd>SPACE</kbd> → toggle viewer
- <kbd>l</kbd> → toggle thumbnail size
- <kbd>z</kbd> → toggle viewer size
- <kbd>s</kbd> → focus filter/search
- <kbd>q</kbd> → quit app
- <kbd>h</kbd> → toggle help
- <kbd>↑</kbd> <kbd>↓</kbd> <kbd>←</kbd> <kbd>→</kbd> → move selection
- <kbd>⤒</kbd> <kbd>⤓</kbd> → move first / last

## Viewer
- <kbd>ESC</kbd> → close viewer
- <kbd>SPACE</kbd> → toggle viewer
- <kbd>+</kbd> → zoom in
- <kbd>-</kbd> → zoom out
- <kbd>0</kbd> → reset zoom

## Viewer mouse controls
- <em>Scroll Wheel</em> → zoom
- <em>Horizontal Scroll</em> → pan
- <em>Click+Drag</em> → pan
- <em>Middle Click</em> → zoom in
- <em>Right Click</em> → zoom out
- <em>Hold Shift</em> → measure
- <em>Shift+Click</em> → set a marker

## License

The AGPL 3.0 or later. Please see [license file](LICENSE) for more information.
