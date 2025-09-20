use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE_BYTES: usize = 4096;
const NUM_V_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const START_ADDR: u16 = 0x200;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

type Opcode = u16;

pub struct Emulator {
    program_counter: u16,
    ram: [u8; RAM_SIZE_BYTES],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], // Pixels are either black or white
    v_regs: [u8; NUM_V_REGS],
    i_reg: u16,

    stack_ptr: u16,
    stack: [Opcode; STACK_SIZE],

    keys: [bool; NUM_KEYS], // Which keys are pressed

    delay_timer: u8,
    sound_timer: u8,
}

impl Emulator {
    pub fn new() -> Self {
        let mut emulator = Emulator {
            program_counter: START_ADDR,
            ram: [0; RAM_SIZE_BYTES],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_regs: [0; NUM_V_REGS],
            i_reg: 0,

            stack_ptr: 0,
            stack: [0; STACK_SIZE],

            keys: [false; NUM_KEYS],

            delay_timer: 0,
            sound_timer: 0,
        };

        // Load font sprites into RAM
        emulator.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        emulator
    }

    pub fn reset(&mut self) {
        self.program_counter = START_ADDR;
        self.ram = [0; RAM_SIZE_BYTES];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_regs = [0; NUM_V_REGS];
        self.i_reg = 0;

        self.stack_ptr = 0;
        self.stack = [0; STACK_SIZE];

        self.keys = [false; NUM_KEYS];

        self.delay_timer = 0;
        self.sound_timer = 0;

        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    // Load byte array from file contents into RAM
    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();

        self.ram[start..end].copy_from_slice(data);
    }

    pub fn tick(&mut self) {
        // Fetch
        let opcode = self.fetch();

        // Decode and execute
        self.execute(opcode);
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                //
            }

