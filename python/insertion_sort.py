#!/usr/bin/env python3
"""
insertion_sort.py — insertion sort, played out like sorting a hand of cards.

The screen is redrawn every step. The card being placed is lifted out of
the hand (yellow, double border) and slides left one slot at a time while
the card it passes (cyan) shifts over to the right. Sorted cards are green,
unsorted cards are dim.

    python insertion_sort.py                 # 8 cards, slow and readable
    python insertion_sort.py -n 12 -s 0.5    # bigger hand, faster
    python insertion_sort.py --numbers       # numbers instead of playing cards
    python insertion_sort.py --ascii         # plain +-| borders for old terminals
    python insertion_sort.py --seed 42       # repeatable deal

Standard library only. Works in any ANSI terminal (Windows 10+ included).
Set NO_COLOR=1 to turn colours off.
"""

import argparse
import os
import random
import shutil
import sys
import time

# ── colour ──────────────────────────────────────────────────────────────────

USE_COLOR = sys.stdout.isatty() and "NO_COLOR" not in os.environ
if os.name == "nt":
    os.system("")  # enables ANSI escape handling in the Windows console

SORTED = "32"      # green
KEY = "1;33"       # bold yellow
SHIFTED = "1;36"   # bold cyan
UNSORTED = "2"     # dim
BOLD = "1"


def c(code, text):
    return f"\033[{code}m{text}\033[0m" if USE_COLOR else text


# ── cards ───────────────────────────────────────────────────────────────────

RANKS = {1: "A", 11: "J", 12: "Q", 13: "K"}
SUITS = ["♠", "♥", "♦", "♣"]

CARD_W = 9   # outer width of a card
CARD_H = 7   # outer height of a card
LIFT = 3     # how many rows the moving card floats above the hand
GAP = 1      # columns between cards

BORDERS = {
    "single": dict(tl="┌", tr="┐", bl="└", br="┘", h="─", v="│"),
    "double": dict(tl="╔", tr="╗", bl="╚", br="╝", h="═", v="║"),
    "ascii":  dict(tl="+", tr="+", bl="+", br="+", h="-", v="|"),
    "ascii2": dict(tl="#", tr="#", bl="#", br="#", h="=", v="#"),
}


class Card:
    def __init__(self, value, suit=None):
        self.value = value
        self.suit = suit

    @property
    def rank(self):
        if self.suit is None:
            return str(self.value)
        return RANKS.get(self.value, str(self.value))

    @property
    def label(self):
        return self.rank + (self.suit or "")

    def __lt__(self, other):
        return self.value < other.value


def deal(n, numbers, rng):
    if numbers:
        return [Card(v) for v in rng.sample(range(1, 100), n)]
    deck = [Card(v, s) for s in SUITS for v in range(1, 14)]
    return rng.sample(deck, n)


def card_lines(card, style, ascii_mode):
    """Return CARD_H strings, each CARD_W visible characters wide."""
    double = style == KEY
    b = BORDERS[("ascii2" if double else "ascii") if ascii_mode else ("double" if double else "single")]
    inner = CARD_W - 2
    pip = card.suit or "•"
    top_corner = card.label.ljust(inner)
    bot_corner = card.label.rjust(inner)
    middle = card.rank if card.suit is None else pip
    rows = [
        b["tl"] + b["h"] * inner + b["tr"],
        b["v"] + top_corner + b["v"],
        b["v"] + " " * inner + b["v"],
        b["v"] + middle.center(inner) + b["v"],
        b["v"] + " " * inner + b["v"],
        b["v"] + bot_corner + b["v"],
        b["bl"] + b["h"] * inner + b["br"],
    ]
    return [c(style, r) for r in rows]


BLANK = " " * CARD_W


# ── rendering ───────────────────────────────────────────────────────────────

