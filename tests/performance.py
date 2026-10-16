#!/usr/bin/env python3
"""Measure sequential release workloads and macOS audio/display service CPU."""

import hashlib
import json
import platform
import resource
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "target" / "performance.jsonl"
PORTS = ("adventure",)


def metadata(output: Path = OUTPUT) -> None:
    """Record the machine, toolchain, trace, and source snapshot beside timings."""
    commands = {
        "cpu": ["sysctl", "-n", "machdep.cpu.brand_string"],
        "rustc": ["rustc", "--version"],
        "sdl2": ["pkg-config", "--modversion", "sdl2"],
        "sdl2_pc_directory": ["pkg-config", "--variable=pcfiledir", "sdl2"],
        "linkage": [
            "otool",
            "-L",
            str(ROOT / "target/release/examples/adventure_performance"),
        ],
    }
    versions = {
        name: subprocess.run(
            command, capture_output=True, text=True, check=True
        ).stdout.strip()
        for name, command in commands.items()
    }
    sdl3 = subprocess.run(
        ["pkg-config", "--modversion", "sdl3"],
        capture_output=True,
        text=True,
        check=False,
    )
    if sdl3.returncode == 0:
        versions["sdl3"] = sdl3.stdout.strip()
    sources: dict[str, str] = {}
    paths = [
        ROOT / "Cargo.toml",
        ROOT / "Cargo.lock",
        ROOT / "tests/easy.trace",
        Path(__file__).resolve(),
    ]
    paths.extend(ROOT.glob("tests/performance.rs"))
    paths.extend((ROOT / "src").rglob("*.rs"))
    paths.extend((ROOT / "examples").rglob("*.rs"))
    paths.extend((ROOT / "src").rglob("Cargo.toml"))
    for path in sorted(paths):
        sources[str(path.relative_to(ROOT))] = hashlib.sha256(
            path.read_bytes()
        ).hexdigest()
    details = {"platform": platform.platform(), "versions": versions, "sha256": sources}
    output.with_suffix(".environment.json").write_text(
        json.dumps(details, indent=2) + "\n"
    )


def services() -> dict[str, tuple[int, float]]:
    """Read cumulative service CPU; retain PIDs to detect service restarts."""
    result = subprocess.run(
        ["ps", "-axww", "-o", "pid=,time=,comm="],
        capture_output=True,
        text=True,
        check=True,
    )
    values: dict[str, tuple[int, float]] = {}
    for line in result.stdout.splitlines():
        pid, cpu, command = line.split(maxsplit=2)
        name = Path(command).name
        if name in ("coreaudiod", "WindowServer"):
            minutes, seconds = cpu.split(":")
            values[name] = (int(pid), int(minutes) * 60 + float(seconds))
    if set(values) != {"coreaudiod", "WindowServer"}:
        raise RuntimeError("cannot find macOS audio/display services")
    return values


def measure(
    port: str, mode: str, repeats: int, trial: int, output: Path = OUTPUT
) -> None:
    """Use child resource usage, so sleeps and service CPU stay separate."""
    before_services = services()
    before_cpu = resource.getrusage(resource.RUSAGE_CHILDREN)
    started = time.monotonic()
    if mode == "baseline":
        time.sleep(30)
        stdout = ""
    else:
        result = subprocess.run(
            [
                str(ROOT / "target/release/examples" / f"{port}_performance"),
                mode,
                str(repeats),
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=True,
        )
        stdout = result.stdout.strip()
    wall = time.monotonic() - started
    after_cpu = resource.getrusage(resource.RUSAGE_CHILDREN)
    after_services = services()
    service_cpu: dict[str, float] = {}
    for name, (pid, cpu) in before_services.items():
        next_pid, next_cpu = after_services[name]
        if pid != next_pid:
            raise RuntimeError(f"{name} restarted during measurement")
        service_cpu[name] = round(next_cpu - cpu, 6)

    # Persist every completed measurement, including both raw CPU components.
    record = {
        "port": port,
        "mode": mode,
        "trial": trial,
        "repeats": repeats,
        "wall_seconds": wall,
        "user_seconds": after_cpu.ru_utime - before_cpu.ru_utime,
        "system_seconds": after_cpu.ru_stime - before_cpu.ru_stime,
        "service_cpu_seconds": service_cpu,
        "stdout": stdout,
    }
    with output.open("a") as stream:
        stream.write(json.dumps(record) + "\n")
    print(json.dumps(record), flush=True)


def main() -> None:
    if sys.platform != "darwin":
        raise RuntimeError("service measurements require macOS")
    if sys.argv[1:] == ["--audio-comparison"]:
        compare_audio()
        return
    if sys.argv[1:]:
        raise RuntimeError("usage: performance.py [--audio-comparison]")
    if OUTPUT.exists():
        raise RuntimeError(f"move or remove previous results before running: {OUTPUT}")
    metadata()

    # Warm each executable before collecting repeated unpaced measurements.
    for port in PORTS:
        subprocess.run(
            [str(ROOT / "target/release/examples" / f"{port}_performance"), "headless"],
            cwd=ROOT,
            check=True,
        )
    for trial in range(1, 6):
        for mode in ("headless", "synthesis"):
            for port in PORTS:
                measure(port, mode, 10, trial)

    # Paced workloads exercise the real device callback over the entire trace duration.
    measure("system", "baseline", 0, 1)
    for port in PORTS:
        for mode in ("idle-sdl", "sdl"):
            measure(port, mode, 1, 1)
    measure("system", "baseline", 0, 2)


def compare_audio() -> None:
    output = ROOT / "target/audio-performance.jsonl"
    if output.exists():
        raise RuntimeError(f"move or remove previous results before running: {output}")
    metadata(output)
    subprocess.run(
        [str(ROOT / "target/release/examples/adventure_performance"), "headless"],
        check=True,
    )
    for trial in range(1, 6):
        for mode in ("headless", "synthesis", "cached"):
            measure("adventure", mode, 10, trial, output)

    # Reverse paired trial order to limit a persistent ordering bias between modes.
    measure("system", "baseline", 0, 1, output)
    for trial, modes in enumerate((("sdl", "sdl-cached"), ("sdl-cached", "sdl")), 1):
        for mode in modes:
            measure("adventure", mode, 1, trial, output)
    for mode in ("idle-sdl", "idle-sdl-cached"):
        measure("adventure", mode, 1, 1, output)
    measure("system", "baseline", 0, 2, output)


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"Performance: {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            print(error.stderr.strip(), file=sys.stderr)
        sys.exit(1)
