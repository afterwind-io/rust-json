use super::utils::{UTF8Reader, UTF8ReaderResult};

const MAX_DEPTH: usize = 100;

// Structural Tokens
const ST_LSBRACKET: &str = "[";
const ST_RSBRACKET: &str = "]";
const ST_LCBRACKET: &str = "{";
const ST_RCBRACKET: &str = "}";
const ST_COLON: &str = ":";
const ST_COMMA: &str = ",";

// Literal Name Tokens
const LN_TRUE: &str = "true";
const LN_FALSE: &str = "false";
const LN_NULL: &str = "null";

// Leading Tokens
const LT_TRUE: &str = "t";
const LT_FALSE: &str = "f";
const LT_NULL: &str = "n";

// Insignificant Whitespace
const WS_CHARACTER_TABULATION: &str = "\u{0009}";
const WS_LINE_FEED: &str = "\u{000A}";
const WS_CARRIAGE_RETURN: &str = "\u{000D}";
const WS_SPACE: &str = "\u{0020}";

const SP_QUOTE: &str = "\"";
const SP_REVERSE_SOLIDUS: &str = "\\";
const SP_SOLIDUS: &str = "/";
const SP_BACKSPACE: &str = "b";
const SP_FORM_FEED: &str = "f";
const SP_LINE_FEED: &str = "n";
const SP_CARRIAGE_RETURN: &str = "r";
const SP_CHARACTER_TABULATION: &str = "t";
const SP_UNICODE: &str = "u";
const SP_MINUS: &str = "-";
const SP_DECIMAL_POINT: &str = ".";

pub fn validate(document: &UTF8Reader) -> Result<(), String> {
    enum State {
        PreDocument,
        PostDocument,
    }

    fn error(index: usize, reason: &str) -> Result<(), String> {
        return Err(format!(
            "Validation Error @ 1:{}\nReason: {}",
            index + 1,
            reason
        ));
    }

    let length = document.len();
    if length == 0 {
        return error(0, "JSON document can not be empty");
    }

    let mut state = State::PreDocument;
    let mut ptr = 0;

    loop {
        let chr = match document.look_ahead(ptr, 1) {
            UTF8ReaderResult::Ok(s) => s,
            UTF8ReaderResult::OutOfBoundError(_) => {
                if let State::PreDocument = state {
                    return error(ptr, "No valid JSON value found");
                }
                break;
            }
        };

        match state {
            State::PreDocument => match chr {
                _ if is_insignificant_whitespace(chr) => ptr += 1,
                _ => {
                    let (result, step) = validate_json_value(document, ptr, 0);
                    ptr += step;

                    match result {
                        Ok(_) => state = State::PostDocument,
                        Err(reason) => return error(ptr, &reason),
                    }
                }
            },
            State::PostDocument => match chr {
                _ if is_insignificant_whitespace(chr) => ptr += 1,
                _ => return error(ptr, &format!("Expect EOF, but found \"{}\"", chr)),
            },
        }
    }

    return Ok(());
}

fn validate_json_value(
    document: &UTF8Reader,
    index: usize,
    depth: usize,
) -> (Result<(), String>, usize) {
    return match document.look_ahead(index, 1) {
        UTF8ReaderResult::OutOfBoundError(_) => {
            return (Err(format!("Look ahead out of bound")), 1);
        }
        UTF8ReaderResult::Ok(chr) => match chr {
            ST_LCBRACKET => validate_object(document, index, depth + 1),
            ST_LSBRACKET => validate_array(document, index, depth + 1),
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | SP_MINUS => {
                validate_number(document, index)
            }
            SP_QUOTE => validate_string(document, index),
            LT_TRUE => validate_true(document, index),
            LT_FALSE => validate_false(document, index),
            LT_NULL => validate_null(document, index),
            _ => {
                return (Err(format!("Unknown character: \"{}\"", chr)), 1);
            }
        },
    };
}

