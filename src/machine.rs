use ez80::{Cpu, CpuMode, FastBus, Reg16};

use crate::{sdk, sound, video};

const MEM_SIZE_16: usize = 0x01_0000;
const MEM_SIZE_EZ80: usize = 0x01_000000;
const CYCLES_PER_FRAME: u64 = 200_000;
const CART_MAGIC: &[u8; 4] = b"EZRA";
const CART_PREFIX_SIZE: usize = CART_MAGIC.len() + 1;

#[derive(Clone, Copy)]
struct MachineConfig {
    cpu_mode: CpuMode,
    mem_size: usize,
    load_addr: u32,
    stack_top: u32,
    framebuffer_addr: u32,
}

impl MachineConfig {
    fn from_cart_id(id: u8) -> Option<Self> {
        let cpu_mode = match id {
            0 => CpuMode::I8080,
            1 => CpuMode::I8085,
            2 => CpuMode::Z80,
            3 => CpuMode::Z80N,
            4 => CpuMode::Z180,
            5 => CpuMode::EZ80,
            _ => return None,
        };
        let is_ez80 = cpu_mode == CpuMode::EZ80;
        Some(Self {
            cpu_mode,
            mem_size: if is_ez80 { MEM_SIZE_EZ80 } else { MEM_SIZE_16 },
            load_addr: if is_ez80 {
                sdk::PROGRAM_LOAD_ADDR_EZ80
            } else {
                sdk::PROGRAM_LOAD_ADDR_16
            },
            stack_top: if is_ez80 {
                sdk::STACK_TOP_EZ80
            } else {
                sdk::STACK_TOP_16
            },
            framebuffer_addr: if is_ez80 {
                sdk::FRAMEBUFFER_ADDR_EZ80
            } else {
                sdk::FRAMEBUFFER_ADDR_16
            },
        })
    }
}

pub struct Console {
    cpu: Cpu,
    bus: Bus,
    pixels: Box<[u32; video::PIXELS]>,
    audio: [i16; sound::STEREO_SAMPLES],
    config: MachineConfig,
    program: Vec<u8>,
}

impl Console {
    pub fn new() -> Self {
        Self::for_config(MachineConfig::from_cart_id(5).expect("eZ80 config is defined"))
    }

    fn for_config(config: MachineConfig) -> Self {
        let mut cpu = Cpu::new_for_mode(config.cpu_mode);
        cpu.state.reg.adl = config.cpu_mode == CpuMode::EZ80;
        cpu.state.set_pc(config.load_addr);
        if config.cpu_mode == CpuMode::EZ80 {
            cpu.state.reg.set24(Reg16::SP, config.stack_top);
        } else {
            cpu.state.reg.set16(Reg16::SP, config.stack_top as u16);
        }
        let mut console = Self {
            cpu,
            bus: Bus::new(config.mem_size, config.framebuffer_addr),
            pixels: Box::new([0; video::PIXELS]),
            audio: [0; sound::STEREO_SAMPLES],
            config,
            program: Vec::new(),
        };
        let framebuffer = config.framebuffer_addr as usize;
        console.bus.mem[framebuffer..framebuffer + sdk::FB_SIZE].fill(b' ');
        console.bus.capture_frame();
        console.present();
        console
    }

    pub fn load_program(&mut self, data: &[u8]) -> bool {
        if data.len() < CART_PREFIX_SIZE || data.get(..CART_MAGIC.len()) != Some(CART_MAGIC) {
            return false;
        }
        let Some(config) = MachineConfig::from_cart_id(data[CART_MAGIC.len()]) else {
            return false;
        };
        let payload = &data[CART_PREFIX_SIZE..];
        let start = config.load_addr as usize;
        if payload.len() > config.mem_size.saturating_sub(start) {
            return false;
        }

        *self = Self::for_config(config);
        self.bus.mem[start..start + payload.len()].copy_from_slice(payload);
        self.program.extend_from_slice(payload);
        true
    }

