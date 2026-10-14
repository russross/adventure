"""Adventure's world updates and three-picture turn scheduler."""

from collections.abc import Callable
from dataclasses import dataclass, field, replace

import artwork as art
from model import (
    CASTLES,
    OBJECT_RECORDS,
    Bat,
    Controls,
    Difficulty,
    Direction,
    DragonState,
    Edge,
    Game,
    GateState,
    ObjectId,
    Phase,
    Position,
    RecordId,
    Room,
    World,
    signed,
)
from rooms import ROOM_COLORS, WALLS, room_exit
from sound import Effect, Sound
from video import Appearance, ObjectLine, Video, wall_pixels


@dataclass
class Display:
    appearance: Appearance = field(default_factory=Appearance)
    walls: tuple[int, ...] = field(
        default_factory=lambda: tuple(wall_pixels(row, True) for row in art.WALLS_ENCLOSED_ROOM)
    )
    objects: tuple[ObjectLine, ObjectLine] = field(
        default_factory=lambda: (ObjectLine(), ObjectLine())
    )
    positions: tuple[Position, Position] = field(default_factory=lambda: (Position(), Position()))
    images: tuple[tuple[int, ...], tuple[int, ...]] = (art.EMPTY_IMAGE, art.EMPTY_IMAGE)
    player_visible: bool = False


