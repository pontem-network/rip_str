use crate::segment::SegmentType;
use memchr::memrchr;
use seshat::unicode::Segmentation;
use std::cmp::min;
use std::collections::VecDeque;
use std::mem;

pub const MAX_BLOCK_SIZE: usize = 1024;
pub const MIN_BLOCK_SIZE: usize = 512;

pub struct Splitter<'a> {
    buffer: &'a str,
    segments: VecDeque<SegmentType>,
}

impl<'a> Splitter<'a> {
    pub fn new(buffer: &'a str) -> Splitter<'a> {
        Splitter {
            buffer,
            segments: VecDeque::new(),
        }
    }
}

impl<'a> Splitter<'a> {
    pub fn make_segments(&mut self, split_point: usize) -> Option<SegmentType> {
        let str = &self.buffer[..split_point];
        self.buffer = &self.buffer[split_point..];

        let mut current_seq = SegmentType::Ascii(vec![]);
        for seq in str.break_graphemes() {
            if seq.is_ascii() {
                if let SegmentType::Ascii(ascii_seq) = &mut current_seq {
                    ascii_seq.extend_from_slice(seq.as_bytes());
                } else {
                    if let SegmentType::Utf8(vars) = &mut current_seq {
                        let is_alphabetic = seq.as_bytes().iter().any(|b| b.is_ascii_alphabetic());
                        if !is_alphabetic {
                            vars.extend(seq.chars());
                            continue;
                        }
                    }
                    let is_current_empty = current_seq.is_empty();
                    let prev = mem::replace(
                        &mut current_seq,
                        SegmentType::Ascii(seq.as_bytes().to_vec()),
                    );
                    if !is_current_empty {
                        self.segments.push_front(prev)
                    }
                }
            } else if seq.len() > 2 {
                if let SegmentType::Unicode(unicode_seq) = &mut current_seq {
                    unicode_seq.push(seq.to_string());
                } else {
                    let is_current_empty = current_seq.is_empty();
                    let prev = mem::replace(
                        &mut current_seq,
                        SegmentType::Unicode(vec![seq.to_string()]),
                    );
                    if !is_current_empty {
                        self.segments.push_front(prev)
                    }
                }
            } else {
                if let SegmentType::Utf8(char_seq) = &mut current_seq {
                    char_seq.extend(seq.chars());
                } else {
                    let is_current_empty = current_seq.is_empty();
                    let prev =
                        mem::replace(&mut current_seq, SegmentType::Utf8(seq.chars().collect()));
                    if !is_current_empty {
                        self.segments.push_front(prev)
                    }
                }
            }
        }

        if !current_seq.is_empty() {
            self.segments
                .push_front(mem::replace(&mut current_seq, SegmentType::Ascii(vec![])));
        }

        self.segments.pop_back()
    }
}

impl<'a> Iterator for Splitter<'a> {
    type Item = SegmentType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() && self.segments.is_empty() {
            return None;
        }

