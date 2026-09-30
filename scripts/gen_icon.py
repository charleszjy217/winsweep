#!/usr/bin/env python3
"""Generate a 1024x1024 app icon PNG for WinSweep using only stdlib.

Design: dark #0D1117 background with a green (#39D353) terminal chevron ">"
plus a subtle ring — matching the geek/terminal aesthetic.
"""
import struct
import zlib
import math
import os

W = H = 1024

BG = (13, 17, 23, 255)        # #0D1117
PANEL = (22, 27, 34, 255)     # #161B22
GREEN = (57, 211, 83, 255)    # #39D353
GREEN_DIM = (57, 211, 83, 90) # translucent green
BORDER = (33, 38, 45, 255)    # #21262D

img = bytearray(W * H * 4)


def put(x, y, r, g, b, a):
    if 0 <= x < W and 0 <= y < H:
        i = (y * W + x) * 4
        # simple source-over alpha blend
        if a >= 255:
            img[i] = r
            img[i + 1] = g
            img[i + 2] = b
            img[i + 3] = 255
        else:
            inv = 255 - a
            img[i] = (r * a + img[i] * inv) // 255
            img[i + 1] = (g * a + img[i + 1] * inv) // 255
            img[i + 2] = (b * a + img[i + 2] * inv) // 255
            img[i + 3] = min(255, img[i + 3] + a)


def fill_bg():
    cx, cy = W / 2, H / 2
    for y in range(H):
        for x in range(W):
            # radial deep-space gradient
            d = math.hypot(x - cx, y - cy) / (W * 0.72)
            d = min(d, 1.0)
            r = int(BG[0] + (PANEL[0] - BG[0]) * (1 - d) * 0.25)
            g = int(BG[1] + (PANEL[1] - BG[1]) * (1 - d) * 0.25)
            b = int(BG[2] + (PANEL[2] - BG[2]) * (1 - d) * 0.25)
            put(x, y, r, g, b, 255)


def rounded_rect(x0, y0, x1, y1, rad, color, thickness=0, fill=False):
    for y in range(y0, y1):
        for x in range(x0, x1):
            # distance to rounded rect border
            dx = max(x0 + rad - x, 0, x - (x1 - 1 - rad))
            dy = max(y0 + rad - y, 0, y - (y1 - 1 - rad))
            dist = math.hypot(dx, dy)
            if fill:
                if dist <= rad:
                    put(x, y, *color)
            else:
                if rad - thickness <= dist <= rad:
                    put(x, y, *color)


def thick_line(x0, y0, x1, y1, width, color):
    half = width / 2.0
    minx, maxx = int(min(x0, x1) - half - 2), int(max(x0, x1) + half + 2)
    miny, maxy = int(min(y0, y1) - half - 2), int(max(y0, y1) + half + 2)
    vx, vy = x1 - x0, y1 - y0
    vlen2 = vx * vx + vy * vy or 1
    for y in range(miny, maxy + 1):
        for x in range(minx, maxx + 1):
            t = ((x - x0) * vx + (y - y0) * vy) / vlen2
            t = max(0.0, min(1.0, t))
            px, py = x0 + t * vx, y0 + t * vy
            if math.hypot(x - px, y - py) <= half:
                put(x, y, *color)


def main():
    fill_bg()
    # rounded panel
    rounded_rect(150, 150, 874, 874, 150, BORDER, thickness=6)
    # terminal chevron ">"
    ax, ay = 380, 360
    mx, my = 640, 512
    bx, by = 380, 664
    thick_line(ax, ay, mx, my, 86, GREEN)
    thick_line(mx, my, bx, by, 86, GREEN)
    # underscore cursor after chevron
    thick_line(690, 664, 830, 664, 70, GREEN_DIM)

    raw = bytearray()
    for y in range(H):
        raw.append(0)  # filter type 0
        raw += img[y * W * 4:(y + 1) * W * 4]

    def chunk(tag, data):
        c = struct.pack(">I", len(data)) + tag + data
        c += struct.pack(">I", zlib.crc32(tag + data) & 0xffffffff)
        return c

    sig = b"\x89PNG\r\n\x1a\n"
    ihdr = struct.pack(">IIBBBBB", W, H, 8, 6, 0, 0, 0)
    idat = zlib.compress(bytes(raw), 9)
    png = sig + chunk(b"IHDR", ihdr) + chunk(b"IDAT", idat) + chunk(b"IEND", b"")

    out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "app-icon.png")
    with open(out, "wb") as f:
        f.write(png)
    print("wrote", out, len(png), "bytes")


if __name__ == "__main__":
    main()
