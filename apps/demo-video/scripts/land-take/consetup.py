"""Size and colour the console this process is attached to, for a film take.

Run from inside the take window (driver.sh). Changes only this console
window: font, 16-colour palette, buffer and window size. Writes the window
handle to argv[1] so the capture can target it.
Usage: python -I consetup.py <hwnd-file> <font-height-px> <cols> <rows>
"""
import ctypes
import sys
from ctypes import wintypes

k32 = ctypes.WinDLL("kernel32", use_last_error=True)
u32 = ctypes.WinDLL("user32", use_last_error=True)
out = k32.GetStdHandle(-11)


class COORD(ctypes.Structure):
    _fields_ = [("X", wintypes.SHORT), ("Y", wintypes.SHORT)]


class SMALL_RECT(ctypes.Structure):
    _fields_ = [("Left", wintypes.SHORT), ("Top", wintypes.SHORT), ("Right", wintypes.SHORT), ("Bottom", wintypes.SHORT)]


class FONT(ctypes.Structure):
    _fields_ = [("cbSize", wintypes.ULONG), ("nFont", wintypes.DWORD), ("dwFontSize", COORD),
                ("FontFamily", wintypes.UINT), ("FontWeight", wintypes.UINT), ("FaceName", wintypes.WCHAR * 32)]


class INFOEX(ctypes.Structure):
    _fields_ = [("cbSize", wintypes.ULONG), ("dwSize", COORD), ("dwCursorPosition", COORD), ("wAttributes", wintypes.WORD),
                ("srWindow", SMALL_RECT), ("dwMaximumWindowSize", COORD), ("wPopupAttributes", wintypes.WORD),
                ("bFullscreenSupported", wintypes.BOOL), ("ColorTable", wintypes.DWORD * 16)]


def check(ok, what):
    if not ok:
        raise OSError(f"{what} failed: {ctypes.get_last_error()}")


hwnd_file = sys.argv[1]
height, cols, rows = (int(a) for a in sys.argv[2:5])

font = FONT(cbSize=ctypes.sizeof(FONT), dwFontSize=COORD(0, height), FontFamily=54, FontWeight=400, FaceName="Cascadia Mono")
check(k32.SetCurrentConsoleFontEx(out, False, ctypes.byref(font)), "SetCurrentConsoleFontEx")
check(k32.GetCurrentConsoleFontEx(out, False, ctypes.byref(font)), "GetCurrentConsoleFontEx")
cell_w, cell_h = font.dwFontSize.X, font.dwFontSize.Y

# Dark neutral palette (COLORREF is 0x00BBGGRR). Order: black, blue, green, cyan,
# red, magenta, yellow, white, then the bright variants.
hexes = ["0d1117", "4c8ef7", "3fb950", "39c5cf", "f85149", "bc8cff", "d29922", "c9d1d9",
         "6e7681", "79c0ff", "56d364", "56d4dd", "ff7b72", "d2a8ff", "e3b341", "f0f6fc"]
info = INFOEX(cbSize=ctypes.sizeof(INFOEX))
check(k32.GetConsoleScreenBufferInfoEx(out, ctypes.byref(info)), "GetConsoleScreenBufferInfoEx")
for i, h in enumerate(hexes):
    r, g, b = (int(h[j:j + 2], 16) for j in (0, 2, 4))
    info.ColorTable[i] = r | (g << 8) | (b << 16)
info.wAttributes = 0x07  # white on black, i.e. c9d1d9 on 0d1117
info.dwSize = COORD(cols, rows)
info.srWindow = SMALL_RECT(0, 0, cols, rows)  # the API treats Right/Bottom as exclusive here
check(k32.SetConsoleScreenBufferInfoEx(out, ctypes.byref(info)), "SetConsoleScreenBufferInfoEx")
k32.SetConsoleScreenBufferSize(out, COORD(cols, rows))
k32.SetConsoleWindowInfo(out, True, ctypes.byref(SMALL_RECT(0, 0, cols - 1, rows - 1)))
k32.SetConsoleTitleW("pytxo-film")

hwnd = k32.GetConsoleWindow()
u32.SetWindowPos(hwnd, None, 0, 0, 0, 0, 0x0001 | 0x0004 | 0x0010)  # move to 0,0; keep size, z-order, focus
with open(hwnd_file, "w") as f:
    f.write(f"{hwnd} {cols}x{rows} cell {cell_w}x{cell_h} client {cols * cell_w}x{rows * cell_h}\n")