        if self.segments.is_empty() {
            if self.buffer.len() <= MAX_BLOCK_SIZE {
                return self.make_segments(self.buffer.len());
            }

            let mut split_point = min(MAX_BLOCK_SIZE, self.buffer.len() - MIN_BLOCK_SIZE);
            match memrchr(
                b'\n',
                &self.buffer.as_bytes()[MIN_BLOCK_SIZE - 1..split_point],
            ) {
                Some(pos) => self.make_segments(MIN_BLOCK_SIZE + pos),
                None => {
                    while !self.buffer.is_char_boundary(split_point) {
                        split_point -= 1;
                    }
                    self.make_segments(split_point)
                }
            }
        } else {
            self.segments.remove(self.segments.len() - 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::segment::SegmentType;
    use crate::splitter::Splitter;

    fn split_check(partition: &[&str]) {
        let text: String = partition.iter().map(|p| p.to_string()).collect();

        let actual: Vec<_> = Splitter::new(&text).map(|s| s.to_string()).collect();
        assert_eq!(partition, &actual);
    }

    #[test]
    fn test_splitter() {
        split_check(&[
            "\
Too show friend entrance first body sometimes disposed. Oh sell this so relied cordial scale mirth sometimes round change never dispatched stand jennings. \
Hills lose terminated exeter oppose everything chicken noisier tended answered ignorant absolute stand branch cousins shy. \
Enjoy not enjoyed sufficient adapted returned size unpleasant suffering commanded improving. \
Repair village towards humoured consider them. \
Finished needed nature would world went proceed possible feelings wishes worthy. \
Ladyship these jointure several shed they forming warmly folly. \
Servants consider fat his cannot winding who brother greatly certainty precaution deal dashwoods. \
Admitting left attention remarkably spoil woody disposed change exercise matter period females weddings world found. \
Moderate age enabled remainder justice sentiments hastily eyes rest provision perfectly. \
Favour barton anxious give everything parish keeps ", "﻿", "no offer use deficient expression prosperous hastened. \
Call forth speaking busy week denoting. Saved ve", "ry period address. \
Often wandered sent money manners sooner exercise roof increasing seeing common furnished show society unreserved enjoyed brought. \
Uneasy declared endeavor found. Prospect set match within existence john passage although. \
Married  been purse prepared taste. Enabled depending more home building place provided under dearest pleasure goodness perhaps prepared society supported. \
Nearer cannot improve invited securing offence settled can tolerably delay savings hung about denoting views. Death believed entirely thing seeing northward that. "]
        );
    }

    #[test]
    fn test_ascii_segments() {
        let text = "Too show friend entrance first body sometimes disposed. Oh sell this so relied cordial scale mirth sometimes round change never dispatched stand jennings. \
Hills lose terminated exeter oppose everything chicken noisier tended answered ignorant absolute stand branch cousins shy. \
Enjoy not enjoyed sufficient adapted returned size unpleasant suffering commanded improving. \
Repair village towards humoured consider them.\n\
Finished needed nature would world went proceed possible feelings wishes worthy. \
Ladyship these jointure several shed they forming warmly folly. \
Servants consider fat his cannot winding who brother greatly certainty precaution deal dashwoods. \
Admitting left attention remarkably spoil woody disposed change exercise matter period females weddings world found. \
";
        let partition = Splitter::new(text).next().unwrap();
        if let SegmentType::Ascii(ascii) = partition {
            assert_eq!(text, String::from_utf8_lossy(&ascii).as_ref());
        } else {
            panic!("Expected ascii segment");
        }
    }

    #[test]
    fn test_utf8_segments() {
        let text = "Не следует, однако забывать, что дальнейшее развитие различных форм деятельности способствует подготовки и реализации форм развития. \
    Равным образом постоянный количественный рост и сфера нашей активности играет важную роль в формировании системы обучения кадров, соответствует насущным потребностям.";
        let partition = Splitter::new(text).next().unwrap();
        if let SegmentType::Utf8(ascii) = partition {
            assert_eq!(text, &ascii.into_iter().collect::<String>());
        } else {
            panic!("Expected utf8 segment");
        }
    }

    fn ascii(str: &str) -> SegmentType {
        SegmentType::Ascii(str.as_bytes().to_vec())
    }

    #[test]
    fn test_complex() {
        let text = "Таким образом реализация намеченных плановых заданий позволяет оценить значение новых предложений😈. \
    //Too show friend entrance first body sometimes disposed.\
        😈 🌋 🏔 🗻 🏕 ⛺️ 🛖 🏠 🏡 🏘\
        👨‍👩‍👧‍👦\
формировании системы обучения кадров.\
    ";
        let partition = Splitter::new(text).collect::<Vec<_>>();
        assert_eq!(partition,
                   vec![
                       SegmentType::Utf8("Таким образом реализация намеченных плановых заданий позволяет оценить значение новых предложений".chars().collect()),
                       SegmentType::Unicode(vec!["😈".to_string()]),
                       ascii(". //Too show friend entrance first body sometimes disposed."),
                       SegmentType::Unicode(vec!["😈".to_string()]),
                       ascii(" "),
                       SegmentType::Unicode(vec!["🌋".to_string()]),
                       ascii(" "),
                       SegmentType::Unicode(vec!["🏔".to_string()]),
                       ascii(" "),
                       SegmentType::Unicode(vec!["🗻".to_string()]),
                       ascii(" "),
                       SegmentType::Unicode(vec!["🏕".to_string()]),
                       ascii(" "),
                       SegmentType::Unicode(vec!["⛺️".to_string()]),
                       ascii(" "),
                       SegmentType::Unicode(vec!["🛖".to_string()]),
                       ascii(" "),
                       SegmentType::Unicode(vec!["🏠".to_string()]),
                       ascii(" "),
                       SegmentType::Unicode(vec!["🏡".to_string()]),
                       ascii(" "),
                       SegmentType::Unicode(vec!["🏘".to_string(), "👨‍👩‍👧‍👦".to_string()]),
                       SegmentType::Utf8("формировании системы обучения кадров.".chars().collect()),
                   ]
        )
    }
}
