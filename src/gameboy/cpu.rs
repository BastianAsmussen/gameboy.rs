const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

#[derive(Debug)]
struct FlagsRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
}

impl std::convert::From<FlagsRegister> for u8 {
    fn from(flags: FlagsRegister) -> Self {
        Self::from(flags.zero) << ZERO_FLAG_BYTE_POSITION
            | Self::from(flags.subtract) << SUBTRACT_FLAG_BYTE_POSITION
            | Self::from(flags.half_carry) << HALF_CARRY_FLAG_BYTE_POSITION
            | Self::from(flags.carry) << CARRY_FLAG_BYTE_POSITION
    }
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(byte: u8) -> Self {
        let zero = ((byte >> ZERO_FLAG_BYTE_POSITION) & 0b1) != 0;
        let subtract = ((byte >> SUBTRACT_FLAG_BYTE_POSITION) & 0b1) != 0;
        let half_carry = ((byte >> HALF_CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
        let carry = ((byte >> CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;

        Self {
            zero,
            subtract,
            half_carry,
            carry,
        }
    }
}

#[derive(Debug)]
pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: FlagsRegister,
    h: u8,
    l: u8,
}

impl Registers {
    pub const fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    pub const fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }
}

#[derive(Debug)]
enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug)]
enum IncDecTarget {
    BC,
    DE,
}

#[derive(Debug)]
enum PrefixTarget {
    B,
}

#[derive(Debug)]
enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

#[derive(Debug)]
enum Instruction {
    Add(ArithmeticTarget),
    Inc(IncDecTarget),
    Rlc(PrefixTarget),
    Jp(JumpTest),
}

impl Instruction {
    const fn from_byte(byte: u8, is_prefixed: bool) -> Option<Self> {
        if is_prefixed {
            Self::from_prefixed_byte(byte)
        } else {
            Self::from_normal_byte(byte)
        }
    }

    const fn from_prefixed_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::Rlc(PrefixTarget::B)),
            _ => None,
        }
    }

    const fn from_normal_byte(byte: u8) -> Option<Self> {
        let instruction = match byte {
            0x02 => Self::Inc(IncDecTarget::BC),
            0x13 => Self::Inc(IncDecTarget::DE),
            _ => return None,
        };

        Some(instruction)
    }
}

#[derive(Debug)]
struct MemoryBus {
    memory: [u8; 0xFFFF],
}

impl MemoryBus {
    const fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }
}

#[derive(Debug)]
struct Cpu {
    registers: Registers,
    pc: u16,
    bus: MemoryBus,
}

impl Cpu {
    fn step(&mut self) {
        let mut instruction_byte = self.bus.read_byte(self.pc);

        let is_prefixed = instruction_byte == 0xCB;
        if is_prefixed {
            instruction_byte = self.bus.read_byte(self.pc + 1);
        }

        let next_pc = Instruction::from_byte(instruction_byte, is_prefixed).map_or_else(
            || {
                let description = format!(
                    "0x{}{instruction_byte:X}",
                    if is_prefixed { "CB" } else { "" },
                );

                panic!("Unknown instruction found! ({description})")
            },
            |instruction| self.execute(instruction),
        );

        self.pc = next_pc;
    }

    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::Add(target) => match target {
                ArithmeticTarget::C => {
                    let value = self.registers.c;
                    let new_value = self.add(value);
                    self.registers.a = new_value;

                    self.pc.wrapping_add(1)
                }
                _ => self.pc,
            },
            Instruction::Jp(jump_test) => {
                let should_jump = match jump_test {
                    JumpTest::NotZero => !self.registers.f.zero,
                    JumpTest::NotCarry => !self.registers.f.carry,
                    JumpTest::Zero => self.registers.f.zero,
                    JumpTest::Carry => self.registers.f.carry,
                    JumpTest::Always => true,
                };

                self.jump(should_jump)
            }
            _ => self.pc,
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);

        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;

        new_value
    }

    fn jump(&self, should_jump: bool) -> u16 {
        if should_jump {
            let least_significant_byte = u16::from(self.bus.read_byte(self.pc + 1));
            let most_significant_byte = u16::from(self.bus.read_byte(self.pc + 2));

            (most_significant_byte << 8) | least_significant_byte
        } else {
            self.pc.wrapping_add(3)
        }
    }
}