fn validate_object(
    document: &UTF8Reader,
    start: usize,
    depth: usize,
) -> (Result<(), String>, usize) {
    enum State {
        Begin,
        PreKey,
        Key,
        PreValue,
        Value,
        PostValue,
    }

    if depth > MAX_DEPTH {
        return (Err(format!("Nested JSON value is too deep")), 0);
    }

    let mut state: State = State::Begin;
    let mut ptr = 0;

    loop {
        let index = start + ptr;

        let chr = match document.look_ahead(index, 1) {
            UTF8ReaderResult::Ok(s) => s,
            UTF8ReaderResult::OutOfBoundError(i) => {
                return (Err(format!("Incomplete number value")), i)
            }
        };

        match state {
            State::Begin => {
                if chr != ST_LCBRACKET {
                    return (Err(String::from("Object should start with \"{\"")), ptr);
                }
                state = State::PreKey;
            }
            State::PreKey => match chr {
                ST_RCBRACKET => return (Ok(()), ptr + 1),
                _ if is_insignificant_whitespace(chr) => {}
                _ => {
                    let (result, step) = validate_string(document, index);
                    ptr += step;

                    if let Ok(_) = result {
                        state = State::PreValue;
                        continue;
                    } else {
                        return (
                            Err(String::from("Object key should be a valid string")),
                            ptr,
                        );
                    }
                }
            },
            State::Key => match chr {
                _ if is_insignificant_whitespace(chr) => {}
                _ => {
                    let (result, step) = validate_string(document, index);
                    ptr += step;

                    if let Ok(_) = result {
                        state = State::PreValue;
                        continue;
                    } else {
                        return (
                            Err(String::from("Object key should be a valid string")),
                            ptr,
                        );
                    }
                }
            },
            State::PreValue => match chr {
                ST_COLON => state = State::Value,
                _ if is_insignificant_whitespace(chr) => {}
                _ => {
                    return (
                        Err(format!("Invalid character after object key: \"{}\"", chr)),
                        ptr,
                    )
                }
            },
            State::Value => match chr {
                _ if is_insignificant_whitespace(chr) => {}
                _ => {
                    let (result, step) = validate_json_value(document, index, depth);
                    ptr += step;

                    if let Ok(_) = result {
                        state = State::PostValue;
                        continue;
                    } else {
                        return (result, ptr);
                    }
                }
            },
            State::PostValue => match chr {
                ST_RCBRACKET => return (Ok(()), ptr + 1),
                ST_COMMA => state = State::Key,
                _ if is_insignificant_whitespace(chr) => {}
                _ => {
                    return (
                        Err(format!("Invalid character after object value: \"{}\"", chr)),
                        ptr,
                    )
                }
            },
        }

        ptr += 1;
    }
}

fn validate_array(
    document: &UTF8Reader,
    start: usize,
    depth: usize,
) -> (Result<(), String>, usize) {
    enum State {
        Begin,
        PreValue,
        Value,
        PostValue,
    }

    if depth > MAX_DEPTH {
        return (Err(format!("Nested JSON value is too deep")), 0);
    }

    let mut state: State = State::Begin;
    let mut ptr = 0;

    loop {
        let index = start + ptr;

        let chr = match document.look_ahead(index, 1) {
            UTF8ReaderResult::Ok(s) => s,
            UTF8ReaderResult::OutOfBoundError(i) => {
                return (Err(format!("Incomplete number value")), i)
            }
        };

        match state {
            State::Begin => {
                if chr != ST_LSBRACKET {
                    return (Err(String::from("Array should start with \"[\"")), ptr);
                }
                state = State::PreValue;
            }
            State::PreValue => match chr {
                ST_RSBRACKET => return (Ok(()), ptr + 1),
                _ if is_insignificant_whitespace(chr) => {}
                _ => {
                    let (result, step) = validate_json_value(document, index, depth);
                    ptr += step;

                    if let Ok(_) = result {
                        state = State::PostValue;
                        continue;
                    } else {
                        return (result, ptr);
                    }
                }
            },
            State::Value => match chr {
                _ if is_insignificant_whitespace(chr) => {}
                _ => {
                    let (result, step) = validate_json_value(document, index, depth);
                    ptr += step;

                    if let Ok(_) = result {
                        state = State::PostValue;
                        continue;
                    } else {
                        return (result, ptr);
                    }
                }
            },
            State::PostValue => match chr {
                ST_RSBRACKET => return (Ok(()), ptr + 1),
                ST_COMMA => state = State::Value,
                _ if is_insignificant_whitespace(chr) => {}
                _ => return (Err(format!("Invalid character: \"{}\"", chr)), ptr),
            },
        }

        ptr += 1;
    }
}

