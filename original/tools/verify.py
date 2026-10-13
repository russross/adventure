#!/usr/bin/env python3
"""Verify reconstruction, graph labels, and the ROM's static scanline budgets."""

import re
import sys

from analyze import BASE, ROOT, Instruction, Mode, basic_blocks, trace, trace_contexts


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def cycle_cost(instruction: Instruction, target: int) -> int:
    # Branch costs depend on the selected edge and its page relationship.
    if instruction.mode == Mode.REL:
        if target == instruction.next_address:
            return 2
        return 3 + int(instruction.next_address >> 8 != target >> 8)
    if instruction.mnemonic == "jmp":
        return 3
    if instruction.mode in (Mode.IMP, Mode.IMM):
        return 2
    if instruction.mode == Mode.ZP:
        return 5 if instruction.mnemonic in ("inc", "dec") else 3
    if instruction.mode == Mode.INDY:
        return 5
    raise ValueError(f"Timing needs an explicit rule at ${instruction.address:04X}")


def timing_paths(instructions: dict[int, Instruction], start: int, finish: int) -> set[int]:
    # Explore both branch edges; the resulting bounds include infeasible flag combinations.
    pending = [(start, 0, frozenset())]
    results: set[int] = set()
    while pending:
        address, elapsed, visited = pending.pop()
        require(address not in visited, f"Timing path loops at ${address:04X}")
        instruction = instructions[address]
        if address == finish:
            results.add(elapsed + cycle_cost(instruction, instruction.next_address))
            continue
        if address != start and address in (0xF097, 0xF0BD, 0xF064, 0xF0B8):
            continue
        if instruction.mode == Mode.REL:
            targets = (instruction.target, instruction.next_address)
        elif instruction.mnemonic == "jmp":
            targets = (instruction.target,)
        else:
            targets = (instruction.next_address,)
        pending.extend(
            (target, elapsed + cycle_cost(instruction, target), visited | {address}) for target in targets
        )
    return results


def verify_graphics(rom: bytes, instructions: dict[int, Instruction]) -> int:
    # Descriptor graphics tables select streams by state ceiling, ending with $FF.
    occupied = {
        address
        for instruction in instructions.values()
        for address in range(instruction.address, instruction.next_address)
    }
    bitmaps: set[int] = set()

    def word(address: int) -> int:
        return int.from_bytes(rom[address - BASE : address - BASE + 2], "little")

    for descriptor in range(0xFF44, 0xFFEF, 9):
        table = word(descriptor + 4)
        while True:
            require(BASE <= table <= 0xFFFD, "Graphics table leaves ROM")
            require(not occupied.intersection(range(table, table + 3)), "Graphics table intersects code")
            bitmaps.add(word(table + 1))
            if rom[table - BASE] == 0xFF:
                break
            table += 3

    # The kernel advances a bitmap index only for a nonzero byte, then holds the zero.
    for bitmap in bitmaps:
        require(BASE <= bitmap <= 0xFFFF, "Bitmap pointer leaves ROM")
        terminator = BASE + rom.index(0, bitmap - BASE)
        require(bitmap >> 8 == terminator >> 8, f"Bitmap crosses page at ${bitmap:04X}")
        require(not occupied.intersection(range(bitmap, terminator + 1)), "Bitmap intersects code")

    # The initial triple and six refreshes consume offsets 0..20 in each room stream.
    for room in range(0xFE1B, 0xFF32, 9):
        playfield = word(room)
        require(BASE <= playfield <= 0xFFEB, "Playfield pointer leaves ROM")
        require(playfield >> 8 == (playfield + 20) >> 8, f"Playfield crosses page at ${playfield:04X}")
        require(not occupied.intersection(range(playfield, playfield + 21)), "Playfield intersects code")
    return len(bitmaps)


