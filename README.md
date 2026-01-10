# LinSplit

A Rust reimplementation of [the Celeste AutoSplitter](https://github.com/ShootMe/LiveSplit.Celeste), for Linux.

## Installation

You can just install it using [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html), if you've got it installed : 
```
cargo install linsplit
```

If you don't have cargo installed, you can download binaries from the [Releases](https://github.com/Paloys/linsplit/releases) tab.
If/when the projet gets big enough, I might try to upload it to various package managers.

## Usage

To use it, just run it using `linsplit` if it's in your path, or with `./linsplit` wherever you put the executable if not.
LinSplit will then start listening on a port for a LiveSplit One connection. Once it has connected to LiveSplit One, it'll start searching for either Celeste or Everest (the modded version of Celeste) in the memory.
The way LinSplit detects Vanilla Celeste in by finding a specific object in the memory of the program by parsing your save files for your file timers (if you're not using the game from Steam, you might want to change the location with the `-f` argument). It is recommended you start LinSplit inside an already started game file, while inside the menu (the 3D map where you choose your chapter). If it fails to detect on the first time, interrupt linsplit with Ctrl-C and try again.

### Arguments

You can pass arguments to linsplit to change LinSplit's behaviour. All arguments can be detailed by running `linsplit --help`.
The only required argument is your splits file (with the `-s` argument), which is the same file you're using on LiveSplit One, or on LiveSplit if you came from Windows.


## Bug reporting / Suggestions

Open an issue ! I'll try to read it as soon as possible. If you're reporting an issue, please provide instructions to recreate it.

## Contributing

My Rust isn't very good, I'm almost no longer a beginner but I don't feel really confident. If you have Rust experience and want to help with the structure or anything else, feel free to open an issue or a pull request ! I just have one go-to rule : no AI-generated code.

## Credits

Credits are due to the developers of [LiveSplit.Celeste](https://github.com/ShootMe/LiveSplit.Celeste) from which I took most of the logic used in this program. I also want to thank the developers of [LiveSplit](https://github.com/LiveSplit/livesplit) (and [livesplit-core](https://github.com/LiveSplit/livesplit-core)), as I took a bit of code (the API part) from livesplit-core and because LiveSplit is an amazing software.
