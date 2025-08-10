use rand::Rng;

pub struct Device{
    pub pc: u16,
    pub stack: [u16; 48],
    pub stack_pointer: u16,
    pub instruction_pointer: u16,

    pub memory: [u8; 4096],
    pub registers: [u8; 16],
    pub keyboard: [bool; 16],
    pub display: [[u8; 32]; 64],
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub display_changed: bool,
}

impl Device{
    pub fn new() -> Self{
        Device{
            pc: 0x200,
            stack: [0; 48],
            stack_pointer: 0,
            instruction_pointer: 0,

            memory: [0; 4096],
            registers: [0; 16],
            keyboard: [false; 16],
            display: [[0;32];64],
            delay_timer: 0,
            sound_timer: 0,
            display_changed: false,
        }
    }

    pub fn fetch(&mut self) -> u16{
        let instruction = ((self.memory[self.pc as usize] as u16) << 8)
                          | ((self.memory[(self.pc+1) as usize]) as u16);
        self.pc+=2; //increment program counter by 2

        
        instruction
    }

    pub fn decode(&mut self, instruction: u16) {

        match instruction & 0xF000 {
            0x0000 => {
                let operation = instruction & 0x0FFF;
                match operation{
                    0x0E0 => self.clear_display(),
                    0x0EE => {
                        self.return_from_subroutine();
                    }
                    _ =>{
                        println!("Operation does not exist for 0x00{:02X}", operation);
                    }
                }
            }

            0x1000 => {
                let addr: u16 = instruction & 0x0FFF;
                self.pc = addr;
            }

            0x2000 => {
                let addr:u16 = instruction & 0xFFF;
                self.call_subroutine(addr);
            }

            0x3000 => {
                let reg_x = ((instruction >> 8) & 0x0F) as usize;
                let value = (instruction & 0xFF) as u8;
                if self.registers[reg_x] == value{
                    self.pc+=2;
                }
            }

            0x4000 => {
                let reg_x = ((instruction >> 8) & 0x0F) as usize;
                let value = (instruction & 0xFF) as u8;

                if self.registers[reg_x] != value{
                    self.pc +=2;
                }
            }

            0x5000 => {
                let reg_x = ((instruction >> 8) & 0x0F) as usize;
                let reg_y = ((instruction >> 4) & 0x0F) as usize;

                if self.registers[reg_x] == self.registers[reg_y] {
                    self.pc+=2;
                }
            }

            0x6000 => {
                let reg_x = ((instruction >> 8) & 0x0F) as usize;
                let value = (instruction & 0xFF) as u8;
                self.registers[reg_x] = value;
            }

            0x7000 => {
                let reg_x = ((instruction >> 8) & 0x0F) as usize;
                let value = (instruction & 0xFF) as u8;
                self.registers[reg_x] = self.registers[reg_x].wrapping_add(value);
            }

            0x8000 => {
                let reg_x = ((instruction & 0x0F00) >> 8) as usize;
                let reg_y = ((instruction & 0x00F0) >> 4) as usize;

                match instruction & 0x000F {
                    0x0 => {
                        self.registers[reg_x] = self.registers[reg_y];
                    }
                    0x1 => {
                        self.registers[reg_x] |= self.registers[reg_y];
                    }
                    0x2 => {
                        self.registers[reg_x] &= self.registers[reg_y];
                    }
                    0x3 => {
                        self.registers[reg_x] ^= self.registers[reg_y];
                    }
                    0x4 => {
                        let (result, carry) = self.registers[reg_x].overflowing_add(self.registers[reg_y]);
                        self.registers[reg_x] = result;
                        self.registers[0xF] = carry as u8;
                    }
                    0x5 => {
                        self.registers[0xF] = (self.registers[reg_x] > self.registers[reg_y]) as u8;
                        self.registers[reg_x] = self.registers[reg_x].wrapping_sub(self.registers[reg_y]);
                    }
                    0x6 => {
                        self.registers[0xF] = self.registers[reg_x] & 0x1;
                        self.registers[reg_x] >>= 1;
                    }
                    0x7 => {
                        self.registers[0xF] = (self.registers[reg_y] > self.registers[reg_x]) as u8;
                        self.registers[reg_x] = self.registers[reg_y].wrapping_sub(self.registers[reg_x]);
                    }
                    0xE => {
                        self.registers[0xF] = (self.registers[reg_x] >> 7) & 0x1;
                        self.registers[reg_x] <<= 1;
                    }
                    _ => {
                        println!("Operation does not exist for 0x8XY{:02X}", instruction & 0x000F);
                    } 
                }
            }

            0x9000 => {
                let reg_x = ((instruction >> 8) & 0x0F) as usize;
                let reg_y = ((instruction >> 4) & 0x0F) as usize;

                if self.registers[reg_x] != self.registers[reg_y] {
                    self.pc +=2;
                }
            }

            0xA000 => {
                let addr = instruction & 0x0FFF;
                self.instruction_pointer = addr;
            }

            0xB000 => {
                let addr = instruction & 0x0FFF;
                self.pc = self.registers[0] as u16 + addr;
            }

            0xC000 => {
                let reg_x = ((instruction >> 8) & 0x0F) as usize;
                let value: u8 = (instruction & 0xFF) as u8;
                
                let mut rng = rand::thread_rng();
                let random_value: u8 = rng.gen_range(0..=255);

                self.registers[reg_x] = value & random_value;
            }

            0xD000 => {
                let reg_x = (instruction >> 8) & 0x0F;
                let reg_y = (instruction >> 4) & 0x0F;
                let height = instruction & 0x000F;

                let x_coord = self.registers[reg_x as usize];
                let y_coord = self.registers[reg_y as usize];

                self.draw_sprite(x_coord,y_coord,height as u8);

            }

            0xE000 => {
                let reg_x = ((instruction >> 8) & 0x0F) as usize;
                let operation = instruction & 0x00FF;
                let key = self.registers[reg_x] as usize; //get lowest nibble of value in VX

                if operation == 0x9E{
                    if self.keyboard[key] {
                        self.pc+=2;
                    }
                }
                else if operation == 0xA1 {
                    if !self.keyboard[key]{
                        self.pc+=2;
                    }
                }
                
            }

           0xF000 => {
                let reg_x = ((instruction & 0x0F00) >> 8) as usize;

                match instruction & 0x00FF {
                    0x07 => {
                        self.registers[reg_x] = self.delay_timer;
                    }
                    0x0A => {
                        self.registers[reg_x] = 0xFF;

                        for i in 0..16{
                            if self.keyboard[i]{
                                self.registers[reg_x] = i as u8;
                                break;
                            }
                        }

                        if self.registers[reg_x] == 0xFF{
                            self.pc -= 2;
                        }
                             
                    }
                    0x15 => {
                        self.delay_timer = self.registers[reg_x];
                    }
                    0x18 => {
                        self.sound_timer = self.registers[reg_x];
                    }
                    0x1E => {
                        self.instruction_pointer = self.instruction_pointer.wrapping_add(self.registers[reg_x] as u16);
                    }
                    0x29 => {
                        self.instruction_pointer = (self.registers[reg_x] as u16) * 5;
                    }
                    0x33 => {
                        let vx = self.registers[reg_x];
                        self.memory[self.instruction_pointer as usize] = vx / 100;
                        self.memory[self.instruction_pointer as usize + 1] = (vx / 10) % 10;
                        self.memory[self.instruction_pointer as usize + 2] = vx % 10;
                    }
                    0x55 => {
                        for i in 0..=reg_x {
                            self.memory[self.instruction_pointer as usize + i] = self.registers[i];
                        }
                    }
                    0x65 => {
                        for i in 0..=reg_x {
                            self.registers[i] = self.memory[self.instruction_pointer as usize + i];
                        }
                    }
                    _ => {
                        println!("Operation does not exist for 0xFX{:02X}", instruction & 0x000F);
                    }
                }
            } 

            _ => {
                println!("No such instruction 0x{:04X}", instruction);
            }
        }
    }

