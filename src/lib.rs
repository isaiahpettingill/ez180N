mod font;
mod machine;
mod retro;
mod sdk;
mod sound;
mod video;

pub use sdk::*;

use core::ffi::c_void;
use machine::Console;
use retro::*;

static mut ENV_CB: Option<retro_environment_t> = None;
static mut VIDEO_CB: Option<retro_video_refresh_t> = None;
static mut AUDIO_BATCH_CB: Option<retro_audio_sample_batch_t> = None;
static mut INPUT_POLL_CB: Option<retro_input_poll_t> = None;
static mut INPUT_STATE_CB: Option<retro_input_state_t> = None;
static mut CORE: Option<Console> = None;

#[unsafe(no_mangle)]
pub extern "C" fn retro_api_version() -> u32 {
    RETRO_API_VERSION
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_environment(cb: retro_environment_t) {
    unsafe {
        ENV_CB = Some(cb);
        let mut fmt = RETRO_PIXEL_FORMAT_XRGB8888;
        cb(
            RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
            &mut fmt as *mut _ as *mut c_void,
        );
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_video_refresh(cb: retro_video_refresh_t) {
    unsafe { VIDEO_CB = Some(cb) }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_audio_sample(_cb: retro_audio_sample_t) {}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_audio_sample_batch(cb: retro_audio_sample_batch_t) {
    unsafe { AUDIO_BATCH_CB = Some(cb) }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_input_poll(cb: retro_input_poll_t) {
    unsafe { INPUT_POLL_CB = Some(cb) }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_input_state(cb: retro_input_state_t) {
    unsafe { INPUT_STATE_CB = Some(cb) }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_init() {
    unsafe { CORE = Some(Console::new()) }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_deinit() {
    unsafe { CORE = None }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_system_info(info: *mut retro_system_info) {
    if info.is_null() {
        return;
    }
    unsafe {
        *info = retro_system_info {
            library_name: c"ez180N".as_ptr(),
            library_version: c"0.1.2".as_ptr(),
            valid_extensions: c"gaem".as_ptr(),
            need_fullpath: false,
            block_extract: false,
        };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_system_av_info(info: *mut retro_system_av_info) {
    if info.is_null() {
        return;
    }
    unsafe {
        *info = retro_system_av_info {
            geometry: retro_game_geometry {
                base_width: video::PIXEL_WIDTH as u32,
                base_height: video::PIXEL_HEIGHT as u32,
                max_width: video::PIXEL_WIDTH as u32,
                max_height: video::PIXEL_HEIGHT as u32,
                aspect_ratio: video::PIXEL_WIDTH as f32 / video::PIXEL_HEIGHT as f32,
            },
            timing: retro_system_timing {
                fps: 60.0,
                sample_rate: sound::SAMPLE_RATE as f64,
            },
        };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_controller_port_device(_port: u32, _device: u32) {}

#[unsafe(no_mangle)]
pub extern "C" fn retro_reset() {
    unsafe {
        let core_slot = &raw mut CORE;
        if let Some(core) = (*core_slot).as_mut() {
            core.reset();
        } else {
            CORE = Some(Console::new());
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_load_game(game: *const retro_game_info) -> bool {
    if game.is_null() {
        return false;
    }
    let mut core = Console::new();
    unsafe {
        let game = &*game;
        if game.data.is_null() || game.size == 0 {
            return false;
        }
        let data = core::slice::from_raw_parts(game.data.cast::<u8>(), game.size);
        if !core.load_program(data) {
            return false;
        }
        CORE = Some(core);
    }
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_unload_game() {
    unsafe { CORE = None }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_run() {
    unsafe {
        if let Some(poll) = INPUT_POLL_CB {
            poll();
        }
        let core_slot = &raw mut CORE;
        let Some(core) = (*core_slot).as_mut() else {
            return;
        };
        if let Some(input) = INPUT_STATE_CB {
            core.set_inputs(read_inputs(input));
        }
        core.run_frame();
        if let Some(video) = VIDEO_CB {
            let fb = core.pixel_framebuffer();
            video(
                fb.as_ptr().cast(),
                video::PIXEL_WIDTH as u32,
                video::PIXEL_HEIGHT as u32,
                (video::PIXEL_WIDTH * 4) as usize,
            );
        }
        if let Some(audio) = AUDIO_BATCH_CB {
            let samples = core.audio_frame();
            audio(samples.as_ptr(), sound::FRAMES_PER_VIDEO_FRAME);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_region() -> u32 {
    RETRO_REGION_NTSC
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_memory_data(_id: u32) -> *mut c_void {
    core::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_memory_size(_id: u32) -> usize {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_serialize_size() -> usize {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_serialize(_data: *mut c_void, _size: usize) -> bool {
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_unserialize(_data: *const c_void, _size: usize) -> bool {
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_cheat_reset() {}

#[unsafe(no_mangle)]
pub extern "C" fn retro_cheat_set(_index: u32, _enabled: bool, _code: *const i8) {}

#[unsafe(no_mangle)]
pub extern "C" fn retro_load_game_special(
    _game_type: u32,
    _info: *const retro_game_info,
    _num_info: usize,
) -> bool {
    false
}

fn read_inputs(input: retro_input_state_t) -> [[u8; 2]; sdk::PLAYER_COUNT] {
    let mut pads = [[0u8; 2]; sdk::PLAYER_COUNT];
    for (port, pad) in pads.iter_mut().enumerate() {
        for (bit, id) in retro::JOYPAD_IDS.iter().copied().enumerate() {
            if input(port as u32, RETRO_DEVICE_JOYPAD, 0, id) != 0 {
                pad[bit / 8] |= 1 << (bit % 8);
            }
        }
    }
    pads
}
