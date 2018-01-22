use regex::{ Regex };


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaToken<'a>
{
    pub token: Token<'a>,
    pub span: Span,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span(pub usize, pub usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Token<'a>
{
    Piece,
    Voice,
    Section,
    Part,
    Play,
    LeftBrace,
    RightBrace,
    Colon,
    Comma,
    BlankLine,
    Num(i64),
    Key(&'a str),
    Ident(&'a str),
    Str(&'a str),

    Barline,
    Rest,
    Hit,
    Ditto,
    Repeat,
    Length(usize),
    Note(&'a str),
    PlayPart(&'a str),
}


pub fn lex<'a>(source: &'a str) -> Vec<MetaToken<'a>>
{
    use self::Token::*;

    let mut tokens = Vec::new();

    for capture in STRUCTURE_REGEX.captures_iter(source)
    {
        if let Some(m) = capture.name("key")
        {
            let text = m.as_str();
            let len = text.len();
            let text = text[..(len-1)].trim();
            tokens.push(MetaToken { token: Key(text), span: Span(m.start(), m.end()) });
        }

        else if let Some(m) = capture.name("ident")
        {
            let token = match m.as_str()
            {
                "piece" => Piece,
                "voice" => Voice,
                "part" => Part,
                "section" => Section,
                "play" => Play,
                s => Ident(s),
            };
            tokens.push(MetaToken { token, span: Span(m.start(), m.end()) });
        }

        else if let Some(m) = capture.name("string")
        {
            let text = m.as_str();
            let len = text.len();
            let text = &text[1..(len-1)];
            tokens.push(MetaToken { token: Str(text), span: Span(m.start(), m.end()) });
        }

        else if let Some(m) = capture.name("number")
        {
            let number = m.as_str().parse().unwrap();
            tokens.push(MetaToken { token: Num(number), span: Span(m.start(), m.end()) });
        }

        else if let Some(m) = capture.name("delim")
        {
            let token = match m.as_str()
            {
                "{" => LeftBrace,
                "}" => RightBrace,
                ":" => Colon,
                "," => Comma,
                _ => unreachable!()
            };
            tokens.push(MetaToken { token, span: Span(m.start(), m.end()) });
        }

        else if let Some(m) = capture.name("staveline")
        {
            let start = m.start();
            let text = m.as_str();

            for capture in MUSIC_REGEX.captures_iter(text)
            {
                if let Some(m) = capture.name("note")
                {
                    tokens.push(MetaToken { token: Note(m.as_str()), span: Span(start + m.start(), start + m.end()) });
                }

                else if let Some(m) = capture.name("part")
                {
                    let text = &m.as_str()[1..];
                    tokens.push(MetaToken { token: PlayPart(text), span: Span(start + m.start(), start + m.end()) });
                }

                else if let Some(m) = capture.name("barline")
                {
                    tokens.push(MetaToken { token: Barline, span: Span(start + m.start(), start + m.end()) });
                }

                else if let Some(m) = capture.name("symbol")
                {
                    let token = match m.as_str()
                    {
                        "-" => Rest,
                        "x" => Hit,
                        "\"" => Ditto,
                        "%" => Repeat,
                        _ => unreachable!()
                    };
                    tokens.push(MetaToken { token, span: Span(start + m.start(), start + m.end()) });
                }

                else if let Some(m) = capture.name("length")
                {
                    let size = match m.as_str()
                    {
                        "." => 2,
                        s if s.as_bytes()[1] == b'.' => s.len() + 1,
                        s => s[1..].parse().unwrap()
                    };
                    tokens.push(MetaToken { token: Length(size), span: Span(start + m.start(), start + m.end()) });
                }

                else if capture.name("whitespace").is_some() || capture.name("comment").is_some()
                {
                    continue
                }

                else if let Some(m) = capture.name("error")
                {
                    println!("{:?}", tokens);
                    panic!("error: Invalid token '{}' at {}", m.as_str(), m.start())
                }
            }
        }

        else if let Some(m) = capture.name("blank")
        {
            tokens.push(MetaToken { token: BlankLine, span: Span(m.start() + 1, m.end()) });
        }

        else if capture.name("whitespace").is_some() || capture.name("comment").is_some()
        {
            continue
        }

        else if let Some(m) = capture.name("error")
        {
            panic!("error: Invalid token '{}' at {}", m.as_str(), m.start())
        }

        else
        {
            unreachable!()
        }
    }

    tokens
}


lazy_static!
{
    static ref STRUCTURE_REGEX: Regex = Regex::new("\
        (?P<key>([a-zA-Z_][a-zA-Z0-9_^,'=\\-]*\\s*|:)?:)|\
        (?P<ident>[a-zA-Z_][a-zA-Z0-9_]*)|\
        (?P<string>\"((\\\\\")|[^\"])*\")|\
        (?P<number>[+\\-]?\\d+)|\
        (?P<delim>[{},])|\
        (?P<staveline>[\\|;](.*))|\
        (?P<comment>//.*)|\
        (?P<blank>\n\\s*\n)|\
        (?P<whitespace>\\s+)|\
        (?P<error>.)\
        ").unwrap();
}

lazy_static!
{
    static ref MUSIC_REGEX: Regex = Regex::new("\
        (?P<note>[a-gA-G][=_\\^]*[,']*)|\
        (?P<part>\\*[a-zA-Z_][a-zA-Z0-9_]*)|\
        (?P<symbol>[\\-x\"%])|\
        (?P<length>\\.(\\d+|\\.*))|\
        (?P<barline>[|;])|\
        (?P<comment>//.*)|\
        (?P<whitespace>\\s+)|\
        (?P<error>.)\
        ").unwrap();
}


#[cfg(test)]
mod tests
{
    use super::*;
    use super::Token::*;

    use test_helpers::mt;


    fn lextest(source: &str, result: Vec<MetaToken>)
    {
        assert_eq!(lex(source), result)
    }

    #[test]
    #[should_panic]
    fn invalid_tokens()
    {
        lextest("@", vec![])
    }

    #[test]
    fn lex_piece()
    {
        lextest("piece{}", vec![mt(Piece, (0,5)), mt(LeftBrace, (5,6)), mt(RightBrace, (6,7))])
    }

    #[test]
    fn lex_section()
    {
        lextest("section {}", vec![mt(Section, (0,7)), mt(LeftBrace, (8,9)), mt(RightBrace, (9,10))])
    }

    #[test]
    fn lex_voice()
    {
        lextest("voice {}", vec![mt(Voice, (0,5)), mt(LeftBrace, (6,7)), mt(RightBrace, (7,8))])
    }

    #[test]
    fn lex_part()
    {
        lextest("part {}", vec![mt(Part, (0,4)), mt(LeftBrace, (5,6)), mt(RightBrace, (6,7))])
    }

    #[test]
    fn lex_play()
    {
        lextest("play {}", vec![mt(Play, (0,4)), mt(LeftBrace, (5,6)), mt(RightBrace, (6,7))])
    }

    #[test]
    fn comments_ignored()
    {
        lextest(
            "// This is a piece\npiece{}",
            vec![mt(Piece, (19,24)), mt(LeftBrace, (24,25)), mt(RightBrace, (25,26))])
    }

    #[test]
    fn insignificant_whitespace_ignored()
    {
        lextest("piece {\n}", vec![mt(Piece, (0,5)), mt(LeftBrace, (6,7)), mt(RightBrace, (8,9))])
    }

    #[test]
    fn lex_name()
    {
        lextest("piece Heroine {}", vec![
                mt(Piece, (0,5)),
                mt(Ident("Heroine"), (6,13)),
                mt(LeftBrace, (14,15)),
                mt(RightBrace, (15,16)),
        ]);
    }

    #[test]
    fn lex_quoted_name()
    {
        lextest(r#"piece "Lust for Life" {}"#, vec![
                mt(Piece, (0,5)),
                mt(Str("Lust for Life"), (6,21)),
                mt(LeftBrace, (22,23)),
                mt(RightBrace, (23,24)),
        ]);
    }

    #[test]
    fn lex_quoted_name_with_quotes_in_it()
    {
        lextest(r#"piece "\"Lust\" for \"Life\"" {}"#, vec![
                mt(Piece, (0,5)),
                mt(Str(r#"\"Lust\" for \"Life\""#), (6,29)),
                mt(LeftBrace, (30,31)),
                mt(RightBrace, (31,32)),
        ]);
    }

    #[test]
    fn lex_empty_key()
    {
        lextest("{ : A }", vec![
                mt(LeftBrace, (0,1)),
                mt(Key(""), (2,3)),
                mt(Ident("A"), (4,5)),
                mt(RightBrace, (6,7)),
        ]);
    }

    #[test]
    fn lex_all_staves_key()
    {
        lextest(":: | -", vec![
                mt(Key(":"), (0,2)),
                mt(Barline, (3,4)),
                mt(Rest, (5,6)),
        ]);
    }

    #[test]
    fn lex_field_in_block()
    {
        lextest(r#"piece LFL { title: "Party Girl" }"#, vec![
                mt(Piece, (0,5)),
                mt(Ident("LFL"), (6,9)),
                mt(LeftBrace, (10,11)),
                mt(Key("title"), (12,18)),
                mt(Str("Party Girl"), (19,31)),
                mt(RightBrace, (32,33)),
        ]);
    }

    #[test]
    fn lex_ridiculous_field_name()
    {
        lextest(r#"piece LFL { F^_=,,''   : "Party Girl" }"#, vec![
                mt(Piece, (0,5)),
                mt(Ident("LFL"), (6,9)),
                mt(LeftBrace, (10,11)),
                mt(Key("F^_=,,''"), (12,24)),
                mt(Str("Party Girl"), (25,37)),
                mt(RightBrace, (38,39)),
        ]);
    }

    #[test]
    fn lex_multiple_fields()
    {
        lextest("{ drums, name: drum_voice }", vec![
                mt(LeftBrace, (0,1)),
                mt(Ident("drums"), (2,7)),
                mt(Comma, (7,8)),
                mt(Key("name"), (9,14)),
                mt(Ident("drum_voice"), (15,25)),
                mt(RightBrace, (26,27)),
        ]);
    }

    #[test]
    fn lex_numbers()
    {
        lextest("{ channel: 0, octave: -1 }", vec![
                mt(LeftBrace, (0,1)),
                mt(Key("channel"), (2,10)),
                mt(Num(0), (11,12)),
                mt(Comma, (12,13)),
                mt(Key("octave"), (14,21)),
                mt(Num(-1), (22,24)),
                mt(RightBrace, (25,26)),
        ]);
    }

    #[test]
    fn lex_blank_lines()
    {
        lextest("{ a: 0,\n\nb: 1 }", vec![
                mt(LeftBrace, (0,1)),
                mt(Key("a"), (2,4)),
                mt(Num(0), (5,6)),
                mt(Comma, (6,7)),
                mt(BlankLine, (8,9)),
                mt(Key("b"), (9,11)),
                mt(Num(1), (12,13)),
                mt(RightBrace, (14,15)),
        ]);
    }

    #[test]
    fn lex_note()
    {
        lextest(": | A", vec![
                mt(Key(""), (0,1)),
                mt(Barline, (2,3)),
                mt(Note("A"), (4,5)),
        ]);
    }

    #[test]
    fn lex_complex_notes()
    {
        lextest(": | B^,,c_''d=", vec![
                mt(Key(""), (0,1)),
                mt(Barline, (2,3)),
                mt(Note("B^,,"), (4,8)),
                mt(Note("c_''"), (8,12)),
                mt(Note("d="), (12,14)),
        ]);
    }

    #[test]
    fn lex_note_length()
    {
        lextest(": | A... B.4 | C.", vec![
                mt(Key(""), (0,1)),
                mt(Barline, (2,3)),
                mt(Note("A"), (4,5)),
                mt(Length(4), (5,8)),
                mt(Note("B"), (9,10)),
                mt(Length(4), (10,12)),
                mt(Barline, (13,14)),
                mt(Note("C"), (15,16)),
                mt(Length(2), (16,17)),
        ]);
    }

    #[test]
    fn lex_symbols()
    {
        lextest("C : | x - x-| % | \" |", vec![
                mt(Key("C"), (0,3)),
                mt(Barline, (4,5)),
                mt(Hit, (6,7)),
                mt(Rest, (8,9)),
                mt(Hit, (10,11)),
                mt(Rest, (11,12)),
                mt(Barline, (12,13)),
                mt(Repeat, (14,15)),
                mt(Barline, (16,17)),
                mt(Ditto, (18,19)),
                mt(Barline, (20,21)),
        ]);
    }

    #[test]
    fn lex_play_part()
    {
        lextest(":| *Theme", vec![
                mt(Key(""), (0,1)),
                mt(Barline, (1,2)),
                mt(PlayPart("Theme"), (3,9)),
        ]);
    }
}
