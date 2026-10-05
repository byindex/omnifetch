WHY OMNIFETCH IS FAST
=====================

MINIMAL SUBPROCESSES
--------------------

Most fetchers shell out dozens of times on every run: awk, sed, lscpu, grep, and
then wait for each one. Omnifetch opens /proc, /sys, the DMI tables, DRM
connectors and netlink sockets directly, so all standard system and hardware modules
run with zero fork/exec.

Subprocesses (via `Command::new`) are strictly restricted to optional fallback probes
where direct sysfs/procfs interfaces are insufficient or proprietary:
- Proprietary GPU stats (`nvidia-smi`)
- Desktop settings where direct D-Bus access is unavailable (`gsettings`)
- MPRIS player control (`busctl`)
- Development environment toolchain queries in the `devenv` module
- User-specified commands in the `command` module

Resolving a hostname is bounded too. The system resolver has no timeout of its
own and will sit on a silent nameserver for five seconds, so lookups run under
a deadline of their own.

THE CACHE
---------

Anything that does not change during a session goes into a shared file in
/dev/shm and is read back in 0.000 ms: CPU, GPU, board, BIOS, audio server,
desktop environment. It invalidates itself on reboot, on package install and on
a config edit, so nothing is ever cleared by hand.

Dynamic peripherals are the exception. Keyboards, mice and touchpads are probed
on the fly, in roughly 0.2 ms, because they can be plugged in at any moment.

A value only reaches the cache once it is real. An empty result usually means
the hardware is not attached yet, and storing that would freeze the "nothing
found" answer for the rest of the session.

NETWORK MODULES
---------------

The two modules that need the internet run in parallel threads, so the total is
about as long as the slowest of them rather than their sum: around 110 ms.
They stay behind --network and never run by default.


WHERE THE TIME GOES
-------------------

On a warm cache the modules themselves take roughly 0.9 ms. The whole process,
binary startup included, lands at 2.1 ms, against 0.57 ms for an empty program
on the same machine.

That 0.57 ms is why the binary is linked statically. The dynamic loader spends
most of that time on searches that fail, because /etc/ld.so.conf.d lists
directories such as linuxbrew, flatpak and nix that are absent on many systems.

Reproduce any of this with python3 benchmark.py; it writes benchmark_results.md.