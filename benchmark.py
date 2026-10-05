#!/usr/bin/env python3
"""Measure omnifetch against other fetchers and write benchmark_results.md.

Run it from the repository root:

    python3 benchmark.py

Needs hyperfine on PATH for the real numbers; without it a built-in timer
takes over and says so in the output.
"""

import argparse
import json
import os
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent
RESULTS = ROOT / "benchmark_results.md"

BOLD = "\033[1m"
GREEN = "\033[0;32m"
CYAN = "\033[0;36m"
YELLOW = "\033[1;33m"
GRAY = "\033[0;90m"
NC = "\033[0m"

# Сколько информационных блоков печатает каждый конкурент. Эти утилиты не
# отдают список модулей машиночитаемым образом, поэтому числа заданы руками,
# а не вытащены из вывода: раскладка у каждой своя.
SHOWN = {
    "fastfetch": 23,
    "neofetch": 16,
    "nitch": 9,
    "paleofetch": 11,
    "pfetch": 6,
    "catnap": 16,
    "macchina": 16,
    # sysprint печатает каждую точку монтирования отдельной строкой, но модуль один.
    "sysprint-linux": 23,
    "sysprint": 23,
}

LAYOUT_ONLY = {"break", "separator", "title", "colors", "total", "<total>"}


def find_binary():
    host = subprocess.run(["rustc", "-vV"], capture_output=True, text=True).stdout
    triple = re.search(r"^host: (\S+)", host, re.M)
    candidates = []
    if triple:
        candidates.append(ROOT / "target" / triple.group(1) / "release" / "omnifetch")
    candidates.append(ROOT / "target" / "release" / "omnifetch")
    return next((c for c in candidates if c.is_file()), None)


def build_binary():
    print(f"{YELLOW}Сборка release бинарника...{NC}")
    host = subprocess.run(["rustc", "-vV"], capture_output=True, text=True).stdout
    triple = re.search(r"^host: (\S+)", host, re.M)
    env = dict(os.environ, RUSTFLAGS="-C target-feature=+crt-static")
    subprocess.run(
        ["cargo", "build", "--release", "--target", triple.group(1)],
        cwd=ROOT, env=env, check=True,
    )
    return ROOT / "target" / triple.group(1) / "release" / "omnifetch"


def omnifetch_runs(binary, network, only_omnifetch):
    runs = [
        ("--fast --no-color", "omnifetch --fast"),
        ("--no-color", "omnifetch"),
        ("--no-cache --no-color", "omnifetch --no-cache"),
        ("--all --no-color", "omnifetch --all"),
        ("--all --no-cache --no-color", "omnifetch --all --no-cache"),
    ]
    if network:
        runs += [
            ("--network --no-color", "omnifetch --network"),
            ("--network --no-cache --no-color", "omnifetch --network --no-cache"),
            ("--all --network --no-color", "omnifetch --all --network"),
            ("--all --network --no-cache --no-color", "omnifetch --all --network --no-cache"),
        ]
    runs += [
        ("-p minimal --no-color", "omnifetch -p minimal"),
        ("-p compact --no-color", "omnifetch -p compact"),
        ("-p modern --no-color", "omnifetch -p modern"),
        ("-p hardware --no-color", "omnifetch -p hardware"),
        ("-p detailed --no-color", "omnifetch -p detailed"),
        ("-p fastfetch --no-color", "omnifetch -p fastfetch"),
        ("-p neofetch --no-color", "omnifetch -p neofetch"),
    ]
    return [(f"{binary} {args}", name) for args, name in runs]


def competitors():
    found = []
    local = ROOT / "dev" / "sysprint-linux"
    if local.is_file():
        local.chmod(0o755)
        found.append((str(local), "sysprint"))
    elif shutil.which("sysprint-linux"):
        found.append(("sysprint-linux", "sysprint"))

    for tool, args in [
        ("fastfetch", ""),
        ("macchina", ""),
        ("pfetch", ""),
        ("nitch", ""),
        ("paleofetch", ""),
        ("catnap", ""),
        ("neofetch", "--stdout"),
    ]:
        if shutil.which(tool):
            found.append((f"{tool} {args}".strip(), tool))
    return found


def module_count(binary, args):
    """Сколько информационных модулей отработало, без служебных блоков."""
    proc = subprocess.run(
        [binary, *args, "-l", "none", "--no-color", "--no-cache", "--timing"],
        capture_output=True, text=True,
    )
    names = set()
    for line in proc.stderr.splitlines():
        match = re.match(r"^\s*[\d.]+ ms  (\S+)", line)
        if match and match.group(1) not in LAYOUT_ONLY:
            names.add(match.group(1))
    return len(names)


def run_hyperfine(targets, warmup, runs, use_shell, export):
    command = ["hyperfine", "--warmup", str(warmup), "--min-runs", str(runs),
               "--export-json", export]
    if not use_shell:
        command.insert(1, "--shell=none")
    for cmd, name in targets:
        command += ["-n", name, cmd]
    subprocess.run(command, check=True)


