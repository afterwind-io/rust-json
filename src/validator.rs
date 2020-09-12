use super::utils::UTF8Reader;

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

enum JSONValue {
    Object,
    Array,
    Number,
    String,
    Ture,
    False,
    Null,
    Unknown,
}

pub fn validate(document: &UTF8Reader) -> Result<(), String> {
    let length = document.len();

    let mut i = 0;
    while i < length {
        let (result, step) = validate_json_value(document, i);
        i += step;

        if let Err(reason) = result {
            return Err(format!(
                "Validation Error @ 1:{}\nReason: {}",
                i + 1,
                reason
            ));
        }
    }

    return Ok(());
}

fn validate_json_value(document: &UTF8Reader, index: usize) -> (Result<(), String>, usize) {
    return match get_next_json_value_type(document, index) {
        JSONValue::Object => validate_object(document, index),
        JSONValue::Array => validate_array(document, index),
        JSONValue::Number => validate_number(document, index),
        JSONValue::String => validate_string(document, index),
        JSONValue::Ture => validate_true(document, index),
        JSONValue::False => validate_false(document, index),
        JSONValue::Null => validate_null(document, index),
        JSONValue::Unknown => {
            let chr = document.look_ahead(index, 1);
            match chr {
                _ if is_insignificant_whitespace(chr) => (Ok(()), 1),
                _ => {
                    return (Err(format!("Unknown character: \"{}\"", chr)), 1);
                }
            }
        }
    };
}

fn get_next_json_value_type(document: &UTF8Reader, index: usize) -> JSONValue {
    match document.look_ahead(index, 1) {
        ST_LCBRACKET => JSONValue::Object,
        ST_LSBRACKET => JSONValue::Array,
        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | SP_MINUS => JSONValue::Number,
        SP_QUOTE => JSONValue::String,
        LT_TRUE => JSONValue::Ture,
        LT_FALSE => JSONValue::False,
        LT_NULL => JSONValue::Null,
        _ => JSONValue::Unknown,
    }
}

