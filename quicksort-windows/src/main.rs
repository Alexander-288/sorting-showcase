// Quicksort where every bar is a real desktop window.
// Run: cargo run --release -p quicksort-windows -- [count]   (Esc or closing a window quits)
#![windows_subsystem = "windows"]

use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{mem, thread};

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::HiDpi::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_ESCAPE};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

// Bar colours, stored in each window's GWLP_USERDATA.
const IDLE: isize = 0;
const PIVOT: isize = 1;
const COMPARE: isize = 2;
const SORTED: isize = 3;
const SMALL: isize = 4; // already on the "smaller than pivot" side

static QUIT: AtomicBool = AtomicBool::new(false);

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    r as u32 | (g as u32) << 8 | (b as u32) << 16
}

unsafe extern "system" fn wndproc(h: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = mem::zeroed();
                let dc = BeginPaint(h, &mut ps);
                let color = match GetWindowLongPtrW(h, GWLP_USERDATA) {
                    PIVOT => rgb(255, 196, 0),
                    COMPARE => rgb(235, 80, 80),
                    SORTED => rgb(70, 200, 120),
                    SMALL => rgb(150, 110, 230),
                    _ => rgb(80, 140, 230),
                };
                let mut rc: RECT = mem::zeroed();
                GetClientRect(h, &mut rc);
                let brush = CreateSolidBrush(color);
                FillRect(dc, &rc, brush);
                DeleteObject(brush);

                // value in big white digits
                let mut text = [0u16; 16];
                let len = GetWindowTextW(h, text.as_mut_ptr(), 16);
                let font = CreateFontW(48, 0, 0, 0, 700, 0, 0, 0, 0, 0, 0, 0, 0, wide("Segoe UI").as_ptr());
                let old = SelectObject(dc, font);
                SetBkMode(dc, TRANSPARENT as _);
                SetTextColor(dc, rgb(255, 255, 255));
                DrawTextW(dc, text.as_ptr(), len, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
                SelectObject(dc, old);
                DeleteObject(font);
                EndPaint(h, &ps);
                0
            }
            WM_CLOSE => {
                QUIT.store(true, Ordering::Relaxed);
                0
            }
            _ => DefWindowProcW(h, msg, w, l),
        }
    }
}

struct Desk {
    bars: Vec<HWND>, // bars[slot] = window currently at that slot
    values: Vec<u32>,
    left: i32,
    bottom: i32,
    slot_w: i32,
    unit_h: f32,
    comparisons: u32,
    swaps: u32,
}

impl Desk {
    fn x(&self, slot: usize) -> i32 {
        self.left + slot as i32 * self.slot_w + 4
    }
    fn h(&self, value: u32) -> i32 {
        (value as f32 * self.unit_h) as i32 + 40
    }

    fn set(&self, slot: usize, state: isize) {
        unsafe {
            SetWindowLongPtrW(self.bars[slot], GWLP_USERDATA, state);
            InvalidateRect(self.bars[slot], null_mut(), 1);
        }
    }

    fn flash(&self, slot: usize, on: bool) {
        let info = FLASHWINFO {
            cbSize: mem::size_of::<FLASHWINFO>() as u32,
            hwnd: self.bars[slot],
            dwFlags: if on { FLASHW_CAPTION | FLASHW_TIMER } else { FLASHW_STOP },
            uCount: 0,
            dwTimeout: 0,
        };
        unsafe { FlashWindowEx(&info) };
    }