def table(export, binary):
    results = json.loads(Path(export).read_text())["results"]
    rows = []
    for entry in results:
        parts = entry["command"].split(" ", 1)
        exe = Path(parts[0]).name
        args = parts[1].split() if len(parts) > 1 else []
        # Считать модули осмысленно только для самого omnifetch.
        if exe == "omnifetch":
            shown = str(module_count(binary, args))
        else:
            shown = str(SHOWN.get(exe, "-"))
        # hyperfine отдаёт секунды.
        rows.append((entry["command"], entry["mean"] * 1000, entry["min"] * 1000, shown))
    rows.sort(key=lambda row: row[1])

    width = max([len("Command")] + [len(row[0]) for row in rows])
    head = ("Command".ljust(width) + " " + "Modules".rjust(9)
            + " " + "Mean ms".rjust(9) + " " + "Min ms".rjust(9))
    lines = [head, "-" * len(head)]
    for cmd, mean, minimum, shown in rows:
        lines.append(f"{cmd.ljust(width)} {shown.rjust(9)} {f'{mean:.1f}':>9} {f'{minimum:.1f}':>9}")
    return "\n".join(lines)


def builtin_timer(targets, warmup, runs):
    """Запасной замер, когда hyperfine не установлен."""
    lines = ["Command".ljust(34) + " " + "Mean ms".rjust(9), "-" * 44]
    for cmd, _ in targets:
        for _ in range(warmup):
            subprocess.run(cmd, shell=True, capture_output=True)
        samples = []
        for _ in range(runs):
            start = time.perf_counter_ns()
            subprocess.run(cmd, shell=True, capture_output=True)
            samples.append((time.perf_counter_ns() - start) / 1e6)
        lines.append(f"{cmd[:34].ljust(34)} {statistics.median(samples):9.1f}")
    return "\n".join(lines)


def write_results(sections, warmup, runs):
    out = [
        "BENCHMARK RESULTS - omnifetch",
        "",
        f"hyperfine, {runs} runs, {warmup} warmup, static build. Lower is better.",
        "",
        "Two ways of starting the program are measured:",
        "",
        "VIA SHELL   the usual case: a terminal opens, your shell starts, and",
        "            omnifetch is in its startup file, so the shell runs it for you.",
        "NO SHELL    you type `omnifetch` yourself and it starts right away.",
        "",
        "Modules = how many information blocks the command printed.",
        "",
    ]
    out += sections
    RESULTS.write_text("\n".join(out) + "\n")


def main():
    parser = argparse.ArgumentParser(
        description="Benchmark omnifetch against other fetchers.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("-N", "--no-shell", action="store_true",
                        help="только без шелла")
    parser.add_argument("-S", "--shell", action="store_true",
                        help="только через системный шелл")
    parser.add_argument("--warmup", type=int, default=3,
                        help="число прогревочных запусков (по умолчанию: 3)")
    parser.add_argument("--runs", type=int, default=10,
                        help="минимальное число запусков (по умолчанию: 10)")
    parser.add_argument("--skip-network", action="store_true",
                        help="пропустить замеры сетевых флагов (--network)")
    parser.add_argument("--only-omnifetch", action="store_true",
                        help="только omnifetch, без конкурентов")
    args = parser.parse_args()

    if args.no_shell and args.shell:
        parser.error("--no-shell и --shell вместе не имеет смысла")

    binary = find_binary() or build_binary()

    targets = omnifetch_runs(binary, not args.skip_network, args.only_omnifetch)
    if not args.only_omnifetch:
        targets += competitors()

    print(f"{CYAN}{BOLD}OMNIFETCH BENCHMARK{NC}")
    print(f"{GRAY}{len(targets)} команд: {', '.join(name for _, name in targets)}{NC}\n")

    if not shutil.which("hyperfine"):
        print(f"{YELLOW}Hyperfine не найден, используется встроенный таймер.{NC}")
        print(f"{GRAY}Числа ниже ориентировочные: таймер считается внутри Python, вместе с его оверхедом.{NC}\n")
        write_results(["1. BUILT-IN TIMER", "", builtin_timer(targets, args.warmup, args.runs)],
                      args.warmup, args.runs)
    else:
        sections = []
        with tempfile.TemporaryDirectory() as tmp:
            if not args.shell:
                export = str(Path(tmp) / "noshell.json")
                print(f"{CYAN}{BOLD}[1/2] Без шелла{NC}")
                run_hyperfine(targets, args.warmup, args.runs, False, export)
                sections += ["", "1. NO SHELL", "", table(export, binary), ""]

            if not args.no_shell:
                export = str(Path(tmp) / "shell.json")
                print(f"{CYAN}{BOLD}[2/2] Через шелл{NC}")
                run_hyperfine(targets, args.warmup, args.runs, True, export)
                sections += ["", "2. VIA SHELL", "", table(export, binary), ""]

        write_results(sections, args.warmup, args.runs)

    print(f"\n{GREEN}{BOLD}Сохранено в {RESULTS}{NC}\n")
    print(RESULTS.read_text())


if __name__ == "__main__":
    sys.exit(main())
