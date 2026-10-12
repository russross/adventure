#!/usr/bin/env python3
"""Trace the official 6502 instruction set from the Adventure ROM vectors."""

import json
import sys
from dataclasses import dataclass
from enum import Enum
from pathlib import Path


class Mode(Enum):
    IMP = 1
    IMM = 2
    ZP = 3
    ZPX = 4
    ZPY = 5
    ABS = 6
    ABSX = 7
    ABSY = 8
    INDX = 9
    INDY = 10
    IND = 11
    REL = 12


# Each row lists official opcodes in the same addressing-mode order.
ALU_MODES = (Mode.IMM, Mode.ZP, Mode.ZPX, Mode.ABS, Mode.ABSX, Mode.ABSY, Mode.INDX, Mode.INDY)
OPCODES: dict[int, tuple[str, Mode]] = {}
for mnemonic, values in (
    ("adc", "69 65 75 6d 7d 79 61 71"),
    ("and", "29 25 35 2d 3d 39 21 31"),
    ("cmp", "c9 c5 d5 cd dd d9 c1 d1"),
    ("eor", "49 45 55 4d 5d 59 41 51"),
    ("lda", "a9 a5 b5 ad bd b9 a1 b1"),
    ("ora", "09 05 15 0d 1d 19 01 11"),
    ("sbc", "e9 e5 f5 ed fd f9 e1 f1"),
):
    for opcode, mode in zip(values.split(), ALU_MODES):
        OPCODES[int(opcode, 16)] = (mnemonic, mode)

# The remaining groups include shifts, transfers, branches, and stores.
for mnemonic, values, modes in (
    ("sta", "85 95 8d 9d 99 81 91", ALU_MODES[1:]),
    ("asl", "0a 06 16 0e 1e", (Mode.IMP, Mode.ZP, Mode.ZPX, Mode.ABS, Mode.ABSX)),
    ("lsr", "4a 46 56 4e 5e", (Mode.IMP, Mode.ZP, Mode.ZPX, Mode.ABS, Mode.ABSX)),
    ("rol", "2a 26 36 2e 3e", (Mode.IMP, Mode.ZP, Mode.ZPX, Mode.ABS, Mode.ABSX)),
    ("ror", "6a 66 76 6e 7e", (Mode.IMP, Mode.ZP, Mode.ZPX, Mode.ABS, Mode.ABSX)),
    ("inc", "e6 f6 ee fe", (Mode.ZP, Mode.ZPX, Mode.ABS, Mode.ABSX)),
    ("dec", "c6 d6 ce de", (Mode.ZP, Mode.ZPX, Mode.ABS, Mode.ABSX)),
    ("ldx", "a2 a6 b6 ae be", (Mode.IMM, Mode.ZP, Mode.ZPY, Mode.ABS, Mode.ABSY)),
    ("ldy", "a0 a4 b4 ac bc", (Mode.IMM, Mode.ZP, Mode.ZPX, Mode.ABS, Mode.ABSX)),
    ("stx", "86 96 8e", (Mode.ZP, Mode.ZPY, Mode.ABS)),
    ("sty", "84 94 8c", (Mode.ZP, Mode.ZPX, Mode.ABS)),
    ("bit", "24 2c", (Mode.ZP, Mode.ABS)),
    ("cpx", "e0 e4 ec", (Mode.IMM, Mode.ZP, Mode.ABS)),
    ("cpy", "c0 c4 cc", (Mode.IMM, Mode.ZP, Mode.ABS)),
    ("jmp", "4c 6c", (Mode.ABS, Mode.IND)),
    ("jsr", "20", (Mode.ABS,)),
):
    for opcode, mode in zip(values.split(), modes):
        OPCODES[int(opcode, 16)] = (mnemonic, mode)

for opcode, mnemonic in (
    (0x90, "bcc"),
    (0xB0, "bcs"),
    (0xF0, "beq"),
    (0x30, "bmi"),
    (0xD0, "bne"),
    (0x10, "bpl"),
    (0x50, "bvc"),
    (0x70, "bvs"),
):
    OPCODES[opcode] = (mnemonic, Mode.REL)
for opcode, mnemonic in (
    (0x00, "brk"),
    (0x18, "clc"),
    (0xD8, "cld"),
    (0x58, "cli"),
    (0xB8, "clv"),
    (0xCA, "dex"),
    (0x88, "dey"),
    (0xE8, "inx"),
    (0xC8, "iny"),
    (0xEA, "nop"),
    (0x48, "pha"),
    (0x08, "php"),
    (0x68, "pla"),
    (0x28, "plp"),
    (0x40, "rti"),
    (0x60, "rts"),
    (0x38, "sec"),
    (0xF8, "sed"),
    (0x78, "sei"),
    (0xAA, "tax"),
    (0xA8, "tay"),
    (0xBA, "tsx"),
    (0x8A, "txa"),
    (0x9A, "txs"),
    (0x98, "tya"),
):
    OPCODES[opcode] = (mnemonic, Mode.IMP)


