use crate::splitter::{Splitter, MAX_BLOCK_SIZE, MIN_BLOCK_SIZE};
use alloc::collections::VecDeque;
use alloc::fmt::{Debug, Display, Formatter};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::mem;
use core::ops::Range;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct Segment {
    index: usize,
    tp: SegmentType,
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum SegmentType {
    Ascii(Vec<u8>),
    Utf8(Vec<char>),
    Unicode(Vec<String>),
}

impl SegmentType {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        match &self {
            SegmentType::Ascii(val) => val.len(),
            SegmentType::Utf8(val) => val.len(),
            SegmentType::Unicode(val) => val.len(),
        }
    }

    pub fn try_merge(&mut self, seg_type: SegmentType) -> Option<SegmentType> {
        if self.len() + seg_type.len() >= MAX_BLOCK_SIZE {
            return Some(seg_type);
        }

        match self {
            SegmentType::Ascii(val) => {
                if let SegmentType::Ascii(val_1) = seg_type {
                    val.extend(val_1);
                    None
                } else {
                    Some(seg_type)
                }
            }
            SegmentType::Utf8(val) => {
                if let SegmentType::Utf8(val_1) = seg_type {
                    val.extend(val_1);
                    None
                } else {
                    Some(seg_type)
                }
            }
            SegmentType::Unicode(val) => {
                if let SegmentType::Unicode(val_1) = seg_type {
                    val.extend(val_1);
                    None
                } else {
                    Some(seg_type)
                }
            }
        }
    }

    pub fn split(&mut self, at: usize) -> SegmentType {
        match self {
            SegmentType::Ascii(val) => SegmentType::Ascii(val.split_off(at)),
            SegmentType::Utf8(val) => SegmentType::Utf8(val.split_off(at)),
            SegmentType::Unicode(val) => SegmentType::Unicode(val.split_off(at)),
        }
    }
}

impl Segment {
    pub fn new(index: usize, tp: SegmentType) -> Segment {
        Segment { index, tp }
    }

    pub fn try_merge(&mut self, new_segments: &mut VecDeque<SegmentType>) {
        if let Some(first) = new_segments.pop_front() {
            if let Some(first) = self.tp.try_merge(first) {
                new_segments.insert(0, first);
            }
        }
    }

    pub fn insert(&mut self, index: usize, text: &str) -> Option<VecDeque<Segment>> {
        let index = index - self.index;
        let mut new_segments = Splitter::new(text).collect::<VecDeque<_>>();

        if self.len() == 0 {
            if let Some(val) = new_segments.pop_front() {
                self.tp = val;
            }
        } else if index == self.len() - 1 {
            self.try_merge(&mut new_segments);
        } else if index == 0 {
            if let Some(mut first) = new_segments.pop_front() {
                mem::swap(&mut self.tp, &mut first);
                new_segments.push_back(first);
                self.try_merge(&mut new_segments);
            }
        } else {
            new_segments.push_back(self.tp.split(index));
            self.try_merge(&mut new_segments);
        }

        if new_segments.is_empty() {
            None
        } else {
            Some(
                new_segments
                    .into_iter()
                    .filter(|t| !t.is_empty())
                    .map(|t| Segment::new(0, t))
                    .collect(),
            )
        }
    }

    pub fn cut(&mut self, range: Range<usize>) -> Option<Segment> {
        let start = range.start - self.index;
        let end = range.end - self.index;

        if start >= self.len() {
            return None;
        }

        if end >= self.len() {
            self.tp.split(start);
            None
        } else {
            let mut last = self.tp.split(start);
            let last = last.split(end - start);
            if last.len() < MIN_BLOCK_SIZE || self.tp.len() < MIN_BLOCK_SIZE {
                if let Some(last) = self.tp.try_merge(last) {
                    if last.is_empty() {
                        None
                    } else {
                        Some(Segment::new(0, last))
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    pub fn replace(&mut self, range: Range<usize>, text: &str) -> Option<VecDeque<Segment>> {
        let start = range.start - self.index;
        let end = range.end - self.index;
        let mut new_segments = Splitter::new(text).collect::<VecDeque<_>>();
        if end > self.len() {
            self.tp.split(start);
            self.try_merge(&mut new_segments);
        } else {
            let end = self.tp.split(end);
            self.tp.split(start);
            self.try_merge(&mut new_segments);

            if !end.is_empty() {
                new_segments.push_back(end);
            }
        }

        if new_segments.is_empty() {
            None
        } else {
            Some(
                new_segments
                    .into_iter()
                    .filter(|t| !t.is_empty())
                    .map(|t| Segment::new(0, t))
                    .collect(),
            )
        }
    }

    pub fn len(&self) -> usize {
        self.tp.len()
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn contains(&self, index: usize) -> bool {
        self.ord(index) == Ordering::Equal
    }

    pub fn ord(&self, index: usize) -> Ordering {
        let start = self.index;

        let end = self.len() + start;

        if start > index {
            Ordering::Greater
        } else if end < index {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl Debug for SegmentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            SegmentType::Ascii(_) => write!(f, "Ascii({})", self),
            SegmentType::Utf8(_) => write!(f, "Utf8({})", self),
            SegmentType::Unicode(_) => write!(f, "Unicode({})", self),
        }
    }
}

impl Display for SegmentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            SegmentType::Ascii(val) => f.write_str(String::from_utf8_lossy(val).as_ref()),
            SegmentType::Utf8(val) => {
                for ch in val {
                    Display::fmt(&ch, f)?;
                }
                Ok(())
            }
            SegmentType::Unicode(unicode) => {
                for ch in unicode {
                    Display::fmt(&ch, f)?;
                }
                Ok(())
            }
        }
    }
}

impl Debug for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}-{:?}", self.index, self.tp)
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.tp)
    }
}

