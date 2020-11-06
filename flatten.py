#!/usr/bin/env python3

import sys

for line in sys.stdin:
    for char in line:
        if char == "\x1b":
            print("{ESC}", end='')
        else:
            print(char, end='')
