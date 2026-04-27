#!/usr/bin/env python3
"""
Strips the contractspecv0 custom section from a Soroban WASM binary.

The contractspecv0 section is metadata used only by CLI tooling (e.g.,
`stellar contract invoke` typed args, `stellar contract info`). It is NOT
used at runtime — the contract executes identically without it.

This reduces binary size significantly for mainnet deployment where the
upload fee is proportional to write bytes.

Usage:
    python3 scripts/strip_spec.py <input.wasm> <output.wasm>
"""

import sys


SECTIONS_TO_STRIP = {"contractspecv0"}


def strip_custom_sections(input_path: str, output_path: str, strip: set) -> None:
    with open(input_path, "rb") as f:
        data = bytearray(f.read())

    magic_version = data[:8]
    if magic_version[:4] != b"\x00asm":
        raise ValueError(f"{input_path} is not a valid WASM file")

    result = bytearray(magic_version)
    i = 8
    stripped_bytes = 0

    while i < len(data):
        section_start = i
        section_id = data[i]
        i += 1

        # Read LEB128-encoded section size
        size = 0
        shift = 0
        while True:
            b = data[i]
            i += 1
            size |= (b & 0x7f) << shift
            shift += 7
            if not (b & 0x80):
                break

        section_end = i + size

        if section_id == 0:
            # Custom section: read the name
            name_len = 0
            shift2 = 0
            j = i
            while True:
                b = data[j]
                j += 1
                name_len |= (b & 0x7f) << shift2
                shift2 += 7
                if not (b & 0x80):
                    break
            name = data[j : j + name_len].decode("utf-8", errors="replace")

            if name in strip:
                section_bytes = section_end - section_start
                print(f"  Stripping '{name}': {section_bytes} bytes saved")
                stripped_bytes += section_bytes
                i = section_end
                continue

        result.extend(data[section_start:section_end])
        i = section_end

    with open(output_path, "wb") as f:
        f.write(result)

    original = len(data)
    stripped = len(result)
    print(f"  {input_path}: {original} bytes -> {stripped} bytes "
          f"({stripped_bytes} bytes / {stripped_bytes / original * 100:.1f}% removed)")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <input.wasm> <output.wasm>")
        sys.exit(1)

    strip_custom_sections(sys.argv[1], sys.argv[2], SECTIONS_TO_STRIP)