fn validate_object(document: &UTF8Reader, start: usize) -> (Result<(), String>, usize) {
    enum State {
        Begin,
        Key,
        PendingValue,
        Value,
        PendingKey,
    }

    let len = document.len();

    let mut state: State = State::Begin;
    let mut ptr = 0;

    loop {
        let index = start + ptr;
        if index >= len {
            break;
        };

        let chr = document.look_ahead(index, 1);
        match state {
            State::Begin => {
                if chr != ST_LCBRACKET {
                    return (Err(String::from("Object should start with \"{\"")), ptr);
                }
                state = State::Key;
            }
            State::Key => {
                if chr == ST_RCBRACKET {
                    return (Ok(()), ptr + 1);
                }

                if is_insignificant_whitespace(chr) {
                    ptr += 1;
                    continue;
                }

                if chr != SP_QUOTE {
                    return (Err(String::from("Object key should start with \"")), ptr);
                }

                let (result, step) = validate_string(document, start + ptr);
                ptr += step;

                if let Ok(_) = result {
                    state = State::PendingValue;
                    continue;
                } else {
                    return (result, ptr);
                }
            }
            State::PendingValue => match chr {
                ST_COLON => state = State::Value,
                _ if is_insignificant_whitespace(chr) => {}
                _ => {
                    return (
                        Err(format!("Invalid character after object key: \"{}\"", chr)),
                        ptr,
                    )
                }
            },
            State::Value => {
                if is_insignificant_whitespace(chr) {
                    ptr += 1;
                    continue;
                }

                let (result, step) = validate_json_value(document, start + ptr);
                ptr += step;

                if let Ok(_) = result {
                    state = State::PendingKey;
                    continue;
                } else {
                    return (result, ptr);
                }
            }
            State::PendingKey => match chr {
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

    let tail = document.get_tail();
    if tail != ST_RCBRACKET {
        return (Err(format!("Object is not closed before EOF")), len - 1);
    }

    return (Ok(()), ptr);
}

fn validate_array(document: &UTF8Reader, start: usize) -> (Result<(), String>, usize) {
    enum State {
        Begin,
        Value,
        PendingValue,
    }

    let len = document.len();

    let mut state: State = State::Begin;
    let mut ptr = 0;

    loop {
        let index = start + ptr;
        if index >= len {
            break;
        };

        let chr = document.look_ahead(index, 1);
        match state {
            State::Begin => {
                if chr != ST_LSBRACKET {
                    return (Err(String::from("Array should start with \"[\"")), ptr);
                }
                state = State::Value;
            }
            State::Value => {
                if chr == ST_RSBRACKET {
                    return (Ok(()), ptr + 1);
                }

                if is_insignificant_whitespace(chr) {
                    ptr += 1;
                    continue;
                }

                let (result, step) = validate_json_value(document, start + ptr);
                ptr += step;

                if let Ok(_) = result {
                    state = State::PendingValue;
                    continue;
                } else {
                    return (result, ptr);
                }
            }
            State::PendingValue => match chr {
                ST_RSBRACKET => return (Ok(()), ptr + 1),
                ST_COMMA => state = State::Value,
                _ if is_insignificant_whitespace(chr) => {}
                _ => return (Err(format!("Invalid character: \"{}\"", chr)), ptr),
            },
        }

        ptr += 1;
    }

    let tail = document.get_tail();
    if tail != ST_RSBRACKET {
        return (Err(format!("Array is not closed before EOF")), len - 1);
    }

    return (Ok(()), ptr);
}

fn validate_number(document: &UTF8Reader, start: usize) -> (Result<(), String>, usize) {
    enum State {
        Begin,
        LeadingMinus,
        LeadingZero,
        Integer,
        PendingFraction,
        Fraction,
        PendingExponent, // + or -
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

    let len = document.len();

    let mut state: State = State::Begin;
    let mut ptr = 0;

    loop {
        let index = start + ptr;
        if index >= len {
            break;
        };

        let chr = document.look_ahead(index, 1);
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
                "e" | "E" => state = State::PendingExponent,
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
                "e" | "E" => state = State::PendingExponent,
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
                "e" | "E" => state = State::PendingExponent,
                _ if is_valid_demical_number(chr, false) => {}
                _ if is_end_of_number(chr) => return (Ok(()), ptr),
                _ => {
                    return (
                        Err(format!("Invalid character in fraction part: {:?}", chr)),
                        ptr,
                    )
                }
            },
            State::PendingExponent => match chr {
                "+" | "-" => state = State::Exponent,
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
    return (Ok(()), ptr);
}

fn validate_string(document: &UTF8Reader, start: usize) -> (Result<(), String>, usize) {
    enum State {
        Begin,
        PlainText,
        Escaping,
        Unicode,
    }

    let len = document.len();

    let mut state: State = State::Begin;
    let mut ptr = 0;
    let mut unicode_ptr = 0;

    loop {
        let index = start + ptr;
        if index >= len {
            break;
        };

        let chr = document.look_ahead(index, 1);
        match state {
            State::Begin => {
                if chr != SP_QUOTE {
                    return (Err(String::from("String value should start with \"")), ptr);
                }

                state = State::PlainText;
            }
            State::PlainText => {
                match chr {
                    SP_QUOTE => return (Ok(()), ptr + 1),
                    SP_REVERSE_SOLIDUS => state = State::Escaping,
                    // TODO control char
                    _ => state = State::PlainText,
                }
            }
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
                if !is_valid_hex_digits(chr) {
                    return (Err(format!("Invalid unicode sequence: {:?}", chr)), ptr);
                }

                unicode_ptr += 1;
                if unicode_ptr == 4 {
                    unicode_ptr = 0;
                    state = State::PlainText;
                }
            }
        }

        ptr += 1;
    }

    let tail = document.get_tail();
    if tail != SP_QUOTE {
        return (Err(format!("String is not closed before EOF")), len - 1);
    }

    return (Ok(()), ptr);
}

fn validate_true(document: &UTF8Reader, start: usize) -> (Result<(), String>, usize) {
    let segment = document.look_ahead(start, 4);
    if segment == LN_TRUE {
        return (Ok(()), 4);
    } else {
        return (
            Err(format!(
                "It seems to be the plain value \"true\", but got \"{}\"",
                segment
            )),
            4,
        );
    }
}

fn validate_false(document: &UTF8Reader, start: usize) -> (Result<(), String>, usize) {
    let segment = document.look_ahead(start, 5);
    if segment == LN_FALSE {
        return (Ok(()), 5);
    } else {
        return (
            Err(format!(
                "It seems to be the plain value \"false\", but got \"{}\"",
                segment
            )),
            5,
        );
    }
}

fn validate_null(document: &UTF8Reader, start: usize) -> (Result<(), String>, usize) {
    let segment = document.look_ahead(start, 4);
    if segment == LN_NULL {
        return (Ok(()), 4);
    } else {
        return (
            Err(format!(
                "It seems to be the plain value \"null\", but got \"{}\"",
                segment
            )),
            4,
        );
    }
}

fn is_valid_hex_digits(chr: &str) -> bool {
    let c = chr.chars().nth(0).unwrap();
    match c {
        '0'..='9' | 'A'..='F' | 'a'..='f' => true,
        _ => false,
    }
}

fn is_insignificant_whitespace(chr: &str) -> bool {
    match chr {
        WS_CHARACTER_TABULATION | WS_LINE_FEED | WS_CARRIAGE_RETURN | WS_SPACE => true,
        _ => false,
    }
}
