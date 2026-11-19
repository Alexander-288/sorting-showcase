# Sorting Showcase

Five sorting algorithms, each shown in a different way.

- **Bubble sort**: a 512-byte x86 boot sector that beeps the PC speaker (`bootloader/`)
- **Selection sort**: a Create Aeronautics airship demo (not built yet)
- **Quicksort**: every bar is a real desktop window (`quicksort-windows/`)
- **Merge sort**: a fullscreen Windows screensaver (`mergesort-saver/`)
- **Insertion sort**: a card animation in the terminal (`python/`)

## Bubble sort bootloader

Needs `nasm` and `qemu` (`scoop install nasm qemu`).

```powershell
.\bootloader\run.ps1
```

Sixteen bars in text mode. The two bars being compared turn red, the sorted tail turns green, and every swap beeps. To boot it on real hardware, write `bubble.bin` to the first sector of a USB stick and boot in legacy BIOS mode.

## Quicksort windows

```powershell
cargo run --release -p quicksort-windows -- 10
```

The number is how many windows to use (2-30). The pivot window is yellow and flashes, the window being checked is red, windows smaller than the pivot are purple, and finished windows are green. Press **Esc** or close any window to stop.

## Merge sort screensaver

```powershell
cargo run --release -p mergesort-saver
```

This opens it in a normal window. To install it as a real screensaver, run `.\mergesort-saver\make-scr.ps1`, then right-click `mergesort.scr` and choose **Install**.

As a screensaver it takes these arguments: `/s` runs fullscreen, `/c` shows settings, and `/p` (preview) is skipped. Any key, click or mouse move closes it. It shuffles, sorts, holds the finished rainbow, and loops.

## Insertion sort in the terminal

```powershell
python python\insertion_sort.py
```

Uses only the standard library. Run it with `--help` to see the options.
