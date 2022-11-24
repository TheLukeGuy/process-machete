# Process Machete 🔪

This is a very simple + lightweight tool to automate the killing of processes. It waits for the configured processes to spawn, kills them when they do, and then kills itself when all configured processes are dead.

You *probably* don't need this tool for anything, but there are a few situations where you might:

* You need a program to run on startup, but you don't want it to keep running after it launches.
    * Possibly because the program uses too many resources to reasonably keep it running.
    * For example, RGB lighting control programs.
* Your operating system starts processes automatically and prevents you from disabling them normally.
* You like killing things with machetes. Concerning.

## Features

See [config.toml](resources/config.toml). I don't feel like typing them all out here.

## Supported OSes

Thanks to the cross-platform [`sysinfo`](https://crates.io/crates/sysinfo) crate, most major operating systems are supported.

| OS      | Supported | Tested | Startup supported[^1] | Notes                                                                                                                                                                                                    |
|---------|-----------|--------|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Windows | Yes       | Yes    | Yes                   |                                                                                                                                                                                                          |
| Linux   | Yes       | No     | No                    |                                                                                                                                                                                                          |
| macOS   | Yes       | No     | No                    |                                                                                                                                                                                                          |
| FreeBSD | Yes       | No     | No                    |                                                                                                                                                                                                          |
| Android | Probably  | No     | No                    | It should theoretically work on any device, but it *may* require root. Use [Termux](https://termux.dev/en/) or similar, optionally with [Termux:Boot](https://f-droid.org/en/packages/com.termux.boot/). |
| iOS     | Sort of   | No     | No                    | Your device must be jailbroken. Use [NewTerm 2](https://chariz.com/get/newterm) or similar. Alternatively, tweaks exist to run commands on boot/jailbreak.                                               |

[^1]: Even if this is "no", you can still manually add it as a startup program yourself. It just can't be done automatically.

## Using

1. Run the executable once for it to generate the configuration:
   ```
   $ process-machete
   [INFO] A default config.toml file has been created in the same folder as this executable. Configure it!
   ```
2. Open `config.toml` and configure it to your liking.
3. Run the executable again and watch it swiftly kill the processes you configured it to kill:
   ```
   $ process-machete
   [INFO] Started watching for 2 processes!
   [INFO] Found: SignalRgb.exe (pid 24504)
   [WARN] Killed: SignalRgb.exe (pid 24504)
   [INFO] Found: Notepad.exe (pid 22516)
   [INFO] Found: Notepad.exe (pid 13108)
   [WARN] Killed: Notepad.exe (pid 22516)
   [WARN] Killed: Notepad.exe (pid 13108)
   [INFO] Done! Killed 3 total processes, or 2/2 (100%) of configured processes.
   ```
4. Enjoy never thinking about those processes again. Or regret the atrocity you just committed. 🎉

### Running on operating system startup

For now, this only supports Windows. On other operating systems, you'll need to add or remove a startup program manually.

To start running it on startup:

```
$ process-machete startup add
[INFO] Added a startup program!
```

To stop running it on startup:

```
$ process-machete startup remove
[INFO] Removed a startup program!
```

If you ever move the executable, you'll need to add the startup program again (which will replace the old entry).

## Building

Install [Rust 1.65](https://www.rust-lang.org/tools/install) or later, then you can simply clone this repository and run:

```
cargo build --release
```

The compiled executable will then be at `target/release/process-machete[.exe]`. You can omit the `--release` flag if you want to build it with debug mode enabled.

## License

Process Machete is licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0))
* MIT License ([LICENSE-MIT](LICENSE-MIT) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))

at your option.
