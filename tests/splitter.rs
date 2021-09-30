use rope_str::splitter::{Partition, Splitter, Segment};

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
Favour barton anxious give everything parish keeps ﻿no offer use deficient expression prosperous hastened. \
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
    if let Partition::Ascii(ascii) = partition {
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
    if let Partition::Utf8(ascii) = partition {
        assert_eq!(text, &ascii.into_iter().collect::<String>());
    } else {
        panic!("Expected utf8 segment");
    }
}

fn ascii(str: &str) -> Segment {
    Segment::Ascii(str.as_bytes().to_vec())
}

#[test]
fn test_complex() {
    let text = "Таким образом реализация намеченных плановых заданий позволяет оценить значение новых предложений😈. \
    //Too show friend entrance first body sometimes disposed.\
        😈 🌋 🏔 🗻 🏕 ⛺️ 🛖 🏠 🏡 🏘\
        👨‍👩‍👧‍👦\
формировании системы обучения кадров.\
    ";
    let partition = Splitter::new(text).next().unwrap();
    assert_eq!(partition,
               Partition::Complex(vec![
                   Segment::Utf8("Таким образом реализация намеченных плановых заданий позволяет оценить значение новых предложений".chars().collect()),
                   Segment::Unicode(vec!["😈".to_string()]),
                   ascii(". //Too show friend entrance first body sometimes disposed."),
                   Segment::Unicode(vec!["😈".to_string()]),
                   ascii(" "),
                   Segment::Unicode(vec!["🌋".to_string()]),
                   ascii(" "),
                   Segment::Unicode(vec!["🏔".to_string()]),
                   ascii(" "),
                   Segment::Unicode(vec!["🗻".to_string()]),
                   ascii(" "),
                   Segment::Unicode(vec!["🏕".to_string()]),
                   ascii(" "),
                   Segment::Unicode(vec!["⛺️".to_string()]),
                   ascii(" "),
                   Segment::Unicode(vec!["🛖".to_string()]),
                   ascii(" "),
                   Segment::Unicode(vec!["🏠".to_string()]),
                   ascii(" "),
                   Segment::Unicode(vec!["🏡".to_string()]),
                   ascii(" "),
                   Segment::Unicode(vec!["🏘".to_string(), "👨‍👩‍👧‍👦".to_string()]),
                   Segment::Utf8("формировании системы обучения кадров.".chars().collect()),
               ])
    )
}
