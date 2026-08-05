use ez80::{Cpu, FastBus};

use crate::{sdk, sound, video};

const MEM_SIZE: usize = 0x01_000000;
const CYCLES_PER_FRAME: u64 = 200_000;
const CART_MAGIC: &[u8; 4] = b"EZRA";
const HEADER_ENTRY_OFFSET: usize = 0x08;
const HEADER_STACK_OFFSET: usize = 0x0B;

pub struct Console {
    cpu: Cpu,
    bus: Bus,
    pixels: Box<[u32; video::PIXELS]>,
    audio: [i16; sound::STEREO_SAMPLES],
}

impl Console {
    pub fn new() -> Self {
        let mut cpu = Cpu::new_ez80();
        cpu.state.reg.adl = true;
        cpu.state.set_pc(sdk::PROGRAM_LOAD_ADDR);
        cpu.state.reg.set24(ez80::Reg16::SP, sdk::STACK_TOP);
        let mut console = Self {
            cpu,
            bus: Bus::new(),
            pixels: Box::new([0; video::PIXELS]),
            audio: [0; sound::STEREO_SAMPLES],
        };
        console.bus.mem[sdk::FRAMEBUFFER_ADDR as usize..][..sdk::FB_SIZE].fill(b' ');
        console.bus.capture_frame();
        console.present();
        console
    }

    pub fn load_program(&mut self, data: &[u8]) {
        let start = sdk::PROGRAM_LOAD_ADDR as usize;
        let available = self.bus.mem.len().saturating_sub(start);
        let len = data.len().min(available);
        self.bus.mem[start..start + len].copy_from_slice(&data[..len]);
        self.cpu
            .state
            .set_pc(cartridge_addr24(data, HEADER_ENTRY_OFFSET).unwrap_or(sdk::PROGRAM_LOAD_ADDR));
        if let Some(stack_top) = cartridge_addr24(data, HEADER_STACK_OFFSET) {
            self.cpu.state.reg.set24(ez80::Reg16::SP, stack_top);
        }
    }

    pub fn set_inputs(&mut self, inputs: [[u8; 2]; sdk::PLAYER_COUNT]) {
        self.bus.inputs = inputs;
    }

    pub fn run_frame(&mut self) {
        self.bus.presented = false;
        self.cpu.run_cycles(&mut self.bus, CYCLES_PER_FRAME);
        if self.bus.presented {
            self.present();
        }
        self.mix_audio();
        self.bus.tick = self.bus.tick.wrapping_add(1);
    }

    pub fn pixel_framebuffer(&self) -> &[u32] {
        &self.pixels[..]
    }

    pub fn audio_frame(&self) -> &[i16] {
        &self.audio
    }

    fn present(&mut self) {
        video::render(&self.bus.presented_frame[..], &mut self.pixels[..]);
    }

    fn mix_audio(&mut self) {
        let wave = &sound::SOUND_TABLE[self.bus.sound as usize];
        for (frame, sample) in wave.iter().copied().enumerate() {
            self.audio[frame * 2] = sample;
            self.audio[frame * 2 + 1] = sample;
        }
        self.bus.sound = 0;
    }
}

fn cartridge_addr24(data: &[u8], offset: usize) -> Option<u32> {
    if data.len() < offset + 3 || data.get(..4) != Some(CART_MAGIC) {
        return None;
    }

    Some(
        (data[offset] as u32)
            | ((data[offset + 1] as u32) << 8)
            | ((data[offset + 2] as u32) << 16),
    )
}

struct Bus {
    mem: Vec<u8>,
    inputs: [[u8; 2]; sdk::PLAYER_COUNT],
    sound: u8,
    tick: u8,
    presented: bool,
    presented_frame: Box<[u8; sdk::FB_SIZE]>,
    cycles: u64,
}

impl Bus {
    fn new() -> Self {
        Self {
            mem: vec![0; MEM_SIZE],
            inputs: [[0; 2]; sdk::PLAYER_COUNT],
            sound: 0,
            tick: 0,
            presented: false,
            presented_frame: Box::new([0; sdk::FB_SIZE]),
            cycles: 0,
        }
    }

    fn capture_frame(&mut self) {
        let start = sdk::FRAMEBUFFER_ADDR as usize;
        self.presented_frame
            .copy_from_slice(&self.mem[start..start + sdk::FB_SIZE]);
        self.presented = true;
    }
}

impl FastBus for Bus {
    fn read8(&mut self, addr: u32) -> u8 {
        self.mem[addr as usize & (MEM_SIZE - 1)]
    }

    fn write8(&mut self, addr: u32, value: u8) {
        self.mem[addr as usize & (MEM_SIZE - 1)] = value;
    }

    fn input8(&mut self, port: u16) -> u8 {
        match port {
            sdk::PORT_TICK => self.tick,
            sdk::PORT_P1_LOW => self.inputs[0][0],
            sdk::PORT_P1_HIGH => self.inputs[0][1],
            sdk::PORT_P2_LOW => self.inputs[1][0],
            sdk::PORT_P2_HIGH => self.inputs[1][1],
            sdk::PORT_P3_LOW => self.inputs[2][0],
            sdk::PORT_P3_HIGH => self.inputs[2][1],
            sdk::PORT_P4_LOW => self.inputs[3][0],
            sdk::PORT_P4_HIGH => self.inputs[3][1],
            _ => 0,
        }
    }

    fn output8(&mut self, port: u16, value: u8) {
        match port {
            sdk::PORT_PRESENT => self.capture_frame(),
            sdk::PORT_SOUND => self.sound = value,
            _ => {}
        }
    }

    fn add_cycles(&mut self, cycles: u32) {
        self.cycles = self.cycles.wrapping_add(cycles as u64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn present_captures_frame_at_port_write() {
        let mut bus = Bus::new();
        let start = sdk::FRAMEBUFFER_ADDR as usize;
        bus.mem[start] = b'A';

        bus.output8(sdk::PORT_PRESENT, 0);
        bus.mem[start] = b'B';

        assert!(bus.presented);
        assert_eq!(bus.presented_frame[0], b'A');
    }

    #[test]
    fn latest_present_replaces_previous_snapshot() {
        let mut bus = Bus::new();
        let start = sdk::FRAMEBUFFER_ADDR as usize;
        bus.mem[start] = b'A';
        bus.output8(sdk::PORT_PRESENT, 0);

        bus.mem[start] = b'B';
        bus.output8(sdk::PORT_PRESENT, 0);

        assert_eq!(bus.presented_frame[0], b'B');
    }
}
