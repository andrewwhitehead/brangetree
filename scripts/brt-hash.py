#!/usr/bin/env python3

import os
import sys
import time

sys.path.append(os.curdir)

from brangetree.hash import gen_leaves, hash_leaves
from brangetree.util import iter_zipped_bits, natural_sort

if __name__ == "__main__":
    argc = len(sys.argv)
    if argc < 2:
        print("Expected: input filename")
        raise SystemExit
    elif argc == 2:
        filename = sys.argv[1]
        if not os.path.isfile(filename):
            print("Not found:", filename)
            raise SystemExit
        fsize = os.path.getsize(filename)
        bits = iter_zipped_bits(filename)
        start = time.perf_counter()
        result = hash_leaves(gen_leaves(bits))
        print("zipped:", fsize)
        print("filled:", result["leaf_count_filled"])
        print("leaves:", result["leaf_count"])
        print("root:  ", result["root"].hex())
        print("time:   {:0.2f}".format(time.perf_counter() - start))
    else:
        paths = sys.argv[1:]
        natural_sort(paths)
        for filename in paths:
            if not os.path.isfile(filename):
                print("Not found:", filename)
                continue
            fsize = os.path.getsize(filename)
            bits = iter_zipped_bits(filename)
            start = time.perf_counter()
            result = hash_leaves(gen_leaves(bits))
            print(
                filename,
                fsize,
                result["leaf_count_filled"],
                result["leaf_count"],
                round(time.perf_counter() - start, 3),
            )