@dataclass(frozen=True)
class Instruction:
    address: int
    mnemonic: str
    mode: Mode
    raw: bytes
    operand: int

    @property
    def next_address(self) -> int:
        return self.address + len(self.raw)

    @property
    def target(self) -> int:
        if self.mode == Mode.REL:
            offset = self.operand if self.operand < 128 else self.operand - 256
            return self.next_address + offset
        return self.operand


# The cartridge occupies the top 4 KiB in the logical 6502 address space.
BASE = 0xF000
ROOT = Path(__file__).resolve().parent.parent


def decode(rom: bytes, address: int) -> Instruction:
    if not BASE <= address < BASE + len(rom):
        raise ValueError(f"Control flow leaves ROM at ${address:04X}")
    offset = address - BASE
    opcode = rom[offset]
    if opcode not in OPCODES:
        raise ValueError(f"Unsupported opcode ${opcode:02X} at ${address:04X}")
    mnemonic, mode = OPCODES[opcode]
    size = 1 if mode == Mode.IMP else 3 if mode in (Mode.ABS, Mode.ABSX, Mode.ABSY, Mode.IND) else 2
    raw = rom[offset : offset + size]
    if len(raw) != size:
        raise ValueError(f"Truncated instruction at ${address:04X}")
    return Instruction(address, mnemonic, mode, raw, int.from_bytes(raw[1:], "little"))


def successors(instruction: Instruction) -> tuple[int, ...]:
    if instruction.mnemonic in ("rts", "rti"):
        return ()
    if instruction.mnemonic == "brk" or instruction.mode == Mode.IND:
        raise ValueError(f"Control transfer needs resolution at ${instruction.address:04X}")
    if instruction.mnemonic == "jmp":
        return (instruction.target,)
    if instruction.mnemonic == "jsr" or instruction.mode == Mode.REL:
        return (instruction.target, instruction.next_address)
    return (instruction.next_address,)


def trace(rom: bytes) -> dict[int, Instruction]:
    pending = [int.from_bytes(rom[offset : offset + 2], "little") for offset in (0xFFA, 0xFFC, 0xFFE)]
    instructions: dict[int, Instruction] = {}
    occupied: dict[int, int] = {}
    while pending:
        address = pending.pop()
        if address in instructions:
            continue
        instruction = decode(rom, address)
        for byte_address in range(address, instruction.next_address):
            if byte_address in occupied:
                raise ValueError(f"Overlapping code at ${byte_address:04X}")
            occupied[byte_address] = address
        instructions[address] = instruction
        pending.extend(successors(instruction))
    return dict(sorted(instructions.items()))


def operand_text(instruction: Instruction) -> str:
    mode, value = instruction.mode, instruction.operand
    if mode == Mode.IMP:
        return ""
    if mode == Mode.REL:
        return f"L_{instruction.target:04X}"
    if mode == Mode.IMM:
        return f"#${value:02X}"
    text = f"${value:02X}" if mode in (Mode.ZP, Mode.ZPX, Mode.ZPY, Mode.INDX, Mode.INDY) else f"${value:04X}"
    if instruction.mnemonic in ("jmp", "jsr"):
        text = f"L_{value:04X}"
    if mode in (Mode.ZPX, Mode.ABSX):
        return text + ",x"
    if mode in (Mode.ZPY, Mode.ABSY):
        return text + ",y"
    if mode == Mode.INDX:
        return "(" + text + ",x)"
    if mode == Mode.INDY:
        return "(" + text + "),y"
    return text


@dataclass(frozen=True)
class ContextTrace:
    max_depth: int
    deepest_calls: tuple[int, ...]
    returns: dict[int, set[int]]
    calls: dict[int, set[tuple[int, int]]]
    visited_count: int