            self.sound_timer -= 1;
        }
    }

    // Uses Big Endian (MSB is at smaller address)
    fn fetch(&mut self) -> Opcode {
        let high_byte = self.ram[self.program_counter as usize] as u16;
        let low_byte = self.ram[(self.program_counter + 1) as usize] as u16;

        self.program_counter += 2;

        (high_byte << 8) | low_byte
    }

    fn push(&mut self, value: u16) {
        self.stack[self.stack_ptr as usize] = value;
        self.stack_ptr += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stack_ptr -= 1;
        self.stack[self.stack_ptr as usize]
    }

    fn execute(&mut self, op: Opcode) {
        // Extract each nibble from opcode
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // NOP: Do nothing
            (0, 0, 0, 0) => return,

            // CLS: Clear screen
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT],

            // RET: Return to address after done running subroutine
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.program_counter = ret_addr;
            }

            // JMP: Move PC to address encoded in last 3 nibbles (NNN)
            (1, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.program_counter = nnn;
            }

            // CALL: Save current PC, then jump to given NNN address (to start subroutine)
            (2, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.push(self.program_counter);
                self.program_counter = nnn;
            }

            // SKIP VX == NN: Skip next line if value in VREG X == NN
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;

                if self.v_regs[x] == nn {
                    self.program_counter += 2;
                }
            }

            // SKIP VX != NN: Skip next line if value in VREG X != NN
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;

                if self.v_regs[x] != nn {
                    self.program_counter += 2;
                }
            }

            // SKIP VX == VY: Skip next line if value in VREG X == value in VREG Y
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_regs[x] == self.v_regs[y] {
                    self.program_counter += 2;
                }
            }

            // VX = NN: Set VREG X to NN
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;

                self.v_regs[x] = nn;
            }

            // VX += NN: Add NN to VREG X
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;

                // Avoid panicking if we overflow with wrapping add
                self.v_regs[x] = self.v_regs[x].wrapping_add(nn);
            }

            // VX = VY: Set VREG X to value in VREG Y
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_regs[x] = self.v_regs[y]
            }

            // VX |= VY: Bitwise OR
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_regs[x] |= self.v_regs[y]
            }

            // VX &= VY: Bitwise AND
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_regs[x] &= self.v_regs[y]
            }

            // VX ^= VY: Bitwise XOR
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_regs[x] ^= self.v_regs[y]
            }

            // VX += VY: Add VREG Y to VREG X. Can overflow, in which case the carry flag of the flag register (16th VREG, or VF) is set to 1
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (sum, did_overflow) = self.v_regs[x].overflowing_add(self.v_regs[y]);

                self.v_regs[x] = sum;
                self.v_regs[0xF] = if did_overflow { 1 } else { 0 };
            }

            // VX -= VY: Subtract VREG Y from VREG X. Can underflow, in which case the VF register is set to 0
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (diff, did_underflow) = self.v_regs[x].overflowing_sub(self.v_regs[y]);

                self.v_regs[x] = diff;
                self.v_regs[0xF] = if did_underflow { 0 } else { 1 };
            }

            // VX >>= 1: Right shift value in VREG X by 1, storing the dropped rightmost bit into the VF register
            (8, _, _, 6) => {
                let x = digit2 as usize;

                let rightmost_bit = self.v_regs[x] & 1;

                self.v_regs[x] >>= 1;
                self.v_regs[0xF] = rightmost_bit;
            }

            // VX = VY - VX: Subtract the registers, but in the opposite order
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (diff, did_underflow) = self.v_regs[y].overflowing_sub(self.v_regs[x]);

                self.v_regs[x] = diff;
                self.v_regs[0xF] = if did_underflow { 0 } else { 1 };
            }

            // VX <<= 1: Left shift value in VREG X by 1, storing the dropped leftmost bit into the VF register
            (8, _, _, 0xE) => {
                let x = digit2 as usize;

                let leftmost_bit = self.v_regs[x] & 0x80;

                self.v_regs[x] <<= 1;
                self.v_regs[0xF] = leftmost_bit;
            }

            // SKIP VX != VY: Skip next line if value in VREG X != value in VREG Y
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_regs[x] != self.v_regs[y] {
                    self.program_counter += 2;
                }
            }

            // I = NNN: Set IREG (mostly used as pointer to address in RAM) to NNN
            (0xA, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.i_reg = nnn;
            }

            // JMP V0 + NNN: Move PC to value in V0 plus NNN
            (0xB, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.program_counter = (self.v_regs[0] as u16) + nnn;
            }

            // VX = rand() & NN: Set VREG X to the last 8 bits of a random integer
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;

                let rng: u8 = random();
                self.v_regs[x] = rng & nn;
            }

            // DRAW: Draw a sprite at X and Y coordinates of screen. The height of the sprite is given by the last nibble.
            // Sprites are stored row by row beginning at the address in IREG.
            // If any pixel is flipped, the VF is set.
            (0xD, _, _, _) => {
                let x_coord = self.v_regs[digit2 as usize] as u16;
                let y_coord = self.v_regs[digit3 as usize] as u16;

                let sprite_rows = digit4;

                let mut did_flip = false;

                for row_idx in 0..sprite_rows {
                    let addr = self.i_reg + row_idx as u16;
                    let pixels = self.ram[addr as usize]; // This u8 stores the 8 pixel bits for this row

                    for pixel_idx in 0..8 {
                        // Skip if the pixel bit is unset
                        if pixels & (0b1000_0000 >> pixel_idx) == 0 {
                            continue;
                        }

                        // Ensure sprites wrap around
                        let x = (x_coord + pixel_idx) as usize % SCREEN_WIDTH;
                        let y = (y_coord + row_idx) as usize % SCREEN_HEIGHT;

                        let screen_idx = SCREEN_WIDTH * y + x;

                        did_flip |= self.screen[screen_idx];
                        self.screen[screen_idx] ^= true; // Toggle the pixel
                    }
                }

                self.v_regs[0xF] = if did_flip { 1 } else { 0 };
            }

            // SKIP KEY PRESS: Skip if the input stored in VREG X is pressed
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let input_idx = self.v_regs[x];

                if self.keys[input_idx as usize] {
                    self.program_counter += 2;
                }
            }

            // SKIP KEY RELEASE: Skip if the input stored in VREG X is NOT pressed
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let input_idx = self.v_regs[x];

                if !self.keys[input_idx as usize] {
                    self.program_counter += 2;
                }
            }

            // VX = DT: Store value of delay timer in VREG X
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_regs[x] = self.delay_timer;
            }

            // WAIT KEY: Block until the player presses any key. The (lowest) pressed key is stored in VREG X.
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;

                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_regs[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                // Repeats this instruction
                if !pressed {
                    self.program_counter -= 2;
                }
            }

            // DT = VX: Store value in VREG X in delay timer
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.delay_timer = self.v_regs[x];
            }

            // DT = ST: Store value in VREG X in sound timer
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.sound_timer = self.v_regs[x];
            }

            // I += VX: Add value in VREG X to IREG. Wraps on overflow.
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                self.i_reg = self.i_reg.wrapping_add(self.v_regs[x] as u16);
            }

            // I = FONT: Set IREG to address of font sprite index in VREG X.
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let which_sprite = self.v_regs[x] as u16;
                self.i_reg = 0 + which_sprite * 5; // Sprites are 5 bytes each and stored starting at 0x0
            }

            // BCD: Store value in VREG X as a BCD in RAM. One byte is used for each digit in base 10, so this will always use 3 bytes.
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let raw = self.v_regs[x];

                let hundreds = (raw / 100) % 10;
                let tens = (raw / 10) % 10;
                let ones = raw % 10;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }

            // STORE V0 - VX: Stores values from VREG's 0 to X (inclusive) into RAM (starting from IREG address)
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i_addr = self.i_reg as usize;

                for i in 0..=x {
                    self.ram[i_addr + i] = self.v_regs[i];
                }
            }

            // LOAD V0 - VX: Loads values from RAM starting from IREG address into VREG's 0 to X (inclusive)
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i_addr = self.i_reg as usize;

                for i in 0..=x {
                    self.v_regs[i] = self.ram[i_addr + i];
                }
            }

            (_, _, _, _) => unimplemented!("Unimplemented opcode: {:x}", op),
        }
    }
}
