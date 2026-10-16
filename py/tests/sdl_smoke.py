"""Exercise the real pygame window, keyboard, cached display, and audio host."""

from pathlib import Path

import pygame

from game import Adventure
from host import Host
from model import Controls, ObjectId, Room


def run() -> None:
    game = Adventure()
    host = Host(game)
    picture = 0
    try:
        assert host.window.size == (960, 582)
        initial = host.sample_controls()
        assert initial is not None
        # Cached pictures still repaint on exposure and retained pictures still poll input.
        assert host.present() is not None
        scaled = host.scaled
        assert host.present() is not None
        assert host.scaled is scaled
        pygame.event.post(pygame.event.Event(pygame.WINDOWEXPOSED))
        host.window.get_surface().fill("magenta")
        assert host.present() is not None
        assert host.window.get_surface().get_at((0, 0))[:3] == (0, 0, 0)

        def present() -> Controls | None:
            nonlocal picture
            if picture == 105:
                return None
            transitions = {
                0: (pygame.K_RETURN, True),
                3: (pygame.K_RETURN, False),
                6: (pygame.K_LEFT, True),
                9: (pygame.K_m, True),
                10: (pygame.K_m, True),
                12: (pygame.K_m, False),
                15: (pygame.K_m, True),
                16: (pygame.K_m, False),
                18: (pygame.K_1, True),
                19: (pygame.K_1, False),
                21: (pygame.K_2, True),
                22: (pygame.K_2, False),
                24: (pygame.K_3, True),
                25: (pygame.K_3, False),
                27: (pygame.K_2, True),
                28: (pygame.K_2, False),
                54: (pygame.K_LEFT, False),
                57: (pygame.K_UP, True),
                90: (pygame.K_UP, False),
            }
            if picture in transitions:
                key, pressed = transitions[picture]
                pygame.event.post(
                    pygame.event.Event(pygame.KEYDOWN if pressed else pygame.KEYUP, key=key, mod=0)
                )
            result = host.present()
            assert host.muted == (9 <= picture < 15)
            size = (
                (320, 194)
                if 18 <= picture <= 20
                else (960, 582)
                if picture < 18 or 24 <= picture <= 26
                else (640, 388)
            )
            assert host.window.size == size
            picture += 1
            return result

        game.run(present, initial)
        assert game.world.player.room == Room.GoldCastle
        assert game.world.held == ObjectId.GoldKey
        assert (game.sound.selected, False) in host.bank
        destination = Path(__file__).resolve().parents[2] / "target/py-smoke.bmp"
        destination.parent.mkdir(exist_ok=True)
        pygame.image.save(host.window.get_surface(), destination)
        pygame.event.post(pygame.event.Event(pygame.QUIT))
        assert host.sample_controls() is None
        print(
            "Python SDL: Gold Key pickup, cached redraw/exposure, mute/repeat, resizing, and quit passed."
        )
    finally:
        host.close()


if __name__ == "__main__":
    run()
