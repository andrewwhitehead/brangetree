import array
import gzip
import re

from bitstring import Bits


def iter_zipped_blocks(filename):
    with gzip.open(filename, "rb") as fp:
        yield from fp


def iter_zipped_bits(filename):
    with gzip.open(filename, "rb") as fp:
        for block in fp:
            yield from iter_bits(block)


def iter_bits(block):
    ql, remain = divmod(len(block), 8)
    if ql:
        arr = array.array("Q", block[: ql * 8])
        arr.byteswap()  # FIXME arch dependent
        for elt in arr:
            for pos in range(63, 0, -1):
                yield elt >> pos & 1
            yield elt & 1
    if remain:
        yield from Bits(block[-remain:])


def natural_sort_key(s, _nsre=re.compile("([0-9]+)")):
    return [int(text) if text.isdigit() else text.lower() for text in _nsre.split(s)]


def natural_sort(lst):
    lst.sort(key=natural_sort_key)
