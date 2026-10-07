# Why Omnifetch is Fast

## Minimal Subprocesses

Most fetchers shell out dozens of times on every run (`awk`, `sed`, `lscpu`, `grep`) and then wait for each subprocess. Omnifetch reads `/proc`, `/sys`, DMI tables, DRM connectors, and netlink sockets directly, so all standard system and hardware modules run with zero `fork`/`exec`.

Subprocesses (via `Command::new`) are strictly restricted to optional fallback probes where direct sysfs/procfs interfaces are insufficient or proprietary:
- Proprietary GPU stats (`nvidia-smi`)
- Desktop settings where direct D-Bus access is unavailable (`gsettings`)
- MPRIS player control (`busctl`)
- Development environment toolchain queries in the `devenv` module
- User-specified commands in the `command` module

Resolving a hostname is bounded too. The system resolver has no timeout of its own and can sit on a silent nameserver for five seconds, so lookups run under an explicit deadline.

## The Cache

Anything that does not change during a session goes into a shared file in `/dev/shm` and is read back in 0.000 ms: CPU, GPU, motherboard, BIOS, audio server, desktop environment. It invalidates itself on reboot, on package install, and on config edits, so nothing needs to be cleared manually.

Dynamic peripherals are the exception: keyboards, mice, and touchpads are probed on the fly (in roughly 0.2 ms) because they can be plugged in or removed at any moment.

A value only reaches the cache once it is real. An empty result usually means the hardware is not attached yet, and caching that would freeze the "nothing found" answer for the rest of the session.

## Network Modules

The two modules that need the internet run in parallel threads, so the total latency is determined by the slowest of them rather than their sum (around 110 ms). They stay behind `--network` and never run by default.

## Where the Time Goes

On a warm cache, module execution takes roughly 0.9 ms. Total execution time, including binary startup, lands at ~2.1 ms (compared to 0.57 ms for an empty binary on the same machine).

That 0.57 ms difference is why release binaries can be linked statically: dynamic loaders spend time searching missing directories listed in `/etc/ld.so.conf.d` (e.g. flatpak, nix, linuxbrew).

You can reproduce benchmarks locally:

```bash
python3 benchmark.py  # outputs benchmark_results.md
```