pub const SAMPLE_RATE: usize = 48_000;
pub const FRAMES_PER_VIDEO_FRAME: usize = SAMPLE_RATE / 60;
pub const STEREO_SAMPLES: usize = FRAMES_PER_VIDEO_FRAME * 2;

pub const SOUND_TABLE: [[i16; FRAMES_PER_VIDEO_FRAME]; 256] = build_sound_table();

const fn build_sound_table() -> [[i16; FRAMES_PER_VIDEO_FRAME]; 256] {
    let mut table = [[0i16; FRAMES_PER_VIDEO_FRAME]; 256];
    let mut id = 1;
    while id < 256 {
        let mut i = 0;
        while i < FRAMES_PER_VIDEO_FRAME {
            table[id][i] = sample(id as u32, i as u32);
            i += 1;
        }
        id += 1;
    }
    table
}

const fn sample(id: u32, phase: u32) -> i16 {
    let freq = 80 + ((id * 37) % 1800);
    let t = (phase * freq * 256 / SAMPLE_RATE as u32) & 255;
    let amp = 700 + ((id as i16 & 31) * 80);
    match id & 3 {
        0 => square(t, amp),
        1 => triangle(t, amp),
        2 => saw(t, amp),
        _ => sineish(t, amp),
    }
}

const fn square(t: u32, amp: i16) -> i16 {
    if t < 128 { amp } else { -amp }
}

const fn triangle(t: u32, amp: i16) -> i16 {
    let v = if t < 128 { t as i32 } else { 255 - t as i32 };
    (((v * 4 - 255) * amp as i32) / 255) as i16
}

const fn saw(t: u32, amp: i16) -> i16 {
    (((t as i32 - 128) * amp as i32) / 128) as i16
}

const fn sineish(t: u32, amp: i16) -> i16 {
    let x = if t < 128 { t as i32 } else { 255 - t as i32 };
    let parabola = (x * (128 - x) * 4) / 128;
    let signed = if t < 128 { parabola } else { -parabola };
    ((signed * amp as i32) / 128) as i16
}
