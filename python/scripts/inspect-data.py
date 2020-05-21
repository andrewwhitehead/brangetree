#!/usr/bin/env python3

import os
import sys

from bitstring import Bits

sys.path.append(os.curdir)

from brangetree.util import iter_zipped_blocks, natural_sort

if len(sys.argv) < 2:
    print("Expected one or more filenames")
    raise SystemExit

paths = sys.argv[1:]
natural_sort(paths)

for filename in paths:
    if not os.path.isfile(filename):
        print("Not found:", filename)
        continue
    fsize = os.path.getsize(filename)

    size = 0
    filled = 0
    for block in iter_zipped_blocks(filename):
        size += len(block) * 8
        filled += Bits(block).count(1)

    print(filename, fsize, size, filled, round(filled / size * 100))
