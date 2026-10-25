#!/usr/bin/env python3
"""Differential migration oracle. Reads the CURRENT Python working tree.

No tables are generated here: Rust tables are now independent authoritative
source. No Plover application, installation, or live settings are touched.
"""
import argparse
import contextlib
import ctypes as c
import importlib
import io
import itertools
import json
from pathlib import Path
import random
import re
import sys

ENGINE_ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--samples', type=int, default=20000)
parser.add_argument('--plover-root', type=Path, default=ENGINE_ROOT.parent / 'Plover_Mejiro')
parser.add_argument('--library', type=Path, default=ENGINE_ROOT / 'target/debug/libmejiro_native.so')
ARGS = parser.parse_args()
ROOT = ARGS.plover_root.expanduser().resolve()
if not (ROOT / 'Mejiro/dictionaries/default/mejiro.py').is_file():
    parser.error('Plover source not found; specify --plover-root PATH')
sys.path.insert(0, str(ROOT))
BASE = 'Mejiro.dictionaries.default.'
module = importlib.import_module(BASE + 'mejiro')
func = importlib.import_module(BASE + 'func')
grammar = importlib.import_module(BASE + 'grammar')
settings = importlib.import_module(BASE + 'settings')
verbs = importlib.import_module(BASE + 'verb')
abbr = importlib.import_module(BASE + 'abbreviations')
typing = importlib.import_module(BASE + 'translate')
CONTEXTS = ['unknown', 'nominalizable', 'incomplete', 'attributive_na',
            'question_predicate', 'noun', 'place', 'indefinite_noun', 'question_particle']
VOWELS = ['A', 'I', 'U', 'IA', 'AU', 'YA', 'YU', 'YAU', 'Y', 'YI', 'YIA', 'YIU', 'YIAU', 'IU', 'IAU']


class State(c.Structure):
    _fields_ = [('vowel', c.c_uint8), ('context', c.c_uint8),
                ('mode', c.c_uint8), ('pending', c.c_uint8)]


def load_library():
    library = c.CDLL(str(ARGS.library.expanduser().resolve()))
    library.mejiro_abi_version.restype = c.c_uint32
    assert library.mejiro_abi_version() == 1
    library.mejiro_translate.argtypes = [c.c_char_p, c.c_size_t, c.POINTER(State),
        c.c_bool, c.c_void_p, c.c_size_t, c.POINTER(State), c.POINTER(c.c_uint8)]
    library.mejiro_translate.restype = c.c_int32
    return library


def reference(stroke, state):
    func.LAST_VOWEL_STROKE = VOWELS[state.vowel]
    module.is_typing_mode = state.mode != 0
    module.typing_mode = 0 if state.mode == 1 else 1
    typing._pending_sokuon = bool(state.pending)
    with contextlib.redirect_stdout(io.StringIO()):
        result = module.lookup((stroke,))
    context = 'unknown'
    if result.startswith('{:mejiro:auto:'):
        _, _, _, mode, left, right = ('x:' + result[2:-1]).split(':')
        result = func.joshi(left, right, CONTEXTS[state.context])
        context = grammar.particle_context(result)
        # Reference lookup intentionally defers typing and recording to formatter.
        if mode != 'kana':
            result = typing.kana_to_typing_output(result, int(mode))
    else:
        tag = re.search(r'\{:mejiro:([^}]+)\}$', result)
        if tag:
            context = tag[1]
            result = result[:tag.start()]
        assert result.startswith('{^') and result.endswith('^}'), result
        result = result[2:-2]
    return result, CONTEXTS.index(context), VOWELS.index(func.LAST_VOWEL_STROKE), int(typing._pending_sokuon), (CONTEXTS.index(context) if result else state.context)


