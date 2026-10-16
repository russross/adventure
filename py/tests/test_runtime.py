"""Compare independent runtime behavior with the frame development target."""

import subprocess
import unittest
from dataclasses import replace
from pathlib import Path

from game import Adventure, sprite_rows
from model import Controls, Difficulty, Direction, Joystick, ObjectId, RecordId
from sound import voice
from video import HEIGHT, WIDTH, Appearance, Contacts, Object, Picture, Video, solid, wall_pixels

PICTURES = 420


def controls(game: int, picture: int) -> Controls:
    if picture < 105:
        direction = Direction(left=18 <= picture < 66, up=69 <= picture < 102)
    else:
        phase = picture // 17 % 4
        direction = Direction(phase == 0, phase == 2, phase == 1, phase == 3)
    return Controls(
        left=Joystick(direction, picture % 31 < 3),
        select=game >= 2 and picture == 0 or game == 3 and picture == 6,
        reset=picture in (12, 300),
        color=picture // 90 % 2 == 0,
        left_difficulty=Difficulty.B if picture < 180 else Difficulty.A,
        right_difficulty=Difficulty.B if picture < 210 else Difficulty.A,
    )


def world_bytes(game: Adventure) -> bytes:
    world = game.world
    positions = [world.positions[record] for record in RecordId] + [world.safe_player]
    result = bytearray(
        value for position in positions for value in (position.room, position.x, position.y)
    )
    result.extend(
        (
            world.game,
            world.won,
            255 if world.held is None else world.held,
            world.held_offset_x,
            world.held_offset_y,
            world.idle_ticks,
            world.idle_periods,
        )
    )
    result.extend(
        world.gates[gate] for gate in (ObjectId.GoldGate, ObjectId.WhiteGate, ObjectId.BlackGate)
    )
    return bytes(result)


class RuntimeTests(unittest.TestCase):
    # Full indexed pictures catch palette, scheduling, collision, and actor divergence together.
    def test_all_games_match_frame_pictures_audio_and_final_positions(self) -> None:
        executable = Path(__file__).resolve().parents[2] / "target/debug/examples/python_reference"
        reference = memoryview(subprocess.run([executable], check=True, capture_output=True).stdout)
        cursor = 0
        for number in range(1, 4):
            game = Adventure()
            pictures = 0

            def present(game: Adventure = game, number: int = number) -> Controls | None:
                nonlocal cursor, pictures
                if pictures == PICTURES:
                    return None
                expected = reference[cursor : cursor + WIDTH * HEIGHT]
                self.assertTrue(
                    game.video.pixels == expected,
                    f"game {number}, picture {pictures}: indexed raster differs",
                )
                cursor += WIDTH * HEIGHT
                channel = (
                    (0, 0, 0)
                    if game.sound.active is None
                    else voice(game.sound.active, game.sound.active_remaining)
                )
                self.assertEqual(
                    bytes((*channel, 0, 0, 0)),
                    reference[cursor : cursor + 6],
                    f"game {number}, picture {pictures}: sound controls differ",
                )
                cursor += 6
                current = controls(number, pictures)
                pictures += 1
                return current

            game.run(present, Controls())
            self.assertEqual(pictures, PICTURES)
            final = world_bytes(game)
            self.assertEqual(
                final,
                reference[cursor : cursor + len(final)],
                f"game {number}: final world differs",
            )
            cursor += len(final)
        self.assertEqual(cursor, len(reference))

    # A frozen snapshot survives live record edits, while the player remains picture-specific.
    def test_snapshot_and_retained_hidden_contacts(self) -> None:
        video = Video()
        picture = Picture(
            Appearance(background=28, foreground=28, walls_in_front=True),
            (solid(0, WIDTH),) * HEIGHT,
            (
                Object(color=28, rows=bytes([255]) * HEIGHT),
                Object(color=28, rows=bytes([255]) * HEIGHT),
            ),
            0,
            (True,) * HEIGHT,
        )
        video.paint(picture)
        expected = video.contacts
        self.assertTrue(expected.objects_touch and expected.player_walls)
        pixels = video.pixels[:]
        video.contacts = Contacts()
        video.paint(picture)
        self.assertEqual(video.contacts, expected)
        picture = replace(picture, objects=(Object(), picture.objects[1]))
        video.paint(picture)
        self.assertEqual(video.pixels, pixels)
        self.assertFalse(video.contacts.objects_touch)
        self.assertFalse(video.contacts.player_objects[0])

    def test_sprite_expansion_preserves_slot_delay_and_wrapping(self) -> None:
        image = (128, 64, 32, 0)
        self.assertEqual(
            sprite_rows(image, 105, 0), bytes((0, 128, 128, 64, 64, 32, 32)) + bytes(187)
        )
        self.assertEqual(
            sprite_rows(image, 105, 1), bytes((0, 0, 0, 128, 128, 64, 64, 32, 32)) + bytes(185)
        )
        self.assertEqual(sprite_rows(image, 0, 0), bytes(HEIGHT))
        self.assertEqual(sprite_rows(image, 0, 1), bytes(HEIGHT))

        # The byte comparison becomes ineligible partway through this long sprite.
        # Slot one clears the fetched row; slot two keeps the preceding nonzero bits.
        long_image = (128,) * 200 + (0,)
        self.assertEqual(sprite_rows(long_image, 200, 0), bytes(1) + bytes([128]) * 66 + bytes(127))
        self.assertEqual(sprite_rows(long_image, 200, 1), bytes(3) + bytes([128]) * 191)

    # A pixel-by-pixel oracle uses footprints directly rather than the compositor's bit masks.
    def test_complete_layers_match_pixel_geometry(self) -> None:
        for index in range(48):
            appearance = Appearance(
                8, 28, bool(index & 1), bool(index & 2), (bool(index & 4), bool(index & 8))
            )
            walls = wall_pixels((0x90, 0x81, 0x42), appearance.mirrored)
            objects = (
                Object((159 + index) % 256, 72, 4 if index & 16 else 1, bytes([0x81]) * HEIGHT),
                Object(0, 94, 4 if index & 32 else 1, bytes([0xC3]) * HEIGHT),
            )
            picture = Picture(appearance, (walls,) * HEIGHT, objects, 159, (True,) * HEIGHT)
            video = Video()
            video.paint(picture)
            expected = bytearray()
            for x in range(WIDTH):
                wall = bool(walls & (1 << x))
                player = (x - 159) % WIDTH < 4
                covered = []
                for obj in objects:
                    offset = (x - obj.x) % WIDTH
                    covered.append(
                        offset < 8 * obj.scale
                        and bool(obj.rows[0] & (128 >> (offset // obj.scale)))
                    )
                color = appearance.foreground if wall or player else appearance.background
                for slot in (1, 0):
                    barrier = appearance.barriers[slot] and (x - (11, 148)[slot]) % WIDTH < 2
                    if (covered[slot] or barrier) and not (
                        appearance.walls_in_front and (wall or player)
                    ):
                        color = objects[slot].color
                expected.append(color)
            self.assertEqual(video.pixels, expected * HEIGHT, index)


if __name__ == "__main__":
    unittest.main()
