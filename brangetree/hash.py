import hashlib
import math


class Marker:
    def __init__(self, name):
        self.name = name

    def __repr__(self):
        return self.name

    def to_bytes(self, *args):
        return self.name.encode("ascii")


B = Marker("B")
E = Marker("E")


def gen_leaves(bits, fill=False):
    left = B
    rev_start = None
    leaf_idx = 0
    for idx, revoked in enumerate(bits):
        if revoked:
            if rev_start is None:
                yield (left, idx)
                leaf_idx += 1
                rev_start = idx
            left = idx
        else:
            rev_start = None
    yield (left, E)
    leaf_idx += 1

    if fill:
        fill_count = pow(2, math.ceil(math.log(leaf_idx, 2))) - leaf_idx
        for _ in range(fill_count):
            yield (E, E)
            leaf_idx += 1
    # print("fill count:", fill_count)


def branch_hash(a, b):
    h = hashlib.sha256(b"1")
    h.update(a)
    h.update(b)
    return h.digest()


def leaf_hash(a, b):
    h = hashlib.sha256(b"0")
    h.update(a.to_bytes(8, "little"))
    h.update(b.to_bytes(8, "little"))
    return h.digest()


def terminator_hash(cache, depth=0, leaf_hash=leaf_hash, branch_hash=branch_hash):
    if len(cache) > depth:
        h = cache[depth]
    elif depth > 0:
        h = terminator_hash(cache, depth - 1, leaf_hash, branch_hash)
        h = branch_hash(h, h)
        cache.append(h)
    else:
        h = leaf_hash(E, E)
        cache.append(h)
    return h


def hash_leaves(leaves, leaf_hash=leaf_hash, branch_hash=branch_hash, fill=True):
    stack = []
    depth = -1
    term_cache = []

    for idx, (a, b) in enumerate(leaves):
        h = leaf_hash(a, b)
        b = idx + 1
        while b & 1 == 0:
            h = branch_hash(stack.pop(), h)
            b = b >> 1
            depth -= 1
        stack.append(h)
        depth += 1
    leaf_count = leaf_count_filled = idx + 1

    if fill:
        height = math.ceil(math.log(leaf_count_filled, 2))
        fill_count = pow(2, height) - leaf_count_filled
        fill_depth = 0
        while fill_count:
            if fill_count & 1:
                h = terminator_hash(term_cache, fill_depth, leaf_hash, branch_hash)
                leaf_count_filled += pow(2, fill_depth)
                b = leaf_count_filled
                c = 0
                while b & 1 == 0:
                    b = b >> 1
                    c += 1
                for _ in range(c - fill_depth):
                    h = branch_hash(stack.pop(), h)
                    b = b >> 1
                    depth -= 1
                stack.append(h)
            fill_depth += 1
            fill_count = fill_count >> 1

    root = stack.pop()
    while stack:
        root = branch_hash(stack.pop(), root)

    return {
        "leaf_count": leaf_count,
        "leaf_count_filled": leaf_count_filled,
        "root": root,
    }