def verify_gameplay_edition(rom: bytes, instructions: dict[int, Instruction]) -> tuple[int, int]:
    # DASM marks named uses, including scoped locals, in its symbol output.
    rebuilt = (ROOT / "build/adventure-gameplay.bin").read_bytes()
    require(rom == rebuilt, "Gameplay edition differs from ADVNTURE.BIN")
    symbols = (ROOT / "build/adventure-gameplay.sym").read_text().splitlines()
    code_addresses: set[int] = set()
    code_labels = 0
    for line in symbols:
        match = re.fullmatch(r"\s*(\S+)\s+([0-9a-fA-F]{4})\s+(.*)", line)
        if match is None:
            continue
        name, value, attributes = match.groups()
        require("R" in attributes, f"Gameplay symbol {name} has no named use")
        address = int(value, 16)
        if address in instructions:
            code_addresses.add(address)
            code_labels += 1
    targets = {BASE} | {
        instruction.target
        for instruction in instructions.values()
        if instruction.mnemonic in ("jmp", "jsr") or instruction.mode == Mode.REL
    }
    require(code_addresses == targets, "Gameplay edition labels differ from named control targets")

    # The listing gives addresses for source rows without adding addresses to the edition itself.
    listing = (ROOT / "build/adventure-gameplay.lst").read_text().splitlines()
    source_addresses: dict[int, int] = {}
    for line in listing:
        match = re.match(r"^\s*(\d+)\s+([0-9a-fA-F]{4})\s", line)
        if match is not None:
            source_addresses[int(match[1])] = int(match[2], 16)
    layouts: dict[int, bool] = {}
    for offset in range(0xE1B, 0xF32, 9):
        address = int.from_bytes(rom[offset : offset + 2], "little")
        layouts.setdefault(address, bool(rom[offset + 4] & 1))

    # Check visual comments against the bytes, including the room's mirror/repeat choice.
    graphic_rows = 0
    source = (ROOT / "adventure-gameplay.asm").read_text().splitlines()
    for line_number, line in enumerate(source, start=1):
        match = re.fullmatch(r"\s*\.byte\s+([^;]+); \|([█ ]+)\|.*", line)
        if match is None:
            continue
        values_text, pixels = match.groups()
        values = [int(value.strip().removeprefix("$"), 16) for value in values_text.split(",")]
        if len(values) == 1:
            bits = [(values[0] >> bit) & 1 for bit in range(7, -1, -1)]
        elif len(values) == 3:
            address = source_addresses[line_number]
            layout = max(start for start in layouts if start <= address)
            bits = [(values[0] >> bit) & 1 for bit in range(4, 8)]
            bits.extend((values[1] >> bit) & 1 for bit in range(7, -1, -1))
            bits.extend((values[2] >> bit) & 1 for bit in range(8))
            bits.extend(list(reversed(bits)) if layouts[layout] else bits[:])
        else:
            raise ValueError(f"Unexpected graphic row at source line {line_number}")
        expected = "".join("█" if bit else " " for bit in bits)
        require(pixels == expected, f"Graphic comment differs from bytes at source line {line_number}")
        graphic_rows += 1
    require(graphic_rows > 0, "Gameplay edition contains no graphic previews")
    return code_labels, graphic_rows


def main() -> None:
    try:
        original = (ROOT.parent / "ADVNTURE.BIN").read_bytes()
        rebuilt = (ROOT / "build/adventure.bin").read_bytes()
        require(original == rebuilt, "Assembled ROM differs from ADVNTURE.BIN")
        instructions = trace(original)
        contexts = trace_contexts(instructions)
        source = (ROOT / "adventure.asm").read_text()
        labels = {int(address, 16) for address in re.findall(r"^L_([0-9A-F]{4}):", source, re.MULTILINE)}
        require(set(basic_blocks(instructions)).issubset(labels), "Assembly lacks a basic-block label")
        require(labels.issubset(instructions), "A code label is outside reachable instructions")
        bitmap_count = verify_graphics(original, instructions)
        gameplay_labels, graphic_rows = verify_gameplay_edition(original, instructions)

        # Check the actual instruction paths between synchronization boundaries.
        refresh_a = timing_paths(instructions, 0xF066, 0xF097)
        ordinary_a = timing_paths(instructions, 0xF066, 0xF0BD)
        require((min(refresh_a), max(refresh_a)) == (41, 75), "Refresh line-A timing changed")
        require((min(ordinary_a), max(ordinary_a)) == (42, 76), "Ordinary line-A timing changed")
        refresh_b = timing_paths(instructions, 0xF099, 0xF064)
        ordinary_b = timing_paths(instructions, 0xF0BF, 0xF064)
        require(refresh_b == {66} and ordinary_b == {33}, "Line-B timing changed")
        require(timing_paths(instructions, 0xF099, 0xF0B8) == {57}, "Refresh exit timing changed")
        require(timing_paths(instructions, 0xF0BF, 0xF0B8) == {24}, "Ordinary exit timing changed")
        print(
            f"Exact match: {len(original)} bytes; all {len(basic_blocks(instructions))} basic blocks labeled."
        )
        print(f"Matched RTS contexts: {contexts.visited_count}; maximum JSR depth: {contexts.max_depth}.")
        print(f"Graphics: {bitmap_count} bitmap starts and 31 room streams; no indexed page crossings.")
        print("Timing: line A 41-75 cycles (refresh), 42-76 (ordinary); line B 66/33 cycles.")
        print(
            f"Gameplay edition: exact match; {gameplay_labels} used code labels; {graphic_rows} visual rows."
        )
    except (OSError, ValueError, KeyError) as error:
        print(f"Verification failed: {error}", file=sys.stderr)
        raise SystemExit(1) from None


if __name__ == "__main__":
    main()