def trace_contexts(instructions: dict[int, Instruction]) -> ContextTrace:
    # A context stores actual JSR sites, so RTS follows the matching continuation.
    pending: list[tuple[int, tuple[int, ...]]] = [(BASE, ())]
    visited: set[tuple[int, tuple[int, ...]]] = set()
    returns: dict[int, set[int]] = {}
    calls: dict[int, set[tuple[int, int]]] = {}
    deepest: tuple[int, ...] = ()
    while pending:
        address, stack = pending.pop()
        if (address, stack) in visited:
            continue
        visited.add((address, stack))
        instruction = instructions[address]
        if instruction.mnemonic == "jsr":
            caller = instructions[stack[-1]].target if stack else BASE
            calls.setdefault(caller, set()).add((address, instruction.target))
            new_stack = (*stack, address)
            if len(new_stack) > 5:
                raise ValueError(f"Call depth exceeds supplied bound at ${address:04X}")
            if len(new_stack) > len(deepest):
                deepest = new_stack
            pending.append((instruction.target, new_stack))
        elif instruction.mnemonic == "rts":
            if not stack:
                raise ValueError(f"RTS has no caller at ${address:04X}")
            destination = instructions[stack[-1]].next_address
            returns.setdefault(address, set()).add(destination)
            pending.append((destination, stack[:-1]))
        else:
            pending.extend((target, stack) for target in successors(instruction))
    if {address for address, _ in visited} != set(instructions):
        raise ValueError("Vector and context-sensitive reachability disagree")
    return ContextTrace(len(deepest), deepest, returns, calls, len(visited))


def basic_blocks(instructions: dict[int, Instruction]) -> dict[int, tuple[int, ...]]:
    # Calls terminate blocks; their continuations and callees both receive labels.
    leaders = {BASE}
    for instruction in instructions.values():
        if instruction.mnemonic in ("jmp", "jsr") or instruction.mode == Mode.REL:
            leaders.update(successors(instruction))
    previous_end = BASE
    for instruction in instructions.values():
        if instruction.address != previous_end:
            leaders.add(instruction.address)
        previous_end = instruction.next_address
    blocks: dict[int, tuple[int, ...]] = {}
    for start in sorted(leaders):
        members: list[int] = []
        address = start
        while address in instructions:
            instruction = instructions[address]
            members.append(address)
            if instruction.mnemonic in ("jmp", "jsr", "rts", "rti") or instruction.mode == Mode.REL:
                break
            address = instruction.next_address
            if address in leaders:
                break
        blocks[start] = tuple(members)
    return blocks


def byte_ranges(addresses: set[int]) -> list[tuple[int, int]]:
    # Inclusive ranges retain every byte, including gaps and the vector words.
    ranges: list[tuple[int, int]] = []
    for address in sorted(addresses):
        if ranges and ranges[-1][1] + 1 == address:
            ranges[-1] = (ranges[-1][0], address)
        else:
            ranges.append((address, address))
    return ranges


def report(rom: bytes, instructions: dict[int, Instruction], contexts: ContextTrace) -> dict[str, object]:
    code = {
        address
        for instruction in instructions.values()
        for address in range(instruction.address, instruction.next_address)
    }
    blocks = basic_blocks(instructions)
    return {
        "origin": "F000",
        "vector_targets": {
            f"{BASE + offset:04X}": f"{int.from_bytes(rom[offset : offset + 2], 'little'):04X}"
            for offset in (0xFFA, 0xFFC, 0xFFE)
        },
        "instruction_count": len(instructions),
        "code_bytes": len(code),
        "noncode_bytes": len(rom) - len(code),
        "code_ranges": [[f"{start:04X}", f"{end:04X}"] for start, end in byte_ranges(code)],
        "noncode_ranges": [
            [f"{start:04X}", f"{end:04X}"]
            for start, end in byte_ranges(set(range(BASE, BASE + len(rom))) - code)
        ],
        "basic_blocks": {
            f"{start:04X}": [f"{address:04X}" for address in members] for start, members in blocks.items()
        },
        "block_edges": {
            f"{start:04X}": [f"{address:04X}" for address in successors(instructions[members[-1]])]
            for start, members in blocks.items()
        },
        "call_graph": {
            f"{caller:04X}": [
                {"site": f"{site:04X}", "target": f"{target:04X}"} for site, target in sorted(edges)
            ]
            for caller, edges in sorted(contexts.calls.items())
        },
        "return_edges": {
            f"{site:04X}": [f"{target:04X}" for target in sorted(targets)]
            for site, targets in sorted(contexts.returns.items())
        },
        "max_call_depth": contexts.max_depth,
        "deepest_call_sites": [f"{site:04X}" for site in contexts.deepest_calls],
        "context_count": contexts.visited_count,
    }


def main() -> None:
    try:
        rom = (ROOT / "ADVNTURE.BIN").read_bytes()
        if len(rom) != 4096:
            raise ValueError("Expected a 4096-byte cartridge")
        instructions = trace(rom)
        contexts = trace_contexts(instructions)
        print(json.dumps(report(rom, instructions, contexts), indent=2))
    except (OSError, ValueError, KeyError) as error:
        print(f"Analysis failed: {error}", file=sys.stderr)
        raise SystemExit(1) from None


if __name__ == "__main__":
    main()
