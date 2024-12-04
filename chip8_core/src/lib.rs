use rand::random;

const RAM_SIZE: usize = 4096;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const START_ADDR: u16 = 0x200;
const NUM_KEYS: usize = 16;
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

pub struct Emu {
    pc: u16,                                      //program counter
    ram: [u8; RAM_SIZE],                          //memory
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], //screen size
    v_reg: [u8; NUM_REGS],                        //v - registers
    i_reg: u16,                                   //i - register
    sp: u16,                                      // stack pointer
    stack: [u16; STACK_SIZE],                     // stack
    keys: [bool; NUM_KEYS],                       // key presses on the chip8
    dt: u8,                                       // delay timer
    st: u8,                                       // sound timer
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    fn push(&mut self, val: u16) {
        //push method for stack
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    // game code error when empty stack popped, so not handling the complete panic.
    fn pop(&mut self) -> u16 {
        //pop method for stack
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        //Fetch
        let op = self.fetch();
        //decode and execute
        self.execute(op);
    }

    fn execute(&mut self, op: u16) {
        //digit separation
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            //NOP
            //(Do nothing)
            (0, 0, 0, 0) => return,

            //CLS
            //(clears the screen)
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH];
            }

            //RET
            //(returns from a subroutine)
            (0, 0, 0xE, 0xE) => {
                let return_address = self.pop();
                self.pc = return_address;
            }

            //JMP NNN
            //(jump to the given address)
            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }

            //CALL NNN
            //(calling a function)
            (2, _, _, _) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }

            //3XNN
            //SKIP NEXT IF VX == NN
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }

            //4XNN
            //SKIP NEXT IF VX != NN
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }

            //5XY0
            //SKIP NEXT IF VX = VY
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }

            //6XNN
            //SET V[X] TO NN
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            }

            //7XNN
            // VX += NN
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] += nn;
            }

            //8XY0
            // VX = VY
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            }

            //8XY1
            //VX |= VY
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            }

            //8XY2
            //VX &= VY
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            }

            //8XY3
            //VX ^= VY
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            }

            //8XY4
            //VX += VY
            //VF CARRY FLAG IS USED WHEN CARRYING
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            //8XY5
            //VX -= VY
            //UNSET VF CARRY FLAG ON BORROW
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            //8XY6
            //VX >>= VY
            // VF CARRY FLAG STORES THE DROPPED OFF VALUE
            (8, _, _, 6) => {
                let x = digit2 as usize;
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            }

            //8XY7
            //VX = VY - VX
            // VF CARRY FLAG STORES THE BORROWED VALUE
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            //8XYE
            // VX <<= 1
            // OVERFLOW STORED IN VF
            (8, _, _, _) => {
                let x = digit2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            }

            //9XY0
            //SKIP ON VX != VY
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }

            //ANNN
            //(address pointer to RAM) I register initisialised I = NNN
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            }

            //BNNN
            //JMP TO V0 + NNN
            (0xB, _, _, _) => {
                let nnn = (op & 0xFFF) as u16;
                self.sp = (self.v_reg[0] as u16) + nnn;
            }

            //CXNN
            //Chip-8's RNG
            // VX = rand() & NN
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            }

            //DRAW SPRITES
            //DXYN
            //X AND Y ARE COORDINATES INTO V_REG AND N IS THE NUMBER OF ROWS. NUMBER OF COLUMNS PER ROW IS ALWAYS 8
            (0xD, _, _, _) => {
                //gets x and y coords
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;
                //number of rows
                let num_rows = digit4;
                //checks if pixels were flipped
                let mut flipped = false;
                //iterate over each row of sprite
                for y_line in 0..num_rows {
                    // row data stored at I register
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    //Iterate over each column in the row
                    for x_line in 0..8 {
                        // Sprites to wrap around screen
                        // Use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                            //get pixel index for 1d screen array
                            let idx = x + SCREEN_WIDTH * y;
                            // check for flipping
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                //if flipped, put in VF register
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            }

            //KEY PRESS SKIP
            //EX9E
            //if index stored in VX is pressed, then we have a SKIP
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            }

            //KEY NOT PRESS SKIP
            //EXA1
            //if index stored in VX is not pressed, then we have a SKIP
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            }

            //FX07
            //VX = DT
            // stores delay timer value into VX
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            }

            //FX0A
            //Waits for key press and loop endlessly until our condition of the key press becomes true
            //TODO: TRY IMPROVING THIS TO BE ASYNCHRONOUS INSTEAD OF LOOPING
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                //TODO: LEARN ABOUT MUTABILITY HERE AND FIND OUT WHY IT IS NECESSARY
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    //REDO OPCODE
                    self.pc -= 2;
                }
            }

            //FX15
            // SET Delay timer to a value from VX, ie. DT = VX
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.vz_reg[x];
            }

            //FX18
            // SET Sound timer to a value from VX, ie. DT = VX
            // same as above, but on sound timer
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.vz_reg[x];
            }

            //FX1E
            // I += VX
            // increment the I-register values
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }

            //explanation of the next opcode
            //ram address of font is just 5 times the value of the thing to be printed.
            //FX29
            //font at I
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            }

            //FX33
            //CONVERTS TO BINARY CODED DECIMAL FORMAT OF THE NUMBER STORED IN VX
            //TODO: find more fast and efficient BCD algorithms so I dont have to do floating point arithmetics
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundereds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }

            //FX55
            //first of the two isntructions that populates registers V0 to VX to RAM
            //STORE V0 to VX to RAM
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            }

            //FX65
            //first of the two instructions that populates registers V0 to VX from RAM
            //LOAD V0 to VX from RAM
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
            }

            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte; //convert to Big Endian
        self.pc += 2; //move ahead
        op
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            if self.st == 0 {
                //BEEP after this ( implement later )
            }
            self.st -= 1;
        }
    }

    //returns a pointer to screen buffer array
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    //key buffer array frontend API manipulator
    //TODO: In the future, I could handle the limit of 16 here and panic directly instead of handling this in the frontend
    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    //loads game code from file to RAM so it can be executed
    //TAKE A LIST OF BYTES AND WRITE TO RAM
    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    //TODO: completed to section 6.2 
}