fn validate_number(document: &UTF8Reader, start: usize) -> (Result<(), String>, usize) {
    enum State {
        Begin,
        LeadingMinus,
        LeadingZero,
        Integer,
        PendingFraction,
        Fraction,
        ExponentSign, // + or -
        PendingExponent,
        Exponent,
    }

    fn is_valid_demical_number(chr: &str, non_zero: bool) -> bool {
        let c = chr.chars().nth(0).unwrap();
        match c {
            '1'..='9' => true,
            '0' => !non_zero,
            _ => false,
        }
    }

    fn is_end_of_number(chr: &str) -> bool {
        match chr {
            ST_COMMA | ST_RCBRACKET | ST_RSBRACKET => true,
            _ if is_insignificant_whitespace(chr) => true,
            _ => false,
        }
    }

    let mut state: State = State::Begin;
    let mut ptr = 0;

    loop {
        let index = start + ptr;

        let chr = match document.look_ahead(index, 1) {
            UTF8ReaderResult::Ok(s) => s,
            UTF8ReaderResult::OutOfBoundError(tail_offset) => match state {
                State::LeadingZero | State::Integer | State::Fraction | State::Exponent => {
                    return (Ok(()), ptr)
                }
                _ => return (Err(format!("Incomplete number value")), tail_offset),
            },
        };

        match state {
            State::Begin => match chr {
                SP_MINUS => state = State::LeadingMinus,
                "0" => state = State::LeadingZero,
                _ if is_valid_demical_number(chr, true) => state = State::Integer,
                _ => return (Err(format!("Invalid number leading: {:?}", chr)), ptr),
            },
            State::LeadingMinus => match chr {
                "0" => state = State::LeadingZero,
                _ if is_valid_demical_number(chr, true) => state = State::Integer,
                _ => {
                    return (
                        Err(format!("Invalid character after leading minus: {:?}", chr)),
                        ptr,
                    )
                }
            },
            State::LeadingZero => match chr {
                SP_DECIMAL_POINT => state = State::PendingFraction,
                "e" | "E" => state = State::ExponentSign,
                _ if is_valid_demical_number(chr, false) => {
                    return (Err(format!("Leading zeros are not allowed")), ptr)
                }
                _ if is_end_of_number(chr) => return (Ok(()), ptr),
                _ => {
                    return (
                        Err(format!("Invalid character after leading zero: {:?}", chr)),
                        ptr,
                    )
                }
            },
            State::Integer => match chr {
                SP_DECIMAL_POINT => state = State::PendingFraction,
                "e" | "E" => state = State::ExponentSign,
                _ if is_valid_demical_number(chr, false) => {}
                _ if is_end_of_number(chr) => return (Ok(()), ptr),
                _ => {
                    return (
                        Err(format!("Invalid character in interger part: {:?}", chr)),
                        ptr,
                    )
                }
            },
            State::PendingFraction => match chr {
                _ if is_valid_demical_number(chr, false) => state = State::Fraction,
                _ => {
                    return (
                        Err(format!("Invalid character after demical point: {:?}", chr)),
                        ptr,
                    )
                }
            },
            State::Fraction => match chr {
                "e" | "E" => state = State::ExponentSign,
                _ if is_valid_demical_number(chr, false) => {}
                _ if is_end_of_number(chr) => return (Ok(()), ptr),
                _ => {
                    return (
                        Err(format!("Invalid character in fraction part: {:?}", chr)),
                        ptr,
                    )
                }
            },
            State::ExponentSign => match chr {
                "+" | "-" => state = State::PendingExponent,
                _ if is_valid_demical_number(chr, false) => state = State::Exponent,
                _ => {
                    return (
                        Err(format!("Invalid character in exponent part: {:?}", chr)),
                        ptr,
                    )
                }
            },
            State::PendingExponent => match chr {
                _ if is_valid_demical_number(chr, false) => state = State::Exponent,
                _ => {
                    return (
                        Err(format!("Invalid character in exponent part: {:?}", chr)),
                        ptr,
                    )
                }
            },
            State::Exponent => match chr {
                _ if is_valid_demical_number(chr, false) => {}
                _ if is_end_of_number(chr) => return (Ok(()), ptr),
                _ => {
                    return (
                        Err(format!("Invalid character in exponent part: {:?}", chr)),
                        ptr,
                    )
                }
            },
        }

        ptr += 1;
    }
}

