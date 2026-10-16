"""Pygame window, keyboard switches, sound playback, and frame pacing."""

import os
from dataclasses import replace

os.environ["PYGAME_HIDE_SUPPORT_PROMPT"] = "1"

import pygame

from game import Adventure
from model import Controls, Difficulty, Direction, Joystick
from palette import NTSC_PALETTE
from sound import SAMPLE_RATE, Effect, effect_samples
from video import HEIGHT, WIDTH


class Host:
    def __init__(self, game: Adventure) -> None:
        pygame.display.init()
        pygame.mixer.init(frequency=SAMPLE_RATE, size=-16, channels=1, buffer=512)
        self.game = game
        self.window = pygame.Window("Adventure", size=(960, 582), resizable=True)
        self.clock = pygame.time.Clock()
        self.controls = Controls()
        self.pressed: set[int] = set()
        self.muted = False
        self.channel = pygame.mixer.Channel(0)
        self.sound_revision = -1
        self.playing: tuple[Effect, int, bool] | None = None
        self.bank: dict[tuple[Effect, bool], tuple[bytes, dict[int, int]]] = {}
        self.image = pygame.Surface((WIDTH, HEIGHT), depth=8)
        self.image.set_palette(
            [
                ((rgb >> 16) & 255, (rgb >> 8) & 255, rgb & 255)
                for color in range(256)
                for rgb in (NTSC_PALETTE[color >> 1],)
            ]
        )
        self.indexed: bytes | None = None
        self.redraw = True
        self.scaled: pygame.Surface | None = None
        self.scaled_size = (0, 0)

    # Switches persist, while joysticks and momentary console buttons follow held keys.
    def sample_controls(self) -> Controls | None:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                return None
            if event.type in (
                pygame.WINDOWSHOWN,
                pygame.WINDOWEXPOSED,
                pygame.WINDOWRESIZED,
                pygame.WINDOWSIZECHANGED,
                pygame.WINDOWRESTORED,
                pygame.WINDOWMAXIMIZED,
                pygame.WINDOWDISPLAYCHANGED,
                pygame.RENDER_TARGETS_RESET,
                pygame.RENDER_DEVICE_RESET,
            ):
                self.redraw = True
            if event.type == pygame.WINDOWFOCUSLOST:
                self.pressed.clear()
            if event.type == pygame.KEYUP:
                self.pressed.discard(event.key)
            if event.type != pygame.KEYDOWN:
                continue
            if event.key == pygame.K_ESCAPE or (
                event.key == pygame.K_q and event.mod & (pygame.KMOD_CTRL | pygame.KMOD_GUI)
            ):
                return None
            repeated = event.key in self.pressed
            self.pressed.add(event.key)
            if event.key == pygame.K_m and not repeated:
                self.muted = not self.muted
                self.sound_revision = -1
            if event.key in (pygame.K_1, pygame.K_2, pygame.K_3):
                scale = event.key - pygame.K_1 + 1
                self.window.size = (320 * scale, HEIGHT * scale)
                self.redraw = True
            if event.key in (pygame.K_F3, pygame.K_F4):
                self.controls.color = event.key == pygame.K_F3
            if event.key in (pygame.K_F5, pygame.K_F6):
                self.controls.left_difficulty = (
                    Difficulty.A if event.key == pygame.K_F5 else Difficulty.B
                )
            if event.key in (pygame.K_F7, pygame.K_F8):
                self.controls.right_difficulty = (
                    Difficulty.A if event.key == pygame.K_F7 else Difficulty.B
                )
        keys = self.pressed
        self.controls.left = Joystick(
            Direction(
                bool(keys & {pygame.K_RIGHT, pygame.K_KP6}),
                bool(keys & {pygame.K_LEFT, pygame.K_KP4}),
                bool(keys & {pygame.K_UP, pygame.K_KP8}),
                bool(keys & {pygame.K_DOWN, pygame.K_KP2}),
            ),
            bool(keys & {pygame.K_SPACE, pygame.K_LCTRL, pygame.K_KP5}),
        )
        self.controls.right = Joystick(
            Direction(
                pygame.K_j in keys, pygame.K_g in keys, pygame.K_y in keys, pygame.K_h in keys
            ),
            pygame.K_f in keys,
        )
        self.controls.select = bool(keys & {pygame.K_F1, pygame.K_TAB})
        self.controls.reset = bool(keys & {pygame.K_F2, pygame.K_RETURN, pygame.K_KP_ENTER})
        return replace(self.controls)

    # Consecutive envelope steps continue the same clip; replacement effects seek their current step.
    def play_sound(self) -> None:
        sound = self.game.sound
        if sound.revision == self.sound_revision:
            return
        self.sound_revision = sound.revision
        if self.muted or sound.active is None:
            self.channel.fadeout(2)
            self.playing = None
            return
        current = (sound.active, sound.active_remaining, sound.every_picture)
        if self.playing != (sound.active, sound.active_remaining + 1, sound.every_picture):
            key = (sound.active, sound.every_picture)
            if key not in self.bank:
                self.bank[key] = effect_samples(*key)
            samples, offsets = self.bank[key]
            clip = pygame.mixer.Sound(buffer=samples[offsets[sound.active_remaining] :])
            self.channel.play(clip, fade_ms=2)
        self.playing = current

    def present(self) -> Controls | None:
        controls = self.sample_controls()
        if controls is None:
            return None
        self.play_sound()
        indexed = bytes(self.game.video.pixels)
        changed = indexed != self.indexed
        if changed:
            pixels = self.image.get_buffer()
            pixels.write(indexed)
            del pixels
            self.indexed = indexed
        width, height = self.window.size
        scale = min(width / 320, height / HEIGHT)
        size = (max(1, round(320 * scale)), max(1, round(HEIGHT * scale)))
        if changed or size != self.scaled_size:
            self.scaled = pygame.transform.scale(self.image, size)
            self.scaled_size = size
            self.redraw = True
        # Exposure still repaints cached pixels; idle boundaries keep input, sound, and pacing live.
        if self.redraw and self.scaled is not None:
            surface = self.window.get_surface()
            surface.fill("black")
            surface.blit(self.scaled, ((width - size[0]) // 2, (height - size[1]) // 2))
            self.window.flip()
            self.redraw = False
        self.clock.tick(60)
        return controls

    def close(self) -> None:
        pygame.quit()
