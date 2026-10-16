"""Complete-picture layers and collision geometry independent of visible priority."""

from dataclasses import dataclass

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
    mask = ((1 << min(width, WIDTH)) - 1) << (x % WIDTH)
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
    player_width: int = 4


# Prepared layers contain screen coordinates and all physical rows, with no live world references.
@dataclass(frozen=True)
class Object:
    x: int = 0
    color: int = 0
    scale: int = 1
    rows: bytes = bytes(HEIGHT)


@dataclass(frozen=True)
class Picture:
    appearance: Appearance
    walls: tuple[int, ...]
    objects: tuple[Object, Object]
    player_x: int
    player_visible: tuple[bool, ...]
    visible_widths: tuple[int, ...] = (WIDTH,) * HEIGHT


@dataclass(frozen=True)
class Contacts:
    player_walls: bool = False
    player_barriers: tuple[bool, bool] = (False, False)
    player_objects: tuple[bool, bool] = (False, False)
    objects_touch: bool = False


class Video:
    def __init__(self) -> None:
        self.pixels = bytearray(WIDTH * HEIGHT)
        self.contacts = Contacts()
        self.retained: Picture | None = None
        self.retained_contacts = Contacts()

    # An identical picture retains hidden contacts as well as its complete raster.
    def paint(self, picture: Picture) -> None:
        if picture == self.retained:
            self.contacts = self.retained_contacts
            return
        appearance = picture.appearance
        player_mask = solid(picture.player_x, appearance.player_width)
        visibility = tuple((1 << width) - 1 for width in picture.visible_widths)
        players = tuple(
            player_mask & mask if visible else 0
            for visible, mask in zip(picture.player_visible, visibility)
        )
        walls = tuple(wall & mask for wall, mask in zip(picture.walls, visibility))
        foreground = tuple(wall | player for wall, player in zip(walls, players))
        self.pixels[:] = bytes([appearance.background]) * (WIDTH * HEIGHT)
        self.paint_layer(foreground, appearance.foreground)

        # Each object completes before the next; geometric contacts never depend on winning color.
        object_masks = tuple(
            tuple(sprite(obj.x, bits, obj.scale) & mask for bits, mask in zip(obj.rows, visibility))
            for obj in picture.objects
        )
        barriers = tuple(
            solid(x, 2) if enabled else 0 for x, enabled in zip((11, 148), appearance.barriers)
        )
        for slot in (1, 0):
            masks = tuple(
                mask | (barriers[slot] & visible)
                for mask, visible in zip(object_masks[slot], visibility)
            )
            if appearance.walls_in_front:
                masks = tuple(mask & ~front for mask, front in zip(masks, foreground))
            self.paint_layer(masks, picture.objects[slot].color)

        # Vertical blanking removes the rest of a partial row, including its background color.
        for row, width in enumerate(picture.visible_widths):
            self.pixels[row * WIDTH + width : (row + 1) * WIDTH] = bytes(WIDTH - width)

        self.contacts = Contacts(
            any(player & wall for player, wall in zip(players, walls)),
            (
                any(player & barriers[0] for player in players),
                any(player & barriers[1] for player in players),
            ),
            (
                any(player & mask for player, mask in zip(players, object_masks[0])),
                any(player & mask for player, mask in zip(players, object_masks[1])),
            ),
            any(first & second for first, second in zip(*object_masks)),
        )
        self.retained = picture
        self.retained_contacts = self.contacts

    # Rows with the same mask share their expanded pixel offsets within this layer.
    def paint_layer(self, rows: tuple[int, ...], color: int) -> None:
        offsets: dict[int, tuple[int, ...]] = {}
        for row, mask in enumerate(rows):
            if mask not in offsets:
                bits = mask
                positions = []
                while bits:
                    lowest = bits & -bits
                    positions.append(lowest.bit_length() - 1)
                    bits ^= lowest
                offsets[mask] = tuple(positions)
            start = row * WIDTH
            for x in offsets[mask]:
                self.pixels[start + x] = color