    pub fn reset(&mut self) {
        let config = self.config;
        let program = self.program.clone();
        *self = Self::for_config(config);
        let start = config.load_addr as usize;
        self.bus.mem[start..start + program.len()].copy_from_slice(&program);
        self.program = program;
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
        video::render(
            &self.bus.presented_frame[..],
            &mut self.pixels[..],
            self.bus.presented_text_color,
            self.bus.presented_background_color,
            self.bus.presented_font,
        );
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

struct Bus {
    mem: Vec<u8>,
    inputs: [[u8; 2]; sdk::PLAYER_COUNT],
    sound: u8,
    tick: u8,
    presented: bool,
    presented_frame: Box<[u8; sdk::FB_SIZE]>,
    text_color: u8,
    background_color: u8,
    presented_text_color: u8,
    presented_background_color: u8,
    font: u8,
    presented_font: u8,
    framebuffer_addr: usize,
    cycles: u64,
}

impl Bus {
    fn new(mem_size: usize, framebuffer_addr: u32) -> Self {
        Self {
            mem: vec![0; mem_size],
            inputs: [[0; 2]; sdk::PLAYER_COUNT],
            sound: 0,
            tick: 0,
            presented: false,
            presented_frame: Box::new([0; sdk::FB_SIZE]),
            text_color: sdk::DEFAULT_TEXT_COLOR,
            background_color: sdk::DEFAULT_BACKGROUND_COLOR,
            presented_text_color: sdk::DEFAULT_TEXT_COLOR,
            presented_background_color: sdk::DEFAULT_BACKGROUND_COLOR,
            font: sdk::FONT_VGA_8X8,
            presented_font: sdk::FONT_VGA_8X8,
            framebuffer_addr: framebuffer_addr as usize,
            cycles: 0,
        }
    }

    fn capture_frame(&mut self) {
        self.presented_frame.copy_from_slice(
            &self.mem[self.framebuffer_addr..self.framebuffer_addr + sdk::FB_SIZE],
        );
        self.presented_text_color = self.text_color;
        self.presented_background_color = self.background_color;
        self.presented_font = self.font;
        self.presented = true;
    }
}

impl FastBus for Bus {
    fn read8(&mut self, addr: u32) -> u8 {
        self.mem[addr as usize & (self.mem.len() - 1)]
    }

    fn write8(&mut self, addr: u32, value: u8) {
        let addr = addr as usize & (self.mem.len() - 1);
        self.mem[addr] = value;
    }

    fn input8(&mut self, port: u16) -> u8 {
        match port & 0x00FF {
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
        match port & 0x00FF {
            sdk::PORT_PRESENT => self.capture_frame(),
            sdk::PORT_TEXT_COLOR => self.text_color = value,
            sdk::PORT_BACKGROUND_COLOR => self.background_color = value,
            sdk::PORT_FONT if value < sdk::FONT_COUNT => self.font = value,
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
        let mut bus = Bus::new(MEM_SIZE_EZ80, sdk::FRAMEBUFFER_ADDR_EZ80);
        let start = sdk::FRAMEBUFFER_ADDR_EZ80 as usize;
        bus.mem[start] = b'A';

        bus.output8(sdk::PORT_PRESENT, 0);
        bus.mem[start] = b'B';

        assert!(bus.presented);
        assert_eq!(bus.presented_frame[0], b'A');
    }

    #[test]
    fn latest_present_replaces_previous_snapshot() {
        let mut bus = Bus::new(MEM_SIZE_EZ80, sdk::FRAMEBUFFER_ADDR_EZ80);
        let start = sdk::FRAMEBUFFER_ADDR_EZ80 as usize;
        bus.mem[start] = b'A';
        bus.output8(sdk::PORT_PRESENT, 0);

        bus.mem[start] = b'B';
        bus.output8(sdk::PORT_PRESENT, 0);

        assert_eq!(bus.presented_frame[0], b'B');
    }

    #[test]
    fn present_captures_text_background_and_font() {
        let mut bus = Bus::new(MEM_SIZE_EZ80, sdk::FRAMEBUFFER_ADDR_EZ80);
        bus.output8(sdk::PORT_TEXT_COLOR, 196);
        bus.output8(sdk::PORT_BACKGROUND_COLOR, 21);
        bus.output8(sdk::PORT_FONT, sdk::FONT_C64);
        bus.output8(sdk::PORT_PRESENT, 0);

        bus.output8(sdk::PORT_TEXT_COLOR, 46);
        bus.output8(sdk::PORT_BACKGROUND_COLOR, 232);
        bus.output8(sdk::PORT_FONT, sdk::FONT_BBC_MICRO);

        assert_eq!(bus.presented_text_color, 196);
        assert_eq!(bus.presented_background_color, 21);
        assert_eq!(bus.presented_font, sdk::FONT_C64);
    }

    #[test]
    fn cartridge_cpu_byte_selects_mode_memory_and_entry() {
        for (id, mode, mem_size, load_addr, framebuffer_addr) in [
            (0, CpuMode::I8080, MEM_SIZE_16, 0, sdk::FRAMEBUFFER_ADDR_16),
            (1, CpuMode::I8085, MEM_SIZE_16, 0, sdk::FRAMEBUFFER_ADDR_16),
            (2, CpuMode::Z80, MEM_SIZE_16, 0, sdk::FRAMEBUFFER_ADDR_16),
            (3, CpuMode::Z80N, MEM_SIZE_16, 0, sdk::FRAMEBUFFER_ADDR_16),
            (4, CpuMode::Z180, MEM_SIZE_16, 0, sdk::FRAMEBUFFER_ADDR_16),
            (
                5,
                CpuMode::EZ80,
                MEM_SIZE_EZ80,
                sdk::PROGRAM_LOAD_ADDR_EZ80,
                sdk::FRAMEBUFFER_ADDR_EZ80,
            ),
        ] {
            let mut console = Console::new();
            let cart = [b'E', b'Z', b'R', b'A', id, 0x76];

            assert!(console.load_program(&cart));
            assert_eq!(console.cpu.mode(), mode);
            assert_eq!(console.cpu.state.pc(), load_addr);
            assert_eq!(console.bus.mem.len(), mem_size);
            assert_eq!(console.bus.mem[load_addr as usize], 0x76);
            assert_eq!(console.bus.framebuffer_addr, framebuffer_addr as usize);
            assert_eq!(console.cpu.state.reg.adl, mode == CpuMode::EZ80);
        }
    }

    #[test]
    fn cartridge_requires_magic_and_known_cpu() {
        let mut console = Console::new();

        assert!(!console.load_program(b"raw"));
        assert!(!console.load_program(b"EZRA\x06"));
    }

    #[test]
    fn reset_restarts_the_loaded_cartridge() {
        let mut console = Console::new();
        let cart = [b'E', b'Z', b'R', b'A', 2, 0x76];

        assert!(console.load_program(&cart));
        console.bus.mem[sdk::FRAMEBUFFER_ADDR_16 as usize] = b'X';
        console.bus.tick = 42;
        console.reset();

        assert_eq!(console.cpu.state.pc(), sdk::PROGRAM_LOAD_ADDR_16);
        assert_eq!(console.bus.tick, 0);
        assert_eq!(console.bus.mem[sdk::PROGRAM_LOAD_ADDR_16 as usize], 0x76);
        assert_eq!(console.bus.mem[sdk::FRAMEBUFFER_ADDR_16 as usize], b' ');
    }

    #[test]
    fn console_ports_decode_the_low_byte_of_z80_io_addresses() {
        let mut bus = Bus::new(MEM_SIZE_16, sdk::FRAMEBUFFER_ADDR_16);
        bus.tick = 0x42;
        bus.mem[sdk::FRAMEBUFFER_ADDR_16 as usize] = b'X';

        assert_eq!(bus.input8(0xAB40), 0x42);
        bus.output8(0xAB11, 196);
        bus.output8(0xAB12, 21);
        bus.output8(0xAB13, sdk::FONT_C16);
        bus.output8(0x0110, 1);

        assert!(bus.presented);
        assert_eq!(bus.presented_text_color, 196);
        assert_eq!(bus.presented_background_color, 21);
        assert_eq!(bus.presented_font, sdk::FONT_C16);
        assert_eq!(bus.presented_frame[0], b'X');
    }

    #[test]
    fn z80_cartridge_reaches_video_and_tick_ports() {
        let mut console = Console::new();
        let cart = [
            b'E', b'Z', b'R', b'A', 2, // Z80 cartridge prefix
            0x21, 0x00, 0xE0, // LD HL,$E000
            0x36, b'X', // LD (HL),'X'
            0x3E, 0x01, // LD A,1
            0xD3, 0x10, // OUT ($10),A
            0xDB, 0x40, // IN A,($40)
            0x76, // HALT
        ];

        assert!(console.load_program(&cart));
        console.run_frame();

        assert_eq!(console.bus.presented_frame[0], b'X');
        assert_eq!(console.bus.tick, 1);
    }
}
