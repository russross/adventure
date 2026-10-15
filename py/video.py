"""Scanline composition; contacts include obscured sprite geometry."""

from dataclasses import dataclass, field

WIDTH = 160
HEIGHT = 194


def wall_pixels(packed: tuple[int, int, int], mirrored: bool) -> int:
    edge, middle, inner = packed
    cells = (
        [bool(edge & (1 << (cell + 4))) for cell in range(4)]
        + [bool(middle & (1 << (7 - cell))) for cell in range(8)]
        + [bool(inner & (1 << cell)) for cell in range(8)]
    )
    cells += cells[::-1] if mirrored else cells[:]
    return sum(15 << (cell * 4) for cell, present in enumerate(cells) if present)


def solid(x: int, width: int) -> int:
    mask = ((1 << width) - 1) << (x % WIDTH)
    return (mask | (mask >> WIDTH)) & ((1 << WIDTH) - 1)


def sprite(x: int, bits: int, scale: int) -> int:
    return sum(solid(x + bit * scale, scale) for bit in range(8) if bits & (128 >> bit))


@dataclass(frozen=True)
class Appearance:
    background: int = 0
    foreground: int = 0
    mirrored: bool = True
    walls_in_front: bool = False
    barriers: tuple[bool, bool] = (False, False)


@dataclass(frozen=True)
class ObjectLine:
    x: int = 0
    color: int = 0
    scale: int = 1
    bits: int = 0


@dataclass
class Contacts:
    player_walls: bool = False
    player_barriers: list[bool] = field(default_factory=lambda: [False, False])
    player_objects: list[bool] = field(default_factory=lambda: [False, False])
    objects_touch: bool = False


class Video:
    def __init__(self) -> None:
        self.pixels = bytearray(WIDTH * HEIGHT)
        self.contacts = Contacts()
        self.row = 0

    def begin_picture(self) -> None:
        self.contacts = Contacts()
        self.row = 0

    # Bit masks describe the geometry directly, independent of layer priority.
    def paint(
        self,
        appearance: Appearance,
        walls: int,
        objects: tuple[ObjectLine, ObjectLine],
        player_x: int,
        player_visible: bool,
    ) -> None:
        player = solid(player_x, 4) if player_visible else 0
        first, second = (sprite(obj.x, obj.bits, obj.scale) for obj in objects)
        left = solid(11, 2) if appearance.barriers[0] else 0
        right = solid(148, 2) if appearance.barriers[1] else 0
        contacts = self.contacts
        contacts.player_walls |= bool(player & walls)
        contacts.player_barriers[0] |= bool(player & left)
        contacts.player_barriers[1] |= bool(player & right)
        contacts.player_objects[0] |= bool(player & first)
        contacts.player_objects[1] |= bool(player & second)
        contacts.objects_touch |= bool(first & second)

        # Paint from the back forward while keeping hidden contacts intact.
        layers = [(walls | player, appearance.foreground)]
        layers += [(second | right, objects[1].color), (first | left, objects[0].color)]
        if appearance.walls_in_front:
            layers.append((walls | player, appearance.foreground))
        start = self.row * WIDTH
        pixels = bytearray([appearance.background]) * WIDTH
        for mask, color in layers:
            while mask:
                lowest = mask & -mask
                pixels[lowest.bit_length() - 1] = color
                mask ^= lowest
        self.pixels[start : start + WIDTH] = pixels
        self.row += 1
