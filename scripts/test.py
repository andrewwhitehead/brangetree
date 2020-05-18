#!/usr/bin/env python3

import json
import os
import sys
import time

from bitstring import Bits

sys.path.append(os.curdir)

from brangetree.hash import gen_leaves, hash_leaves
from brangetree.util import iter_bits


def test_leaf_hash(a, b):
    return f"{a},{b}"


def test_branch_hash(a, b):
    return [a, b]


if __name__ == "__main__":
    bits = list(iter_bits(b"\xfe\xff\xff\xff\xff\xff\xff\xff"))
    print(len(bits))
    print(bits)
    print(list(Bits(b"\xfe\xff\xff\xff\xff\xff\xff\xff")))

    # size = int(sys.argv[1]) if len(sys.argv) > 1 else 16
    # registry = Bits(os.urandom(size // 8))
    # leaves = list(gen_leaves(registry))
    size = int(sys.argv[1]) if len(sys.argv) > 1 else 12
    leaves = [(i, i) for i in range(size)]  # range(23)]
    print(leaves)
    result = hash_leaves(leaves, test_leaf_hash, test_branch_hash)

    start = time.perf_counter()
    # print("size:  ", len(registry))
    print("leaves:", result["leaf_count"])
    print("root:  ", result["root"])
    # print("time:   {:0.2f}".format(time.perf_counter() - start))
    print(json.dumps(result["root"], indent=2))