   fn call_subroutine(&mut self, address: u16){
        //stores the program counter in the stack
        //increments stack pointer by 1 
        //and sets value of program counter to the address of the subroutine
        if self.stack_pointer < 48{
            self.stack[self.stack_pointer as usize] = self.pc;
            self.stack_pointer += 1;
            self.pc = address;

        }
        else{
            panic!("Stack overflow!!");
        }
   }

   fn return_from_subroutine(&mut self){
       //decrement stack pointer and
       //retrieve value of program counter from the stack
       if self.stack_pointer > 0{
           self.stack_pointer -= 1;
           self.pc = self.stack[self.stack_pointer as usize];
       }
       else{
           panic!("Stack underflow!! sp= {}", self.stack_pointer);
       }
   }

    pub fn update_timers(&mut self){
        //decrement timers by one, no much more
        if self.delay_timer > 0{
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0{
            self.sound_timer -= 1;
        }
    }

    fn clear_display(&mut self){
        //reset all the pixels in the display to the default value
        for x in 0..64{
            for y in 0..32{
                self.display[x as usize][y as usize] = 0;
            }
        }
        self.display_changed = true;
    }

    fn draw_sprite(&mut self, x: u8, y: u8, height: u8){
        //draw sprite of 8xheight at x,y in the display
        //by XORing every pixel with 1
        //if a bit was set before XORing, set collision flag (register 15) to 1
        
        //reset collision flagh
        self.registers[0xF] = 0;

        for row in 0..height{
            let sprite_index = self.instruction_pointer + (row as u16);
            let sprite_data = self.memory[sprite_index as usize];

            for col in 0..8{
               
               //check if pixel is set for this sprite
               // 0x80 >> col moves 1 to the column bit position
               if (sprite_data & (0x80 >> col)) != 0{ 

                   //make sure pixel coordinates are inside valid display resolution
                   let pixel_x = ((x as u16) + (col as u16)) % 64;
                   let pixel_y = ((y as u16) + (row as u16)) % 32;

                   //check collision before modifying the pixel
                   if self.display[pixel_x as usize][pixel_y as usize] == 1{
                       self.registers[0xF] = 1;
                   }

                   self.display[pixel_x as usize][pixel_y as usize] ^= 1;
               } 
            }
        }
        self.display_changed = true;
    }
}
