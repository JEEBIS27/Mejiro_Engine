#!/usr/bin/env python3
"""Check the Plover ctypes bridge against the original native preview ABI.

The former independent Python-language oracle has been retired: production
Plover now uses Rust. This checks serialization, errors and control transport,
not agreement between independent implementations of Japanese rules.
"""
import argparse
import ctypes as c
from pathlib import Path
import random
import sys

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--samples", type=int, default=1000)
parser.add_argument("--plover-root", type=Path, default=ROOT.parent / "Plover_Mejiro")
parser.add_argument("--library", type=Path, default=ROOT / "target/debug/libmejiro_native.so")
args = parser.parse_args()
sys.path.insert(0, str(args.plover_root.resolve()))
from Mejiro.engine import Engine, State

bridge = Engine(args.library.resolve())
library = bridge.library
library.mejiro_translate.argtypes = [
    c.c_char_p, c.c_size_t, c.POINTER(State), c.c_bool,
    c.c_void_p, c.c_size_t, c.POINTER(State), c.POINTER(c.c_uint8),
]
library.mejiro_translate.restype = c.c_int32
rng = random.Random(74)
consonants = ("", "S", "T", "K", "N", "ST", "SK", "TK", "SN", "TN",
              "KN", "TKN", "STK", "STN", "SKN", "STKN")
vowels = ("A", "I", "U", "IA", "AU", "YA", "YU", "YAU", "Y", "YI",
          "YIA", "YIU", "YIAU", "IU", "IAU")
particles = ("", "n", "t", "k", "tk", "nt", "nk", "ntk")
fixed = ["KA-", "STKNIUntk-", "TNYIUntk-", "TNYIAUntk-", "k-k",
         "ntk-", "#", "#-", "-U", "#-n", "#n*", "#-*", "#n",
         "#-t", "#-k", "-KNA", "invalid"]
strokes = fixed + [
    rng.choice(consonants) + rng.choice(vowels) + rng.choice(particles) + "-"
    + rng.choice(consonants) + rng.choice(vowels) + rng.choice(particles)
    for _ in range(args.samples)
]
for raw in strokes:
    state = State(rng.randrange(15), rng.randrange(9), rng.randrange(3), rng.randrange(2))
    encoded = raw.encode("ascii")
    output, next_state, context = c.create_string_buffer(2048), State(), c.c_uint8()
    rc = library.mejiro_translate(encoded, len(encoded), c.byref(state), True,
                                  output, len(output), c.byref(next_state), c.byref(context))
    try:
        result = bridge.preview(raw, state)
    except KeyError:
        assert rc == -1, (raw, rc)
        continue
    if result.request:
        assert rc == -3, (raw, rc, result.request)
    else:
        assert rc == 0, (raw, rc)
        assert result.body == output.value.decode("utf-8"), raw
        assert bytes(result.state) == bytes(next_state), raw
print(f"{len(strokes)} Plover bridge previews match the native ABI (including controls/errors).")