class Screen:
    def __init__(self, delay, ascii_mode, total):
        self.delay = delay
        self.ascii = ascii_mode
        self.total = total
        self.comparisons = 0
        self.shifts = 0

    def frame(self, hand, sorted_upto, message, key_at=None, shifted_at=None, pause=1.0):
        cols = shutil.get_terminal_size((80, 24)).columns
        per_row = max(1, (cols - 2 + GAP) // (CARD_W + GAP))

        out = ["\033[H\033[J" if USE_COLOR else "\n" * 2]
        out.append(c(BOLD, "  INSERTION SORT") + c(UNSORTED, "  —  each card slides left until it fits"))
        out.append("  " + "   ".join([c(SORTED, "■ sorted"), c(KEY, "■ moving"),
                                     c(SHIFTED, "■ shifted"), c(UNSORTED, "■ unsorted")]))
        out.append("")

        for start in range(0, len(hand), per_row):
            chunk = range(start, min(start + per_row, len(hand)))
            lines = [[] for _ in range(CARD_H + LIFT)]
            for i in chunk:
                if i == key_at:
                    style = KEY
                elif i == shifted_at:
                    style = SHIFTED
                elif i <= sorted_upto:
                    style = SORTED
                else:
                    style = UNSORTED
                art = card_lines(hand[i], style, self.ascii)
                offset = 0 if i == key_at else LIFT  # lifted card sits higher
                for r in range(CARD_H + LIFT):
                    k = r - offset
                    lines[r].append(art[k] if 0 <= k < CARD_H else BLANK)
            for parts in lines:
                out.append("  " + (" " * GAP).join(parts))
            out.append("")

        out.append("  " + message)
        out.append("")
        out.append(c(UNSORTED, f"  cards {self.total}   comparisons {self.comparisons}   "
                               f"shifts {self.shifts}   (Ctrl+C to quit)"))
        sys.stdout.write("\n".join(out) + "\n")
        sys.stdout.flush()
        time.sleep(self.delay * pause)


# ── the sort ────────────────────────────────────────────────────────────────

def insertion_sort(hand, scr):
    scr.frame(hand, 0, "Dealt. The first card on its own is already a sorted hand.", pause=2)

    for i in range(1, len(hand)):
        key = hand[i]
        j = i
        scr.frame(hand, i - 1, f"Pick up {c(KEY, key.label)}.", key_at=j, pause=1.2)

        while j > 0:
            scr.comparisons += 1
            left = hand[j - 1]
            if not key < left:
                scr.frame(hand, i - 1, f"{c(KEY, key.label)} ≥ {left.label} — stop here.",
                          key_at=j, pause=0.8)
                break
            hand[j], hand[j - 1] = left, key
            j -= 1
            scr.shifts += 1
            scr.frame(hand, i, f"{c(KEY, key.label)} < {c(SHIFTED, left.label)} — "
                               f"{c(SHIFTED, left.label)} shifts right.",
                      key_at=j, shifted_at=j + 1)

        where = "at the front" if j == 0 else f"after {hand[j - 1].label}"
        scr.frame(hand, i, f"{c(SORTED, key.label)} drops into place {where}.", pause=1.2)

    scr.frame(hand, len(hand) - 1, c(BOLD, "Sorted."), pause=0)


# ── main ────────────────────────────────────────────────────────────────────

def main():
    p = argparse.ArgumentParser(description="Insertion sort, drawn as a hand of cards.")
    p.add_argument("-n", type=int, default=8, help="hand size (default 8)")
    p.add_argument("-s", "--speed", type=float, default=1.0,
                   help="seconds per step (default 1.0; lower is faster)")
    p.add_argument("--numbers", action="store_true", help="sort numbers instead of playing cards")
    p.add_argument("--ascii", action="store_true", help="draw borders with +-| only")
    p.add_argument("--seed", type=int, help="random seed for a repeatable deal")
    args = p.parse_args()

    n = max(2, min(args.n, 52 if not args.numbers else 99))
    hand = deal(n, args.numbers, random.Random(args.seed))

    if USE_COLOR:
        sys.stdout.write("\033[?25l")  # hide cursor while animating
    try:
        insertion_sort(hand, Screen(max(0.0, args.speed), args.ascii, n))
    finally:
        if USE_COLOR:
            sys.stdout.write("\033[0m\033[?25h")  # restore cursor
            sys.stdout.flush()

    assert all(not hand[k + 1] < hand[k] for k in range(n - 1))


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print()
        sys.exit(130)
