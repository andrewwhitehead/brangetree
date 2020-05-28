#!/usr/bin/env python3

import gzip
import os
import random

from bitstring import BitArray

# size in bits, percent filled
sets = [
    (16, 1),
    (16, 2),
    (16, 5),
    (16, 10),
    (16, 25),
    (16, 50),
    (20, 1),
    (20, 2),
    (20, 5),
    (20, 10),
    (20, 25),
    (20, 50),
    (22, 1),
    (22, 2),
    (22, 5),
    (22, 10),
    (22, 25),
    (22, 50),
    (23, 1),
    (23, 2),
    (23, 5),
    (23, 10),
    (23, 25),
    (23, 50),
    (24, 1),
    (24, 2),
    (24, 5),
    (24, 10),
    (24, 25),
    (24, 50),
]

if not os.path.isdir("data"):
    os.mkdir("data")

for (index_bits, fill_perc) in sets:
    size = pow(2, index_bits)

    fill_count = round(fill_perc * size / 100)
    inv_count = size - fill_count if fill_perc > 50 else fill_count

    # slower method
    # registry.invert(range(0, fill_count))
    # random.shuffle(registry)

    if fill_perc == 50:
        registry = BitArray(os.urandom(size // 8))
        filled = registry.count(1)
        if filled > fill_count:
            registry.invert()
            filled = size - filled
    else:
        filled = 0
        registry = BitArray(length=size)

    for _ in range(index_bits):
        registry.set(1, random.choices(range(0, size), k=(inv_count - filled)))
        filled = registry.count(1)
        if inv_count - filled < 10:
            break

    while filled < inv_count:
        pos = random.randrange(size)
        if not registry[pos]:
            registry.set(1, pos)
            filled += 1

    if fill_perc > 50:
        registry.invert()

    filename = f"data/{index_bits}bits_{fill_perc}pc_random.gz"
    with gzip.open(filename, "wb") as fp:
        registry.tofile(fp)
    print(filename)
