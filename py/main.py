#!/usr/bin/env python3
"""Run the Python Adventure port."""

import sys
from types import TracebackType

import pygame

from game import Adventure
from host import Host

CONTROLS = """Adventure
Move: arrows / keypad 8,2,4,6   Drop: Space / Left Ctrl / keypad 5
Start/reset: Enter / F2        Select game: Tab / F1
Color/B&W: F3/F4               Difficulty left: F5/F6, right: F7/F8
Mute: M                       Window size: 1/2/3
Quit: Escape / Ctrl+Q / Cmd+Q
"""


def main() -> int:
    if len(sys.argv) > 1:
        if sys.argv[1:] in (["--help"], ["-h"]):
            print(CONTROLS)
            return 0
        print("Adventure: no command-line options; use --help for controls", file=sys.stderr)
        return 2
    host = None
    try:
        game = Adventure()
        host = Host(game)
        print(CONTROLS)
        controls = host.sample_controls()
        if controls is not None:
            game.run(host.present, controls)
        return 0
    except (pygame.error, OSError) as error:
        print(f"Adventure: {error}", file=sys.stderr)
        return 1
    finally:
        if host is not None:
            host.close()
        else:
            pygame.quit()


def report_unexpected(
    kind: type[BaseException], error: BaseException, traceback: TracebackType | None
) -> None:
    print(f"Adventure: unexpected {kind.__name__}: {error}", file=sys.stderr)


if __name__ == "__main__":
    sys.excepthook = report_unexpected
    sys.exit(main())