impl From<Vec<u8>> for Segment {
    fn from(val: Vec<u8>) -> Self {
        Segment {
            index: 0,
            tp: SegmentType::Ascii(val),
        }
    }
}

impl From<Vec<char>> for Segment {
    fn from(val: Vec<char>) -> Self {
        Segment {
            index: 0,
            tp: SegmentType::Utf8(val),
        }
    }
}

impl From<Vec<String>> for Segment {
    fn from(val: Vec<String>) -> Self {
        Segment {
            index: 0,
            tp: SegmentType::Unicode(val),
        }
    }
}

impl Default for Segment {
    fn default() -> Self {
        Segment {
            index: 0,
            tp: SegmentType::Ascii(vec![]),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::segment::{Segment, SegmentType};
    use alloc::format;
    use alloc::string::ToString;
    use core::cmp::Ordering;

    #[test]
    fn test_ord() {
        let seg = Segment::new(5, SegmentType::Ascii("Hello world".as_bytes().to_vec()));
        assert_eq!(seg.ord(1), Ordering::Greater);
        assert_eq!(seg.ord(5), Ordering::Equal);
        assert_eq!(seg.ord(14), Ordering::Equal);
        assert_eq!(seg.ord(16), Ordering::Equal);
        assert_eq!(seg.ord(17), Ordering::Less);
        assert!(!seg.contains(0));
        assert!(seg.contains(14));
        assert!(!seg.contains(17));
    }

    #[test]
    fn test_insert() {
        let mut seg = Segment::new(0, SegmentType::Ascii("Hello world".as_bytes().to_vec()));
        assert!(seg.insert(10, ". Hi, bro.").is_none());
        assert_eq!(seg.to_string(), "Hello world. Hi, bro.".to_string());

        assert!(seg.insert(0, "Hi, bro.").is_none());
        assert_eq!(seg.to_string(), "Hi, bro.Hello world. Hi, bro.".to_string());

        let last = seg.insert(8, " ").unwrap().pop_front().unwrap();
        assert_eq!(seg.to_string(), "Hi, bro. ".to_string());
        assert_eq!(last.to_string(), "Hello world. Hi, bro.".to_string());

        let mut last = seg.insert(2, "🏡 ").unwrap();
        assert_eq!(seg.to_string(), "Hi".to_string());
        assert_eq!(last.pop_front().unwrap().to_string(), "🏡".to_string());
        assert_eq!(last.pop_front().unwrap().to_string(), " ".to_string());
        assert_eq!(last.pop_front().unwrap().to_string(), ", bro. ".to_string());

        let mut seg = Segment::new(0, SegmentType::Ascii("".as_bytes().to_vec()));
        seg.insert(0, "H");
        seg.insert(1, "e");
        seg.insert(2, "l");
        seg.insert(3, "l");
        seg.insert(4, "o");
        assert_eq!(seg.to_string(), "Hello".to_string());
    }

    #[test]
    fn test_cut() {
        let mut seg = Segment::new(0, SegmentType::Ascii("Hello world".as_bytes().to_vec()));
        assert!(seg.cut(5..10).is_none());
        assert_eq!(seg.to_string(), "Hellod");

        let mut seg = Segment::new(0, SegmentType::Ascii("Hello world".as_bytes().to_vec()));
        assert!(seg.cut(5..11).is_none());
        assert_eq!(seg.to_string(), "Hello");

        let mut seg = Segment::new(0, SegmentType::Ascii("Hello world".as_bytes().to_vec()));
        assert!(seg.cut(5..20).is_none());
        assert_eq!(seg.to_string(), "Hello");

        let mut seg = Segment::new(0, SegmentType::Ascii("Hello world".as_bytes().to_vec()));
        assert!(seg.cut(5..6).is_none());
        assert_eq!(seg.to_string(), "Helloworld");
    }

    #[test]
    fn test_replace() {
        let mut seg = Segment::new(0, SegmentType::Ascii("Hello world".as_bytes().to_vec()));
        assert!(seg.replace(6..11, "Json").is_none());
        assert_eq!(seg.to_string(), "Hello Json");
        let mut last = seg.replace(7..7, "ack").unwrap();
        assert_eq!(seg.to_string(), "Hello Jack");
        assert_eq!(last.pop_front().unwrap().to_string(), "son".to_string());

        let mut seg = Segment::new(0, SegmentType::Ascii("Hello world".as_bytes().to_vec()));
        assert!(seg.replace(6..20, "Json").is_none());
        assert_eq!(seg.to_string(), "Hello Json");

        let mut seg = Segment::new(0, SegmentType::Ascii("Hello world".as_bytes().to_vec()));
        assert!(seg.replace(5..20, " ").is_none());
        assert_eq!(seg.to_string(), "Hello ");
    }

    #[test]
    fn replace_small() {
        let mut seg = Segment::new(0, SegmentType::Ascii("hello world".as_bytes().to_vec()));
        let mut new_seg = seg.replace(1..9, "era").unwrap();
        assert_eq!(
            "herald",
            format!("{}{}", seg.to_string(), new_seg.pop_front().unwrap())
        );
    }
}
