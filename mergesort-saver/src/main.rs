// Merge sort screensaver: hundreds of rainbow bars, split and merged forever.
// As a .scr: /s runs fullscreen, /c shows settings, /p (preview) is skipped.
// With no arguments it runs in a normal window, handy for `cargo run`.
#![windows_subsystem = "windows"]

use std::time::{SystemTime, UNIX_EPOCH};

use minifb::{Key, MouseButton, Window, WindowOptions};
use windows_sys::Win32::Foundation::{HWND, POINT};
use windows_sys::Win32::UI::HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWL_STYLE, GetCursorPos, GetSystemMetrics, HWND_TOPMOST, MB_OK, MessageBoxW, SM_CXSCREEN, SM_CYSCREEN, SWP_FRAMECHANGED,
    SWP_SHOWWINDOW, SetWindowLongPtrW, SetWindowPos, WS_POPUP, WS_VISIBLE,
};

const SORT_SECONDS: usize = 16; // how long one sort takes on screen
const SHUFFLE_SECONDS: usize = 2;
const HOLD_FRAMES: usize = 150; // pause on the finished rainbow
const BG: u32 = 0x0a0a12;

// One recorded write: arr[idx] = val, while merging the range lo..hi.
#[derive(Clone, Copy)]
struct Op {
    idx: u32,
    val: u32,
    lo: u32,
    hi: u32,
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

// Fisher-Yates shuffle, recorded as writes so it can be animated too.
fn record_shuffle(arr: &mut [u32], rng: &mut Rng) -> Vec<Op> {
    let n = arr.len() as u32;
    let mut ops = Vec::new();
    for i in (1..arr.len()).rev() {
        let j = (rng.next() % (i as u64 + 1)) as usize;
        arr.swap(i, j);
        ops.push(Op { idx: i as u32, val: arr[i], lo: 0, hi: n });
        ops.push(Op { idx: j as u32, val: arr[j], lo: 0, hi: n });
    }
    ops
}

// Top-down merge sort, recording every write into the array.
fn record_merge_sort(arr: &mut [u32]) -> Vec<Op> {
    let mut ops = Vec::new();
    let mut aux = arr.to_vec();
    sort(arr, &mut aux, 0, arr.len(), &mut ops);
    ops
}

fn sort(arr: &mut [u32], aux: &mut [u32], lo: usize, hi: usize, ops: &mut Vec<Op>) {
    if hi - lo < 2 {
        return;
    }
    let mid = (lo + hi) / 2;
    sort(arr, aux, lo, mid, ops);
    sort(arr, aux, mid, hi, ops);
    aux[lo..hi].copy_from_slice(&arr[lo..hi]);
    let (mut i, mut j) = (lo, mid);
    for k in lo..hi {
        let take_left = j >= hi || (i < mid && aux[i] <= aux[j]);
        arr[k] = if take_left { i += 1; aux[i - 1] } else { j += 1; aux[j - 1] };
        ops.push(Op { idx: k as u32, val: arr[k], lo: lo as u32, hi: hi as u32 });
    }
}

// Hue 0..1 to a packed 0RGB colour, scaled by brightness.
fn rainbow(h: f32, bright: f32) -> u32 {
    let f = |n: f32| {
        let k = (n + h * 6.0) % 6.0;
        let v = 1.0 - 0.75 * k.min(4.0 - k).clamp(0.0, 1.0);
        (v * bright * 255.0) as u32
    };
    f(5.0) << 16 | f(3.0) << 8 | f(1.0)
}

fn draw(buf: &mut [u32], w: usize, h: usize, arr: &[u32], active: (u32, u32), last: u32) {
    buf.fill(BG);
    let n = arr.len();
    let bar_w = w / n;
    for x in 0..w {
        let i = x * n / w;
        if bar_w >= 3 && (x + 1) * n / w != i && x + 1 < w {
            continue; // 1px gap at the right edge of wide bars
        }
        let v = arr[i] as f32 / n as f32;
        let bar_h = ((v * 0.9) * h as f32) as usize;
        let inside = (i as u32) >= active.0 && (i as u32) < active.1;
        let color = if i as u32 == last { 0xffffff } else { rainbow(v * 0.83, if inside { 1.0 } else { 0.35 }) };
        for y in h - bar_h..h {
            buf[y * w + x] = color;
        }
    }
}

enum Mode {
    Saver,
    Windowed,
}

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default().to_lowercase();
    let mode = match arg.get(..2) {
        Some("/s") | Some("-s") => Mode::Saver,
        Some("/p") | Some("-p") => return, // no preview in the little monitor
        Some("/c") | Some("-c") => {
            let msg: Vec<u16> = "Merge sort screensaver.\nNo settings yet.\0".encode_utf16().collect();
            let title: Vec<u16> = "Merge Sort\0".encode_utf16().collect();
            unsafe { MessageBoxW(std::ptr::null_mut(), msg.as_ptr(), title.as_ptr(), MB_OK) };
            return;
        }
        _ => Mode::Windowed,
    };

    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
    let (w, h, opts) = match mode {
        Mode::Saver => {
            let (sw, sh) = unsafe { (GetSystemMetrics(SM_CXSCREEN) as usize, GetSystemMetrics(SM_CYSCREEN) as usize) };
            (sw, sh, WindowOptions { borderless: true, topmost: true, ..WindowOptions::default() })
        }
        Mode::Windowed => (1280, 720, WindowOptions::default()),
    };
    let mut window = Window::new("Merge Sort", w, h, opts).expect("could not open window");
    window.set_target_fps(60);
    if let Mode::Saver = mode {
        // minifb keeps a frame on Windows, so make it a bare fullscreen popup ourselves
        let hwnd = window.get_window_handle() as HWND;
        unsafe {
            SetWindowLongPtrW(hwnd, GWL_STYLE, (WS_POPUP | WS_VISIBLE) as isize);
            SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, w as i32, h as i32, SWP_FRAMECHANGED | SWP_SHOWWINDOW);
        }
        window.set_cursor_visibility(false);
    }

    let n = (w / 3).clamp(64, 640);
    let mut arr: Vec<u32> = (1..=n as u32).collect();
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64 | 1;
    let mut rng = Rng(seed);
    let mut buf = vec![BG; w * h];
    let mut start_mouse = None;
    let mut shuffling = true;
    let mut frame = 0usize;

    'outer: loop {
        // Record the next act on a copy, then play it back write by write.
        let mut target = arr.clone();
        let (ops, seconds) = if shuffling {
            (record_shuffle(&mut target, &mut rng), SHUFFLE_SECONDS)
        } else {
            (record_merge_sort(&mut target), SORT_SECONDS)
        };
        let per_frame = (ops.len() / (seconds * 60)).max(1);

        for chunk in ops.chunks(per_frame).chain(std::iter::repeat_n(&[][..], if shuffling { 30 } else { HOLD_FRAMES })) {
            for op in chunk {
                arr[op.idx as usize] = op.val;
            }
            let (active, last) = match chunk.last() {
                Some(op) => ((op.lo, op.hi), op.idx),
                None => ((0, n as u32), u32::MAX), // holding: everything bright
            };
            draw(&mut buf, w, h, &arr, active, last);
            frame += 1;
            if window.update_with_buffer(&buf, w, h).is_err() || !window.is_open() || should_quit(&window, &mode, &mut start_mouse, frame) {
                break 'outer;
            }
        }
        debug_assert!(shuffling || arr.windows(2).all(|p| p[0] <= p[1]));
        shuffling = !shuffling;
    }
}

