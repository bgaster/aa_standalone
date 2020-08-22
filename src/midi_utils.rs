//! midi

use std::fmt;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NoteSym {    
    A0 = 21,
    AS0 = 22,
    B0  = 23,

    C1  = 24,
    CS1 = 25,
    D1  = 26,
    DS1 = 27,
    E1  = 28,
    F1  = 29,
    FS1 = 30,
    G1  = 31,
    GS1 = 32,
    A1  = 33,
    AS1 = 34,
    B1  = 35,

    C2  = 36,
    CS2 = 37,
    D2  = 38,
    DS2 = 39,
    E2  = 40,
    F2  = 41,
    FS2 = 42,
    G2  = 43,
    GS2 = 44,
    A2  = 45,
    AS2 = 46,
    B2  = 47,

    C3  = 48,
    CS3 = 49,
    D3  = 50,
    DS3 = 51,
    E3  = 52,
    F3  = 53,
    FS3 = 54,
    G3  = 55,
    GS3 = 56,
    A3  = 57,
    AS3 = 58,
    B3  = 59,

    C4  = 60,
    CS4 = 61,
    D4  = 62,
    DS4 = 63,
    E4  = 64,
    F4  = 65,

    FS4 = 66,
    G4  = 67,
    GS4 = 68,
    A4  = 69,
    AS4 = 70,
    B4  = 71,

    C5  = 72,
    CS5 = 73,
    D5  = 74,
    DS5 = 75,
    E5  = 76,
    F5  = 77,
    FS5 = 78,
    G5  = 79,
    GS5 = 80,
    A5  = 81,
    AS5 = 82,
    B5  = 83,

    C6  = 84,
    CS6 = 85,
    D6  = 86,
    DS6 = 87,
    E6  = 88,
    F6  = 89,
    FS6 = 90,
    G6  = 91,
    GS6 = 92,
    A6  = 93,
    AS6 = 94,
    B6  = 95,

    C7  = 96,
    CS7 = 97,
    D7  = 98,
    DS7 = 99,
    E7  = 100,
    F7  = 101,
    FS7 = 102,
    G7  = 103,
    GS7 = 104,
    A7  = 105,
    AS7 = 106,
    B7  = 107,

    C8  = 108,

    None,
}

pub type Pitch = f32;

impl NoteSym {
    /// convert midi note to its corresponding frequency
    #[inline]
    pub fn to_freq(self) -> Pitch {
        2.0f32.powf( (self as i32 - 69) as Pitch / 12.0 ) * 440.0
    }

    /// convert midi note to an index within voices range, can then be used as 
    /// sample index, for example.
    #[inline]
    pub fn to_index(self, voices: u32) -> u32 {
        (12.0*(self.to_freq() / 130.81).log2()).round().abs() as u32 % voices
    }

    #[inline]
    pub fn freq_to_index(freq: f32, voices: u32) -> u32 {
        (12.0*(freq / 130.81).log2()).round().abs() as u32 % voices
    }

    /// convert midi note to its corresponding frequency, with explicit base tuning
    pub fn to_freq_tuning(self, tuning: Pitch) -> Pitch {
        2.0f32.powf( (self as i32 - 69) as f32 / 12.0 ) * tuning
    }