fn validate_string(document: &UTF8Reader, start: usize) -> (Result<(), String>, usize) {
    enum State {
        Begin,
        PlainText,
        Escaping,
        Unicode,
    }

    fn is_control_character(chr: &str) -> bool {
        let c = chr.chars().nth(0).unwrap();
        match c {
            '\u{0000}'..='\u{001F}' => true,
            _ => false,
        }
    }

    fn is_hex_digit(chr: &str) -> bool {
        let c = chr.chars().nth(0).unwrap();
        match c {
            '0'..='9' | 'A'..='F' | 'a'..='f' => true,
            _ => false,
        }
    }

    let mut state: State = State::Begin;
    let mut ptr = 0;
    let mut unicode_len = 0;

    loop {
        let index = start + ptr;

        let chr = match document.look_ahead(index, 1) {
            UTF8ReaderResult::Ok(s) => s,
            UTF8ReaderResult::OutOfBoundError(i) => {
                return (Err(format!("Incomplete string value")), i)
            }
        };

        match state {
            State::Begin => {
                if chr != SP_QUOTE {
                    return (Err(String::from("String value should start with \"")), ptr);
                }

                state = State::PlainText;
            }
            State::PlainText => match chr {
                SP_QUOTE => return (Ok(()), ptr + 1),
                SP_REVERSE_SOLIDUS => state = State::Escaping,
                _ if is_control_character(chr) => {
                    return (
                        Err(format!("Control character \"{}\" should be escaped", chr)),
                        ptr,
                    )
                }
                _ => state = State::PlainText,
            },
            State::Escaping => match chr {
                SP_QUOTE
                | SP_REVERSE_SOLIDUS
                | SP_SOLIDUS
                | SP_BACKSPACE
                | SP_FORM_FEED
                | SP_LINE_FEED
                | SP_CARRIAGE_RETURN
                | SP_CHARACTER_TABULATION => state = State::PlainText,
                SP_UNICODE => {
                    state = State::Unicode;
                }
                _ => return (Err(format!("Invalid escaping character: {:?}", chr)), ptr),
            },
            State::Unicode => {
                if !is_hex_digit(chr) {
                    return (Err(format!("Invalid unicode sequence: {:?}", chr)), ptr);
                }

                unicode_len += 1;
                if unicode_len == 4 {
                    unicode_len = 0;
                    state = State::PlainText;
                }
            }
        }

        ptr += 1;
    }
}

fn validate_true(document: &UTF8Reader, start: usize) -> (Result<(), String>, usize) {
    let segment = document.look_ahead(start, 4);
    match segment {
        UTF8ReaderResult::OutOfBoundError(i) => {
            return (Err(format!("Incomplete literal name \"true\"",)), i);
        }
        UTF8ReaderResult::Ok(name) => {
            if name == LN_TRUE {
                return (Ok(()), 4);
            } else {
                return (
                    Err(format!(
                        "It seems to be the plain value \"true\", but got \"{}\"",
                        name
                    )),
                    4,
                );
            }
        }
    }
}

fn validate_false(document: &UTF8Reader, start: usize) -> (Result<(), String>, usize) {
    let segment = document.look_ahead(start, 5);
    match segment {
        UTF8ReaderResult::OutOfBoundError(i) => {
            return (Err(format!("Incomplete literal name \"false\"",)), i);
        }
        UTF8ReaderResult::Ok(name) => {
            if name == LN_FALSE {
                return (Ok(()), 5);
            } else {
                return (
                    Err(format!(
                        "It seems to be the plain value \"false\", but got \"{}\"",
                        name
                    )),
                    5,
                );
            }
        }
    }
}

fn validate_null(document: &UTF8Reader, start: usize) -> (Result<(), String>, usize) {
    let segment = document.look_ahead(start, 4);
    match segment {
        UTF8ReaderResult::OutOfBoundError(i) => {
            return (Err(format!("Incomplete literal name \"null\"",)), i);
        }
        UTF8ReaderResult::Ok(name) => {
            if name == LN_NULL {
                return (Ok(()), 4);
            } else {
                return (
                    Err(format!(
                        "It seems to be the plain value \"null\", but got \"{}\"",
                        name
                    )),
                    4,
                );
            }
        }
    }
}

fn is_insignificant_whitespace(chr: &str) -> bool {
    match chr {
        WS_CHARACTER_TABULATION | WS_LINE_FEED | WS_CARRIAGE_RETURN | WS_SPACE => true,
        _ => false,
    }
}