def corpus(samples):
    particles = settings.PARTICLE_KEY_LIST
    stems = set(abbr.ABSTRACT_MAP)
    stems |= {key + '*' for table in (abbr.VERB_GODAN_MAP, abbr.VERB_KAMI_MAP, abbr.VERB_SIMO_MAP) for key in table}
    stems |= {left + '-' + right + '*' for left in abbr.ABSTRACT_MAP_LEFT for right in abbr.ABSTRACT_MAP_RIGHT}
    stems |= {'-*', 'K-', 'I-K', 'A-ST', '-TN', '-SKNI', '-TNIA'}
    # Every registered word with all 64 auxiliary/particle combinations.
    for stem in sorted(stems):
        star = '*' if stem.endswith('*') else ''
        left, right = stem.removesuffix('*').split('-')
        for lp in particles:
            for rp in particles:
                yield left + lp + '-' + right + rp + star
    # Every consonant/vowel on each hand, and every particle pair in every
    # grammatical context are exercised by main's deterministic state choices.
    consonants = [x[0] for x in settings.conso_stroke_to_roma]
    for consonant in consonants:
        for vowel in [''] + VOWELS:
            for p in particles:
                yield consonant + vowel + p + '-'
                yield '-' + consonant + vowel + p
    rng = random.Random(310)
    for _ in range(samples):
        def hand():
            return rng.choice(consonants) + rng.choice([''] + VOWELS) + rng.choice(particles)
        yield ('#' if rng.randrange(4) == 0 else '') + hand() + '-' + hand() + ('*' if rng.randrange(2) else '')


def main():
    args = ARGS
    library = load_library()
    count = controls = 0
    rng = random.Random(31)
    inputs = ((stroke, State(rng.randrange(15), rng.randrange(9), rng.randrange(3), rng.randrange(2)))
              for stroke in corpus(args.samples))
    boundaries = ((left + '-' + right, State(0, context, mode, pending))
                  for left in settings.PARTICLE_KEY_LIST
                  for right in settings.PARTICLE_KEY_LIST
                  for context in range(9) for mode in range(3) for pending in range(2))
    def check(stroke, state):
        nonlocal count, controls
        output = c.create_string_buffer(2048)
        next_state = State()
        context = c.c_uint8()
        raw = stroke.encode()
        rc = library.mejiro_translate(raw, len(raw), c.byref(state), False, output, len(output), c.byref(next_state), c.byref(context))
        if rc == -3:
            controls += 1
            return state
        assert rc == 0, (stroke, rc)
        expected = reference(stroke, state)
        actual = (output.value.decode(), context.value, next_state.vowel, next_state.pending, next_state.context)
        assert actual == expected, (stroke, tuple(getattr(state, k) for k, _ in State._fields_), actual, expected)
        count += 1
        return next_state

    for stroke, state in itertools.chain(inputs, boundaries):
        check(stroke, state)
    # Consecutive committed states, including empty output and key boundaries.
    words = ['n-*', 'k-k', 'ntk-', '-nt', 'KA-', 'KI-', 'K-I', 'KAtk-',
             'TA-', '-n', 'STNtk-', 'K-T', 'K-KAU', 'N-N', 'k-nk', 'ntk-k']
    for mode in range(3):
        state = State(0, 0, mode, 0)
        for _ in range(3000):
            state = check(rng.choice(words), state)
        for pending in range(2):
            check('#STKNYIAUntk-STKNYIAUntk*', State(0, 1, mode, pending))

    # Command dictionary priority, bypassing typing conversion, with typed keys
    # serialized by the desktop adapter. Ignore attachment-only syntax.
    def normalized(text):
        text = re.sub(r'\{\^([^}]*)\}', lambda m: m[1].removesuffix('^'), text)
        return text.replace('{}', '')
    commands = json.loads((ROOT / 'Mejiro/dictionaries/default/mejiro_commands.json').read_text())
    command_count = 0
    for stroke, expected in commands.items():
        state = State(1, 1, 2, 1)
        output = c.create_string_buffer(2048)
        next_state = State()
        context = c.c_uint8()
        raw = stroke.encode()
        rc = library.mejiro_translate(raw, len(raw), c.byref(state), True, output, len(output), c.byref(next_state), c.byref(context))
        if expected.startswith('='):
            assert rc == -3, (stroke, rc)
            continue
        assert rc == 0, (stroke, rc)
        actual = '{^' + output.value.decode() + '^}'
        assert normalized(actual) == normalized(expected), (stroke, actual, expected)
        assert (next_state.vowel, next_state.mode, next_state.pending) == (1, 2, 1)
        command_count += 1
    print(f'{command_count} command dictionary outputs matched.')
    print(f'{count:,} Plover/Rust conversions matched; {controls} control requests checked separately in Rust tests.')


if __name__ == '__main__':
    main()