    pub fn from_u8(value: u8) -> NoteSym {
        match value {
            21 => NoteSym::A0,
            22 => NoteSym::AS0,
            23 => NoteSym::B0,

            24 => NoteSym::C1,
            25 => NoteSym::CS1,
            26 => NoteSym::D1,
            27 => NoteSym::DS1,
            28 => NoteSym::E1,
            29 => NoteSym::F1,
            30 => NoteSym::FS1,
            31 => NoteSym::G1,
            32 => NoteSym::GS1,
            33 => NoteSym::A1,
            34 => NoteSym::AS1,
            35 => NoteSym::B1,

            36 => NoteSym::C2,
            37 => NoteSym::CS2,
            38 => NoteSym::D2,
            39 => NoteSym::DS2,
            40 => NoteSym::E2,
            41 => NoteSym::F2,
            42 => NoteSym::FS2,
            43 => NoteSym::G2,
            44 => NoteSym::GS2,
            45 => NoteSym::A2,
            46 => NoteSym::AS2,
            47 => NoteSym::B2,

            48 => NoteSym::C3,
            49 => NoteSym::CS3,
            50 => NoteSym::D3,
            51 => NoteSym::DS3,
            52 => NoteSym::E3,
            53 => NoteSym::F3,
            54 => NoteSym::FS3,
            55 => NoteSym::G3,
            56 => NoteSym::GS3,
            57 => NoteSym::A3,
            58 => NoteSym::AS3,
            59 => NoteSym::B3,

            60 => NoteSym::C4,
            61 => NoteSym::CS4,
            62 => NoteSym::D4,
            63 => NoteSym::DS4,
            64 => NoteSym::E4,
            65 => NoteSym::F4,

            66 => NoteSym::FS4,
            67 => NoteSym::G4,
            68 => NoteSym::GS4,
            69 => NoteSym::A4,
            70 => NoteSym::AS4,
            71 => NoteSym::B4,

            72 => NoteSym::C5,
            73 => NoteSym::CS5,
            74 => NoteSym::D5,
            75 => NoteSym::DS5,
            76 => NoteSym::E5,
            77 => NoteSym::F5,
            78 => NoteSym::FS5,
            79 => NoteSym::G5,
            80 => NoteSym::GS5,
            81 => NoteSym::A5,
            82 => NoteSym::AS5,
            83 => NoteSym::B5,

            84 => NoteSym::C6,
            85 => NoteSym::CS6,
            86 => NoteSym::D6,
            87 => NoteSym::DS6,
            88 => NoteSym::E6,
            89 => NoteSym::F6,
            90 => NoteSym::FS6,
            91 => NoteSym::G6,
            92 => NoteSym::GS6,
            93 => NoteSym::A6,
            94 => NoteSym::AS6,
            95 => NoteSym::B6,

            96 => NoteSym::C7,
            97 => NoteSym::CS7,
            98 => NoteSym::D7,
            99 => NoteSym::DS7,
            100 => NoteSym::E7,
            101 => NoteSym::F7,
            102 => NoteSym::FS7,
            103 => NoteSym::G7,
            104 => NoteSym::GS7,
            105 => NoteSym::A7,
            106 => NoteSym::AS7,
            107 => NoteSym::B7,

            108 => NoteSym::C8,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

impl fmt::Display for NoteSym {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NoteSym::A0 =>  { write!(f, "A0") },
            NoteSym::AS0 => { write!(f, "AS0") },
            NoteSym::B0  => { write!(f, "B0") },

            NoteSym::C1  => { write!(f, "C1") },
            NoteSym::CS1 => { write!(f, "CS1") },
            NoteSym::D1  => { write!(f, "D1") },
            NoteSym::DS1 => { write!(f, "DS1") },
            NoteSym::E1  => { write!(f, "E1") },
            NoteSym::F1  => { write!(f, "F1") },
            NoteSym::FS1 => { write!(f, "FS1") },
            NoteSym::G1  => { write!(f, "G1") },
            NoteSym::GS1 => { write!(f, "GS1") },
            NoteSym::A1  => { write!(f, "A1") },
            NoteSym::AS1 => { write!(f, "A1S") },
            NoteSym::B1  => { write!(f, "B1") },

            NoteSym::C2  => { write!(f, "C2") },
            NoteSym::CS2 => { write!(f, "CS2") },
            NoteSym::D2  => { write!(f, "D2") },
            NoteSym::DS2 => { write!(f, "DS2") },
            NoteSym::E2  => { write!(f, "E2") },
            NoteSym::F2  => { write!(f, "F2") },
            NoteSym::FS2 => { write!(f, "FS2") },
            NoteSym::G2  => { write!(f, "G2") },
            NoteSym::GS2 => { write!(f, "GS2") },
            NoteSym::A2  => { write!(f, "A2") },
            NoteSym::AS2 => { write!(f, "AS2") },
            NoteSym::B2  => { write!(f, "B2") },

            NoteSym::C3  => { write!(f, "C3") },
            NoteSym::CS3 => { write!(f, "CS3") },
            NoteSym::D3  => { write!(f, "D3") },
            NoteSym::DS3 => { write!(f, "DS3") },
            NoteSym::E3  => { write!(f, "E3") },
            NoteSym::F3  => { write!(f, "F3") },
            NoteSym::FS3 => { write!(f, "FS3") },
            NoteSym::G3  => { write!(f, "G3") },
            NoteSym::GS3 => { write!(f, "GS3") },
            NoteSym::A3  => { write!(f, "A3") },
            NoteSym::AS3 => { write!(f, "AS3") },
            NoteSym::B3  => { write!(f, "B3") },

            NoteSym::C4  => { write!(f, "C4") },
            NoteSym::CS4 => { write!(f, "CS4") },
            NoteSym::D4  => { write!(f, "D4") },
            NoteSym::DS4 => { write!(f, "DS4") },
            NoteSym::E4  => { write!(f, "E4") },
            NoteSym::F4  => { write!(f, "F4") },
            NoteSym::FS4 => { write!(f, "FS4") },
            NoteSym::G4  => { write!(f, "G4") },
            NoteSym::GS4 => { write!(f, "GS4") },
            NoteSym::A4  => { write!(f, "A4") },
            NoteSym::AS4 => { write!(f, "AS4") },
            NoteSym::B4  => { write!(f, "B4") },

            NoteSym::C5  => { write!(f, "C5") },
            NoteSym::CS5 => { write!(f, "CS5") },
            NoteSym::D5  => { write!(f, "D5") },
            NoteSym::DS5 => { write!(f, "DS5") },
            NoteSym::E5  => { write!(f, "E5") },
            NoteSym::F5  => { write!(f, "F5") },
            NoteSym::FS5 => { write!(f, "FS5") },
            NoteSym::G5  => { write!(f, "G5") },
            NoteSym::GS5 => { write!(f, "GS5") },
            NoteSym::A5  => { write!(f, "A5") },
            NoteSym::AS5 => { write!(f, "AS5") },
            NoteSym::B5  => { write!(f, "B5") },

            NoteSym::C6  => { write!(f, "C6") },
            NoteSym::CS6 => { write!(f, "CS6") },
            NoteSym::D6  => { write!(f, "D6") },
            NoteSym::DS6 => { write!(f, "DS6") },
            NoteSym::E6  => { write!(f, "E6") },
            NoteSym::F6  => { write!(f, "F6") },
            NoteSym::FS6 => { write!(f, "FS6") },
            NoteSym::G6  => { write!(f, "G6") },
            NoteSym::GS6 => { write!(f, "GS6") },
            NoteSym::A6  => { write!(f, "A6") },
            NoteSym::AS6 => { write!(f, "AS6") },
            NoteSym::B6  => { write!(f, "B6") },

            NoteSym::C7  => { write!(f, "C7") },
            NoteSym::CS7 => { write!(f, "CS7") },
            NoteSym::D7  => { write!(f, "D7") },
            NoteSym::DS7 => { write!(f, "DS7") },
            NoteSym::E7  => { write!(f, "E7") },
            NoteSym::F7  => { write!(f, "F7") },
            NoteSym::FS7 => { write!(f, "FS7") },
            NoteSym::G7  => { write!(f, "G7") },
            NoteSym::GS7 => { write!(f, "GS7") },
            NoteSym::A7  => { write!(f, "A7") },
            NoteSym::AS7 => { write!(f, "AS7") },
            NoteSym::B7  => { write!(f, "B7") },

            NoteSym::C8  => { write!(f, "C8") },

            _ => { write!(f, "None") }
        }
    }
}