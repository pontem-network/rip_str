use memchr::memrchr;
use seshat::unicode::Segmentation;
use std::cmp::min;
use std::fmt::{Debug, Display, Formatter, Write};
use std::mem;

const MAX_BLOCK_SIZE: usize = 1024;
const MIN_BLOCK_SIZE: usize = 512;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum Partition {
    Ascii(Vec<u8>),
    Utf8(Vec<char>),
    Complex(Vec<Segment>),
}

impl Debug for Partition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Partition::Ascii(_) => write!(f, "Ascii({})", self),
            Partition::Utf8(_) => write!(f, "Utf8({})", self),
            Partition::Complex(segs) => {
                write!(f, "Complex(")?;
                for seg in segs {
                    write!(f, "{:?}", seg)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl Display for Partition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Partition::Ascii(val) => f.write_str(String::from_utf8_lossy(val).as_ref()),
            Partition::Utf8(val) => {
                for ch in val {
                    Display::fmt(&ch, f)?;
                }
                Ok(())
            }
            Partition::Complex(vals) => {
                for val in vals {
                    Display::fmt(&val, f)?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum Segment {
    Ascii(Vec<u8>),
    Utf8(Vec<char>),
    Unicode(Vec<String>),
}

impl Debug for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::Ascii(_) => write!(f, "Ascii({})", self),
            Segment::Utf8(_) => write!(f, "Utf8({})", self),
            Segment::Unicode(_) => write!(f, "Unicode({})", self),
        }
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::Ascii(val) => f.write_str(String::from_utf8_lossy(val).as_ref()),
            Segment::Utf8(val) => {
                for ch in val {
                    Display::fmt(&ch, f)?;
                }
                Ok(())
            }
            Segment::Unicode(val) => {
                for s in val {
                    Display::fmt(&s, f)?;
                }
                Ok(())
            }
        }
    }
}

impl Segment {
    pub fn is_empty(&self) -> bool {
        match self {
            Segment::Ascii(val) => val.is_empty(),
            Segment::Utf8(val) => val.is_empty(),
            Segment::Unicode(val) => val.is_empty(),
        }
    }
}

pub struct Splitter<'a> {
    buffer: &'a str,
}

impl<'a> Splitter<'a> {
    pub fn new(buffer: &'a str) -> Splitter<'a> {
        Splitter { buffer }
    }
}

impl<'a> Splitter<'a> {
    pub fn make_partition(&mut self, split_point: usize) -> Partition {
        let str = &self.buffer[..split_point];
        self.buffer = &self.buffer[split_point..];
        let mut segments = vec![];

        let mut current_seq = Segment::Ascii(vec![]);
        for seq in str.break_graphemes() {
            if seq.is_ascii() {
                if let Segment::Ascii(ascii_seq) = &mut current_seq {
                    ascii_seq.extend_from_slice(seq.as_bytes());
                } else {
                    if let Segment::Utf8(vars) = &mut current_seq {
                        let is_alphabetic = seq.as_bytes().iter().any(|b| b.is_ascii_alphabetic());
                        if !is_alphabetic {
                            vars.extend(seq.chars());
                            continue;
                        }
                    }
                    let is_current_empty = current_seq.is_empty();
                    let prev =
                        mem::replace(&mut current_seq, Segment::Ascii(seq.as_bytes().to_vec()));
                    if !is_current_empty {
                        segments.push(prev)
                    }
                }
            } else if seq.len() > 2 {
                if let Segment::Unicode(unicode_seq) = &mut current_seq {
                    unicode_seq.push(seq.to_string());
                } else {
                    let is_current_empty = current_seq.is_empty();
                    let prev =
                        mem::replace(&mut current_seq, Segment::Unicode(vec![seq.to_string()]));
                    if !is_current_empty {
                        segments.push(prev)
                    }
                }
            } else {
                if let Segment::Utf8(char_seq) = &mut current_seq {
                    char_seq.extend(seq.chars());
                } else {
                    let is_current_empty = current_seq.is_empty();
                    let prev = mem::replace(&mut current_seq, Segment::Utf8(seq.chars().collect()));
                    if !is_current_empty {
                        segments.push(prev)
                    }
                }
            }
        }

        if !current_seq.is_empty() {
            segments.push(mem::replace(&mut current_seq, Segment::Ascii(vec![])));
        }

        if segments.len() == 1 {
            let seg = segments.remove(0);
            match seg {
                Segment::Ascii(ascii) => Partition::Ascii(ascii),
                Segment::Utf8(utf8) => Partition::Utf8(utf8),
                Segment::Unicode(unicode) => Partition::Complex(vec![Segment::Unicode(unicode)]),
            }
        } else {
            Partition::Complex(segments)
        }
    }
}

impl<'a> Iterator for Splitter<'a> {
    type Item = Partition;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.len() == 0 {
            return None;
        }

        if self.buffer.len() <= MAX_BLOCK_SIZE {
            return Some(self.make_partition(self.buffer.len()));
        }

        let mut split_point = min(MAX_BLOCK_SIZE, self.buffer.len() - MIN_BLOCK_SIZE);
        match memrchr(
            b'\n',
            &self.buffer.as_bytes()[MIN_BLOCK_SIZE - 1..split_point],
        ) {
            Some(pos) => Some(self.make_partition(MIN_BLOCK_SIZE + pos)),
            None => {
                while !self.buffer.is_char_boundary(split_point) {
                    split_point -= 1;
                }
                Some(self.make_partition(split_point))
            }
        }
    }
}