class Adventure:
    def __init__(self) -> None:
        self.world = World()
        self.video = Video()
        self.sound = Sound()
        self.controls = Controls()
        self.display = Display()
        self.phase = Phase.Player

    # Presentation returns the inputs sampled at that boundary; None means quit.
    def run(self, present: Callable[[], Controls | None], controls: Controls) -> None:
        self.controls = controls
        if not self.initialize_world(present):
            return
        while self.advance_picture(present):
            pass

    def present_picture(self, present: Callable[[], Controls | None]) -> bool:
        controls = present()
        if controls is None:
            return False
        self.controls = controls
        return True

    # Each phase preserves which side of presentation owns an update.
    def advance_picture(self, present: Callable[[], Controls | None]) -> bool:
        world = self.world
        if self.phase == Phase.Player:
            if not self.poll_console(present):
                return False
            self.sound.advance(world.won)
            if self.sound.active == Effect.Victory:
                self.display.appearance = replace(
                    self.display.appearance, foreground=self.sound.active_remaining
                )
            self.track_inactivity()
            if world.won:
                if not self.present_picture(present):
                    return False
                self.draw_room()
                self.prepare_display()
                return True
            if world.positions[RecordId.Chalice].room == Room.GoldTreasury:
                world.won = True
                self.sound.start(Effect.Victory, 255)
            self.move_player(Phase.Player)
            self.move_held_object()
            if not self.present_picture(present):
                return False
            self.prepare_display()
            self.draw_room()
            self.pickup_and_drop()
            self.phase = Phase.BatAndGates
        elif self.phase == Phase.BatAndGates:
            self.move_player(Phase.BatAndGates)
            self.update_light()
            if not self.present_picture(present):
                return False
            self.update_bat()
            self.update_gates()
            self.draw_room()
            self.update_dragon(RecordId.Grundle)
            self.update_dragon(RecordId.Yorgle)
            self.phase = Phase.RemainingActors
        else:
            if not self.present_picture(present):
                return False
            self.move_player(Phase.RemainingActors)
            self.update_dragon(RecordId.Rhindle)
            self.update_magnet()
            self.draw_room()
            self.phase = Phase.Player
        return True

    # Reset revives the player and dragons; Select rebuilds the selected world.
    def poll_console(self, present: Callable[[], Controls | None]) -> bool:
        world = self.world
        if self.controls.reset and not world.previous_reset:
            if world.won:
                return self.initialize_world(present)
            world.player = Position(Room.GoldCastle, 80, 32)
            world.safe_player = world.player
            for dragon in world.dragons.values():
                dragon.state = DragonState.Wandering
                dragon.bite_timer = 0
            self.sound.remaining = 0
            world.held = ObjectId.Empty
        if self.controls.select and not world.previous_select:
            if world.player.room == Room.GameSelection:
                world.game = Game((world.game + 1) % 3)
            return self.initialize_world(present)
        self.remember_switches()
        return True

    def remember_switches(self) -> None:
        self.world.previous_reset = self.controls.reset
        self.world.previous_select = self.controls.select

    def initialize_world(self, present: Callable[[], Controls | None]) -> bool:
        world = self.world
        world.player = Position(Room.GameSelection, world.player.x, 0)
        world.safe_player = Position(Room.GameSelection, world.safe_player.x, 0)
        world.positions[RecordId.Dot] = Position(Room.BlackMazeDot, 81, 18)
        world.gates = {gate: int(GateState.Closed) for gate, _, _, _ in CASTLES}
        world.bat = Bat()
        if world.game == Game.One:
            placements = (
                (RecordId.Rhindle, Room.AboveWhiteCastle, 80, 32),
                (RecordId.Yorgle, Room.WestHall, 80, 32),
                (RecordId.Grundle, Room.RedHall, 80, 32),
                (RecordId.Magnet, Room.BlackCastleInterior, 128, 32),
                (RecordId.Sword, Room.GoldTreasury, 32, 32),
                (RecordId.Chalice, Room.PurpleChamber, 48, 32),
                (RecordId.Bridge, Room.BlueMazeNorth, 41, 55),
                (RecordId.GoldKey, Room.GoldCastle, 32, 64),
                (RecordId.WhiteKey, Room.AboveWhiteCastle, 32, 64),
                (RecordId.BlackKey, Room.RedHall, 32, 64),
                (RecordId.Bat, Room.WhiteCastleInterior, 32, 32),
            )
        else:
            placements = (
                (RecordId.Rhindle, Room.BlackMaze14, 80, 32),
                (RecordId.Yorgle, Room.RedMazeSouth, 80, 32),
                (RecordId.Grundle, Room.BlueMazeNorth, 80, 32),
                (RecordId.Magnet, Room.AboveWhiteCastle, 128, 32),
                (RecordId.Sword, Room.GoldCastle, 32, 32),
                (RecordId.Chalice, Room.BlackMaze14, 48, 32),
                (RecordId.Bridge, Room.CatacombsSide, 64, 64),
                (RecordId.GoldKey, Room.CatacombsCenter, 32, 64),
                (RecordId.WhiteKey, Room.BlueMazeSouth, 32, 64),
                (RecordId.BlackKey, Room.RedMazeSouth, 32, 64),
                (RecordId.Bat, Room.BelowGoldCastle, 32, 32),
            )
        for record, room, x, y in placements:
            world.positions[record] = Position(room, x, y)
        first_game = world.game == Game.One
        for dragon in world.dragons.values():
            dragon.state = DragonState.Wandering
            dragon.bite_timer = 0
            dragon.direction = Direction(first_game, True, True, first_game)
        world.bat.direction = Direction(first_game, True, first_game, True)
        if world.game == Game.Three:
            self.randomize_world()
            if not self.present_picture(present):
                return False
            self.draw_room()
        world.won = False
        world.held = ObjectId.Empty
        self.remember_switches()
        return True

    # Rejection sampling consumes the same seed sequence as the cartridge.
    def randomize_world(self) -> None:
        for record in (
            RecordId.Magnet,
            RecordId.Bat,
            RecordId.BlackKey,
            RecordId.WhiteKey,
            RecordId.GoldKey,
            RecordId.Bridge,
            RecordId.Sword,
            RecordId.Grundle,
            RecordId.Yorgle,
            RecordId.Rhindle,
            RecordId.Chalice,
        ):
            first, last = Room.WestHall, Room.RedHall
            if record == RecordId.Chalice:
                first, last = Room.BlackMaze13, Room.WhiteCastleInterior
            elif record == RecordId.BlackKey:
                last = Room.GoldTreasury
            elif record == RecordId.WhiteKey:
                last = Room.BlackMazeEntrance
            while True:
                seed = self.world.idle_ticks
                self.world.idle_ticks = ((seed >> 5) + seed + 1) & 255
                room = self.world.idle_ticks & 31
                if first <= room <= last:
                    self.world.positions[record] = replace(
                        self.world.positions[record], room=Room(room)
                    )
                    break

    def track_inactivity(self) -> None:
        world = self.world
        world.idle_ticks = (world.idle_ticks + 1) & 255
        if world.idle_ticks == 0:
            world.idle_periods = ((world.idle_periods + 1) & 255) or 128
        if (
            not self.controls.left.direction.is_idle()
            or not self.controls.right.direction.is_idle()
            or self.controls.reset
            or self.controls.select
        ):
            world.idle_periods = 0

    # Collisions roll back to the safe position before the next movement attempt.
    def move_player(self, phase: Phase) -> None:
        world, contacts = self.world, self.video.contacts
        blocked = (
            contacts.player_walls
            or contacts.player_barriers[0]
            or contacts.player_barriers[1]
            and world.second_visible != ObjectId.Dot
            or contacts.player_objects[0]
            and world.first_visible != ObjectId.Light
            or contacts.player_objects[1]
            and world.second_visible != ObjectId.Light
        )
        bridge = world.positions[RecordId.Bridge]
        bridge_x = (world.player.x - bridge.x) & 255
        bridge_y = (bridge.y - world.player.y) & 255
        crossing = (
            phase == Phase.RemainingActors
            and world.held != ObjectId.Bridge
            and world.player.room == bridge.room
            and 10 <= bridge_x < 23
            and not 25 <= bridge_y < 252
        )
        if not blocked or crossing:
            world.joystick_direction = Direction()
            world.safe_player = world.player
        if phase == Phase.Player:
            world.joystick_direction = self.controls.left.direction
        world.player = world.safe_player
        direction = world.joystick_direction
        if phase == Phase.BatAndGates:
            direction = direction.vertical()
        elif phase == Phase.RemainingActors:
            direction = direction.horizontal()
        self.move_object(RecordId.Player, direction, 3)

    def step_object(self, record: RecordId, direction: Direction, steps: int) -> None:
        position = self.world.positions[record]
        self.world.positions[record] = Position(
            position.room,
            (position.x + steps * (direction.right - direction.left)) & 255,
            (position.y + steps * (direction.up - direction.down)) & 255,
        )

    # Castle exits take priority over ordinary edges; all coordinates wrap as bytes.
    def move_object(self, record: RecordId, direction: Direction, steps: int) -> None:
        self.step_object(record, direction, steps)
        world = self.world
        position = world.positions[record]
        for gate, _, inside, outside in CASTLES:
            if (
                world.gates[gate] != GateState.Closed
                and position.room == inside
                and signed(position.y - 13) < 0
            ):
                world.positions[record] = Position(outside, 80, 44)
                world.gates[gate] = GateState.Open
                return
        room, x, y = position.room, position.x, position.y
        if signed(y - 106) >= 0:
            y, edge = 13, Edge.North
        elif x < 3 or x >= 240:
            x, edge = (158 if record == RecordId.Player else 154), Edge.West
        elif y < 13:
            y, edge = 105, Edge.South
        elif x >= (159 if record == RecordId.Player else 155):
            if (
                record == RecordId.Player
                and room == Room.SecretPassage
                and world.positions[RecordId.Dot].room != Room.BlackMazeDot
            ):
                world.positions[record] = Position(Room.RobinettCredits, 3, y)
                return
            x, edge = 3, Edge.East
        else:
            return
        world.positions[record] = Position(room_exit(room, edge, world.game), x, y)

    # Pickup uses the freshly drawn contacts and can immediately follow a drop.
    def pickup_and_drop(self) -> None:
        world = self.world
        newly_pressed = self.controls.left.fire and not world.fire_was_pressed
        world.fire_was_pressed = self.controls.left.fire
        if newly_pressed and world.held != ObjectId.Empty:
            world.held = ObjectId.Empty
            self.sound.start(Effect.Drop, 4)
        if self.video.contacts.player_objects[0]:
            touched = world.first_visible
        elif self.video.contacts.player_objects[1]:
            touched = world.second_visible
        else:
            return
        if touched <= ObjectId.Grundle or self.object_position(touched).room != world.player.room:
            return
        record = OBJECT_RECORDS.get(touched)
        if record is None:
            return
        if touched != world.held:
            self.sound.start(Effect.Pickup, 4)
        world.held = touched
        self.step_object(record, world.joystick_direction, 6)
        position = world.positions[record]
        world.held_offset_x = (position.x - world.player.x) & 255
        world.held_offset_y = (position.y - world.player.y) & 255

    def move_held_object(self) -> None:
        world = self.world
        if world.held == ObjectId.Empty:
            return
        record = OBJECT_RECORDS.get(world.held)
        if record is not None:
            world.positions[record] = Position(
                world.player.room,
                (world.player.x + world.held_offset_x) & 255,
                (world.player.y + world.held_offset_y) & 255,
            )
            self.move_object(record, Direction(), 0)

    def touching_player(self, obj: ObjectId) -> bool:
        if obj == self.world.first_visible:
            return self.video.contacts.player_objects[0]
        if obj == self.world.second_visible:
            return self.video.contacts.player_objects[1]
        return False

    def touching_object(self, obj: ObjectId) -> ObjectId:
        if self.video.contacts.objects_touch:
            if obj == self.world.first_visible:
                return self.world.second_visible
            if obj == self.world.second_visible:
                return self.world.first_visible
        return ObjectId.Empty

    # Key contact advances a gate before entry is resolved and animation advances.
    def update_gates(self) -> None:
        world = self.world
        for gate, key, inside, _ in CASTLES:
            touched = self.touching_object(gate)
            if touched == key:
                world.gates[gate] = (world.gates[gate] + 1) & 255
            if world.gates[gate] != GateState.Closed:
                record = None
                if self.touching_player(gate):
                    world.gates[gate] = GateState.Open
                    record = RecordId.Player
                elif touched != ObjectId.Empty:
                    record = OBJECT_RECORDS.get(touched)
                if record is not None:
                    world.positions[record] = replace(world.positions[record], room=inside, y=16)
            value = world.gates[gate]
            if value not in (GateState.Open, GateState.Closed):
                value = (value + 1) & 255
                world.gates[gate] = int(GateState.Open) if value == 56 else value

    def update_magnet(self) -> None:
        positions = self.world.positions
        magnet = positions[RecordId.Magnet]
        positions[RecordId.Magnet] = replace(magnet, y=(magnet.y - 8) & 255)
        for item in (
            RecordId.GoldKey,
            RecordId.WhiteKey,
            RecordId.BlackKey,
            RecordId.Sword,
            RecordId.Bridge,
            RecordId.Chalice,
        ):
            direction = self.direction_toward(item, RecordId.Magnet)
            if direction is not None:
                self.move_object(item, direction, 1)
                break
        positions[RecordId.Magnet] = magnet

    def update_light(self) -> None:
        world = self.world
        if not world.player.room.is_dark():
            world.positions[RecordId.Light] = replace(world.positions[RecordId.Light], y=0)
            return
        x = (world.player.x - 14) & 255
        x = 1 if x >= 240 else 129 if x >= 130 else x
        world.positions[RecordId.Light] = Position(
            world.player.room, x, (world.player.y + 14) & 255
        )

    # A target in another room is absent; a coincident target gives idle motion.
    def direction_toward(self, source: RecordId, target: RecordId) -> Direction | None:
        start, end = self.world.positions[source], self.world.positions[target]
        if start.room != end.room:
            return None
        return Direction(end.x > start.x, end.x < start.x, end.y > start.y, end.y < start.y)

    def dragon_direction(self, record: RecordId) -> Direction:
        if self.controls.right_difficulty == Difficulty.A:
            direction = self.direction_toward(RecordId.Sword, record)
            if direction is not None:
                return direction
        if record == RecordId.Yorgle:
            direction = self.direction_toward(RecordId.GoldKey, record)
            if direction is not None:
                return direction
        targets = {
            RecordId.Rhindle: (RecordId.Player, RecordId.Chalice, RecordId.WhiteKey),
            RecordId.Yorgle: (RecordId.Player, RecordId.Chalice),
            RecordId.Grundle: (
                RecordId.Player,
                RecordId.Chalice,
                RecordId.Bridge,
                RecordId.Magnet,
                RecordId.BlackKey,
            ),
        }
        for target in targets[record]:
            direction = self.direction_toward(record, target)
            if direction is not None:
                return direction
        return self.world.dragons[record].direction

    # Sword contact follows bite initiation, allowing a dragon to die in the same update.
    def update_dragon(self, record: RecordId) -> None:
        world = self.world
        dragon = world.dragons[record]
        obj = ObjectId[record.name]
        if dragon.state == DragonState.Wandering:
            dragon.direction = self.dragon_direction(record)
            self.move_object(record, dragon.direction, 3 if record == RecordId.Rhindle else 2)
            if self.touching_player(obj):
                first_game = world.game == Game.One
                easy = self.controls.left_difficulty == Difficulty.B
                dragon.bite_timer = (208 if easy else 232) if first_game else (240 if easy else 246)
                dragon.state = DragonState.Biting
                world.positions[record] = replace(
                    world.positions[record], x=world.safe_player.x, y=world.safe_player.y
                )
                self.sound.start(Effect.Roar, 16)
            if self.touching_object(obj) == ObjectId.Sword:
                dragon.state = DragonState.Dead
                dragon.bite_timer = 0
                self.sound.start(Effect.DragonSlain, 16)
        elif dragon.state == DragonState.Swallowed:
            position = world.positions[record]
            world.player = Position(position.room, (position.x + 3) & 255, (position.y - 10) & 255)
            world.safe_player = world.player
        elif dragon.state == DragonState.Biting:
            count = (dragon.bite_timer + 1) & 255
            dragon.bite_timer = count
            if count < 3:
                dragon.state = DragonState(count)
                dragon.bite_timer = 0
            if count < 252 or not self.touching_player(obj):
                return
            dragon.state = DragonState.Swallowed
            dragon.bite_timer = 0
            self.sound.start(Effect.Swallow, 16)
            position = world.positions[record]
            world.positions[record] = replace(
                position, x=min(position.x, 155), y=max(position.y, 23)
            )

    # The bat moves before checking its target and holds its heading while committed.
    def update_bat(self) -> None:
        world, bat = self.world, self.world.bat
        bat.animation = (bat.animation + 1) % 8
        if bat.flight_timer:
            bat.flight_timer = (bat.flight_timer + 1) & 255
            self.move_object(RecordId.Bat, bat.direction, 3)
        else:
            target = next(
                (
                    record
                    for record in (
                        RecordId.Chalice,
                        RecordId.Sword,
                        RecordId.Bridge,
                        RecordId.GoldKey,
                        RecordId.WhiteKey,
                        RecordId.BlackKey,
                        RecordId.Rhindle,
                        RecordId.Yorgle,
                        RecordId.Grundle,
                        RecordId.Magnet,
                    )
                    if bat.carried != RecordId.Bat
                    and bat.carried != record
                    and world.positions[record].room == world.positions[RecordId.Bat].room
                ),
                None,
            )
            if target is not None:
                direction = self.direction_toward(RecordId.Bat, target)
                assert direction is not None
                bat.direction = direction
            self.move_object(RecordId.Bat, bat.direction, 3)
            if target is not None:
                position, goal = world.positions[RecordId.Bat], world.positions[target]
                if (
                    goal.room == position.room
                    and (goal.x - position.x + 4) & 248 == 0
                    and (goal.y - position.y + 4) & 248 == 0
                ):
                    bat.carried = target
                    bat.flight_timer = 16
        if bat.carried is not None:
            position = world.positions[RecordId.Bat]
            world.positions[bat.carried] = Position(
                position.room, (position.x + 8) & 255, position.y
            )
            if OBJECT_RECORDS.get(world.held) == bat.carried:
                world.held = ObjectId.Empty

    # Visible identities rotate through inhabitants, preserving the original flicker.
    def select_objects(self) -> None:
        world = self.world
        world.first_visible = world.second_visible = ObjectId.Empty
        candidate = world.last_candidate
        while True:
            candidate = ObjectId((candidate + 1) % 18)
            if self.object_position(candidate).room == world.player.room:
                if world.first_visible == ObjectId.Empty:
                    world.first_visible = candidate
                else:
                    world.second_visible = candidate
                    break
            if candidate == world.last_candidate:
                break
        world.last_candidate = candidate
        if (
            world.first_visible == ObjectId.Light
            or world.first_visible == ObjectId.Bridge
            and world.second_visible != ObjectId.Light
        ):
            world.first_visible, world.second_visible = world.second_visible, world.first_visible

    def object_position(self, obj: ObjectId) -> Position:
        record = OBJECT_RECORDS.get(obj)
        if record is not None:
            return self.world.positions[record]
        return {
            ObjectId.GoldGate: Position(Room.GoldCastle, 77, 49),
            ObjectId.WhiteGate: Position(Room.WhiteCastle, 77, 49),
            ObjectId.BlackGate: Position(Room.BlackCastle, 77, 49),
            ObjectId.Credits: Position(Room.RobinettCredits, 80, 105),
            ObjectId.GameNumber: Position(Room.GameSelection, 80, 64),
        }[obj]

    def display_color(self, color: int | None) -> int:
        value = self.world.idle_ticks if color is None else color >> 1
        if self.world.idle_periods & 128:
            value = (value ^ self.world.idle_periods) & 251
        return (value << 1) & 255

    # Room and object appearances are captured once for all three pictures.
    def prepare_display(self) -> None:
        room = self.world.player.room
        color = ROOM_COLORS[room]
        if not room.is_dark() and not self.controls.color:
            color = 12 if room == Room.WhiteCastle else 2 if room == Room.BlackCastle else 10
        mirrored = room not in (Room.BlackMaze14, Room.BlackMazeDot)
        self.display.appearance = Appearance(
            self.display_color(8),
            self.display_color(color),
            mirrored,
            room.is_dark(),
            (
                room in (Room.WestHall, Room.SidePassage0d),
                room in (Room.SecretPassage, Room.SidePassage0c),
            ),
        )
        self.display.walls = tuple(wall_pixels(row, mirrored) for row in WALLS[room])
        self.select_objects()
        first, second = self.world.first_visible, self.world.second_visible
        self.display.positions = (self.object_position(first), self.object_position(second))
        self.display.images = (self.object_image(first), self.object_image(second))
        self.display.objects = (self.object_line(first), self.object_line(second))

    def object_line(self, obj: ObjectId) -> ObjectLine:
        wide = obj in (ObjectId.Light, ObjectId.Bridge)
        return ObjectLine(
            object_x(self.object_position(obj).x, wide),
            self.object_color(obj),
            4 if wide else 1,
        )

    def object_image(self, obj: ObjectId) -> tuple[int, ...]:
        if obj in (ObjectId.GoldGate, ObjectId.WhiteGate, ObjectId.BlackGate):
            stage = min(max(self.world.gates[obj] - 1, 0) // 4, 12)
            return art.GATE_IMAGE[abs(stage - 6) * 2 :]
        if obj == ObjectId.GameNumber:
            return (art.GAME_ONE_IMAGE, art.GAME_TWO_IMAGE, art.GAME_THREE_IMAGE)[self.world.game]
        if obj in (ObjectId.Rhindle, ObjectId.Yorgle, ObjectId.Grundle):
            state = self.world.dragons[RecordId[obj.name]].state
            if state == DragonState.Dead:
                return art.DRAGON_DEAD_IMAGE
            if state == DragonState.Biting:
                return art.DRAGON_BITING_IMAGE
            return art.DRAGON_WALKING_IMAGE
        if obj == ObjectId.Bat:
            return art.BAT_FRAME_ONE if self.world.bat.animation < 4 else art.BAT_FRAME_TWO
        return {
            ObjectId.Light: art.LIGHT_IMAGE,
            ObjectId.Credits: art.ROBINETT_CREDITS_IMAGE,
            ObjectId.Sword: art.SWORD_IMAGE,
            ObjectId.Bridge: art.BRIDGE_IMAGE,
            ObjectId.GoldKey: art.KEY_IMAGE,
            ObjectId.WhiteKey: art.KEY_IMAGE,
            ObjectId.BlackKey: art.KEY_IMAGE,
            ObjectId.Dot: art.DOT_IMAGE,
            ObjectId.Chalice: art.CHALICE_IMAGE,
            ObjectId.Magnet: art.MAGNET_IMAGE,
            ObjectId.Empty: art.EMPTY_IMAGE,
        }[obj]

    def object_color(self, obj: ObjectId) -> int:
        if self.controls.color:
            colors: dict[ObjectId, int | None] = {
                ObjectId.Light: 0x28,
                ObjectId.Credits: None,
                ObjectId.Chalice: None,
                ObjectId.GameNumber: 0xC8,
                ObjectId.Grundle: 0xC8,
                ObjectId.Rhindle: 0x36,
                ObjectId.Yorgle: 0x1A,
                ObjectId.Sword: 0x1A,
                ObjectId.GoldKey: 0x1A,
                ObjectId.Bridge: 0x66,
                ObjectId.WhiteKey: 0x0E,
                ObjectId.Dot: 0x08,
            }
        else:
            colors = {
                ObjectId.Light: 12,
                ObjectId.Rhindle: 14,
                ObjectId.WhiteKey: 14,
                ObjectId.Yorgle: 6,
                ObjectId.Sword: 6,
                ObjectId.GoldKey: 6,
                ObjectId.Chalice: 6,
                ObjectId.Magnet: 6,
                ObjectId.Bridge: 2,
                ObjectId.Dot: 8,
            }
        return self.display_color(colors.get(obj, 0))

    # Each logical row paints retained data, then commits the newly fetched sprite rows.
    def draw_room(self) -> None:
        display = self.display
        objects = tuple(
            replace(obj, x=object_x(position.x, obj.scale == 4), bits=0)
            for obj, position in zip(display.objects, display.positions)
        )
        first, second = objects
        walls = display.walls[0]
        player_x = (positioned_x(self.world.player.x) - 1) % 160
        player_y = (self.world.player.y - 4) & 255
        visible = display.player_visible
        cursors = [0, 0]
        staged_second = 0
        band = 1
        self.video.begin_picture()
        for row in range(104, 7, -1):
            if row != 104 and signed(row - display.positions[1].y) < 0:
                image = display.images[1]
                staged_second = image[cursors[1]] if cursors[1] < len(image) else 0
                cursors[1] += staged_second != 0
            bits = 0
            if signed(row - display.positions[0].y) < 0:
                image = display.images[0]
                bits = image[cursors[0]] if cursors[0] < len(image) else 0
                cursors[0] += bits != 0
            self.video.paint(display.appearance, walls, (first, second), player_x, visible)
            first = replace(first, bits=bits)
            second = replace(second, bits=staged_second)
            visible = (row - player_y) & 252 == 0
            if row & 15 == 0:
                walls = display.walls[band]
                band += 1
            self.video.paint(display.appearance, walls, (first, second), player_x, visible)
        display.player_visible = visible


def positioned_x(x: int) -> int:
    if x <= 164:
        return x - 1
    if x <= 239:
        return x % 15 - 4
    return x - 229


def object_x(x: int, wide: bool) -> int:
    return (positioned_x(x) + wide) % 160