// Screensaver rules: any key, click or real mouse move ends it.
fn should_quit(window: &Window, mode: &Mode, start: &mut Option<(i32, i32)>, frame: usize) -> bool {
    if frame < 30 {
        return false; // ignore input while the window settles
    }
    if window.is_key_down(Key::Escape) {
        return true;
    }
    let Mode::Saver = mode else { return false };
    if !window.get_keys().is_empty() || window.get_mouse_down(MouseButton::Left) || window.get_mouse_down(MouseButton::Right) {
        return true;
    }
    // real screen position, so moving our own window doesn't count as a move
    let mut p = POINT { x: 0, y: 0 };
    if unsafe { GetCursorPos(&mut p) } != 0 {
        let (sx, sy) = *start.get_or_insert((p.x, p.y));
        if (p.x - sx).abs() + (p.y - sy).abs() > 10 {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaying_ops_sorts_the_array() {
        let mut rng = Rng(12345);
        let mut arr: Vec<u32> = (1..=500).collect();
        let mut shuffled = arr.clone();
        for op in record_shuffle(&mut shuffled, &mut rng) {
            arr[op.idx as usize] = op.val;
        }
        assert_eq!(arr, shuffled);
        let mut sorted = arr.clone();
        for op in record_merge_sort(&mut sorted) {
            arr[op.idx as usize] = op.val;
        }
        assert!(arr.windows(2).all(|p| p[0] < p[1]));
    }
}