    fn place(&self, hwnd: HWND, x: i32, lift: i32, value: u32) {
        let h = self.h(value);
        unsafe {
            SetWindowPos(hwnd, null_mut(), x, self.bottom - h - lift, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
        }
    }

    // Slide two windows past each other: one hops over the top, one ducks under.
    fn swap(&mut self, i: usize, j: usize) {
        if i == j {
            return;
        }
        self.swaps += 1;
        let (a, b) = (self.bars[i], self.bars[j]);
        let (va, vb) = (self.values[i], self.values[j]);
        let (xi, xj) = (self.x(i), self.x(j));
        let frames = 18 + (j as i32 - i as i32).abs() * 2;
        for f in 1..=frames {
            let t = f as f32 / frames as f32;
            let ease = t * t * (3.0 - 2.0 * t);
            let hop = ((t * std::f32::consts::PI).sin() * 120.0) as i32;
            self.place(a, xi + ((xj - xi) as f32 * ease) as i32, hop, va);
            self.place(b, xj + ((xi - xj) as f32 * ease) as i32, -hop / 3, vb);
            pump(12);
        }
        self.bars.swap(i, j);
        self.values.swap(i, j);
    }

    // Lomuto partition with the last window as pivot.
    fn quicksort(&mut self, lo: usize, hi: usize) {
        if lo > hi {
            return;
        }
        if lo == hi {
            self.set(lo, SORTED);
            pump(250);
            return;
        }
        let pivot = self.values[hi];
        self.set(hi, PIVOT);
        self.flash(hi, true);
        pump(500);

        let mut store = lo;
        for k in lo..hi {
            self.comparisons += 1;
            self.set(k, COMPARE);
            pump(260);
            if self.values[k] < pivot {
                self.set(k, SMALL);
                self.swap(store, k);
                store += 1;
            } else {
                self.set(k, IDLE);
            }
        }
        self.flash(hi, false);
        self.swap(store, hi); // pivot jumps into its final slot
        for k in lo..=hi {
            self.set(k, IDLE);
        }
        self.set(store, SORTED);
        pump(400);

        if store > lo {
            self.quicksort(lo, store - 1);
        }
        self.quicksort(store + 1, hi);
    }
}

// Keep windows responsive while waiting; exit on Esc or close.
fn pump(ms: u64) {
    let end = Instant::now() + Duration::from_millis(ms);
    unsafe {
        let mut msg: MSG = mem::zeroed();
        loop {
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            if QUIT.load(Ordering::Relaxed) || GetAsyncKeyState(VK_ESCAPE as i32) < 0 {
                std::process::exit(0);
            }
            if Instant::now() >= end {
                break;
            }
            thread::sleep(Duration::from_millis(4));
        }
    }
}

// Tiny xorshift so we need no rand crate.
fn shuffled(n: usize) -> Vec<u32> {
    let mut s = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64 | 1;
    let mut v: Vec<u32> = (1..=n as u32).collect();
    for i in (1..n).rev() {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        v.swap(i, (s % (i as u64 + 1)) as usize);
    }
    v
}

fn main() {
    let n: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(10).clamp(2, 30);

    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let inst = GetModuleHandleW(null_mut());
        let class = wide("SortBar");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: inst,
            lpszClassName: class.as_ptr(),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            ..mem::zeroed()
        };
        RegisterClassW(&wc);

        let mut work: RECT = mem::zeroed();
        SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut work as *mut _ as _, 0);
        let width = work.right - work.left;
        let height = work.bottom - work.top;
        let slot_w = width / n as i32;

        let values = shuffled(n);
        let mut desk = Desk {
            bars: Vec::new(),
            values: values.clone(),
            left: work.left,
            bottom: work.bottom - 10,
            slot_w,
            unit_h: (height as f32 * 0.75) / n as f32,
            comparisons: 0,
            swaps: 0,
        };

        // Windows pop up one at a time, left to right.
        for (slot, &v) in values.iter().enumerate() {
            let h = desk.h(v);
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
                class.as_ptr(),
                wide(&v.to_string()).as_ptr(),
                WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
                desk.x(slot),
                desk.bottom - h,
                slot_w - 8,
                h,
                null_mut(),
                null_mut(),
                inst,
                null_mut(),
            );
            desk.bars.push(hwnd);
            pump(120);
        }
        pump(800);

        desk.quicksort(0, n - 1);

        // Victory lap: each window hops in order.
        for slot in 0..n {
            let (hwnd, v, x) = (desk.bars[slot], desk.values[slot], desk.x(slot));
            for f in 0..=10 {
                let hop = ((f as f32 / 10.0 * std::f32::consts::PI).sin() * 60.0) as i32;
                desk.place(hwnd, x, hop, v);
                pump(10);
            }
        }
        assert!(desk.values.windows(2).all(|p| p[0] <= p[1]));
        let summary = format!("{n} windows sorted.\n{} comparisons, {} swaps.", desk.comparisons, desk.swaps);
        MessageBoxW(null_mut(), wide(&summary).as_ptr(), wide("Quicksort").as_ptr(), MB_OK | MB_TOPMOST);
    }
}
