// This JSON dumping code was stolen from [Maciej Hirsz's](https://github.com/maciejhirsz) excellent
// [json crate](https://github.com/maciejhirsz/json-rust), and modified to work on our custom internal
// value.
//
// The original code is licensed in the same way as this crate.

use std::io;
use std::io::Write;

use super::Value;

const QU: u8 = b'"';
const BS: u8 = b'\\';
const BB: u8 = b'b';
const TT: u8 = b't';
const NN: u8 = b'n';
const FF: u8 = b'f';
const RR: u8 = b'r';
const UU: u8 = b'u';
const __: u8 = 0;

// Look up table for characters that need escaping in a product string
static ESCAPED: [u8; 256] = [
    // 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
    __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
    __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];

/// Default trait for serializing JSONValue into string.
pub trait Serializer {
    type T: Write;

    fn get_writer(&mut self) -> &mut Self::T;

    #[inline(always)]
    fn write(&mut self, slice: &[u8]) -> io::Result<()> {
        self.get_writer().write_all(slice)
    }

    #[inline(always)]
    fn write_char(&mut self, ch: u8) -> io::Result<()> {
        self.get_writer().write_all(&[ch])
    }

    fn write_min(&mut self, slice: &[u8], min: u8) -> io::Result<()>;

    #[inline(always)]
    fn new_line(&mut self) -> io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn indent(&mut self) {}

    #[inline(always)]
    fn dedent(&mut self) {}

    #[inline(never)]
    fn write_string_complex(&mut self, string: &str, mut start: usize) -> io::Result<()> {
        self.write(&string.as_bytes()[..start])?;

        for (index, ch) in string.bytes().enumerate().skip(start) {
            let escape = ESCAPED[ch as usize];
            if escape > 0 {
                self.write(&string.as_bytes()[start..index])?;
                self.write(&[b'\\', escape])?;
                start = index + 1;
            }
            if escape == b'u' {
                write!(self.get_writer(), "{:04x}", ch)?;
            }
        }
        self.write(&string.as_bytes()[start..])?;

        self.write_char(b'"')
    }

    #[inline(always)]
    fn write_string(&mut self, string: &str) -> io::Result<()> {
        self.write_char(b'"')?;

        for (index, ch) in string.bytes().enumerate() {
            if ESCAPED[ch as usize] > 0 {
                return self.write_string_complex(string, index);
            }
        }

        self.write(string.as_bytes())?;
        self.write_char(b'"')
    }

    #[inline(always)]
    fn write_number(&mut self, number: f64) -> io::Result<()> {
        const MAX_SIGNIFICANT_DIGITS: usize = 15;

        if number.is_finite() {
            let mut buffer = dtoa::Buffer::new();
            let formatted = buffer.format_finite(number).as_bytes();

            // JSONata uses JSON.stringify with Number.toPrecision(15) to format numbers.
            //
            // dtoa gets us close to the behaviour of JSON.stringify, in particular for
            // switching to scientific notation (which Rust format! doesn't do), but dtoa
            // doesn't support specifying a number of significant digits.
            //
            // This craziness limits the number of significant digits and trims off trailing
            // zeroes in the fraction by doing string manipulation.
            //
            // It's not pretty, and I'm sure there's a better way to do this.
            let mut split_iter = formatted.split(|b| *b == b'.');
            let whole = split_iter.next();
            let fraction = split_iter.next();
            if let Some(whole) = whole {
                self.write(whole)?;
                if whole.len() < MAX_SIGNIFICANT_DIGITS {
                    if let Some(fraction) = fraction {
                        let fraction_length =
                            usize::min(MAX_SIGNIFICANT_DIGITS - whole.len(), fraction.len());
                        if fraction_length > 0 {
                            let fraction = unsafe {
                                std::str::from_utf8_unchecked(&fraction[0..fraction_length])
                                    .trim_end_matches('0')
                            };
                            if !fraction.is_empty() {
                                self.write_char(b'.')?;
                                self.write(fraction.as_bytes())?;
                            }
                        }
                    }
                }
            } else {
                self.write(formatted)?;
            }

            Ok(())
        } else {
            self.write(b"null")
        }
    }

    #[inline(always)]
    fn write_object<'a>(&mut self, object: &'a Value<'a>) -> io::Result<()> {
        self.write_char(b'{')?;
        let mut iter = object.entries();

        if let Some((key, value)) = iter.next() {
            self.indent();
            self.new_line()?;
            self.write_string(key)?;
            self.write_min(b": ", b':')?;
            self.write_json(value)?;
        } else {
            self.write_char(b'}')?;
            return Ok(());
        }

        for (key, value) in iter {
            self.write_char(b',')?;
            self.new_line()?;
            self.write_string(key)?;
            self.write_min(b": ", b':')?;
            self.write_json(value)?;
        }

        self.dedent();
        self.new_line()?;
        self.write_char(b'}')
    }

    fn write_json<'a>(&mut self, json: &'a Value<'a>) -> io::Result<()> {
        match *json {
            Value::Undefined => Ok(()),
            Value::Null => self.write(b"null"),
            Value::String(ref string) => self.write_string(string),
            Value::Number(n) => self.write_number(n),
            Value::Bool(true) => self.write(b"true"),
            Value::Bool(false) => self.write(b"false"),
            Value::Array(..) => {
                self.write_char(b'[')?;
                let mut iter = json.members();

                if let Some(item) = iter.next() {
                    self.indent();
                    self.new_line()?;
                    self.write_json(item)?;
                } else {
                    self.write_char(b']')?;
                    return Ok(());
                }

                for item in iter {
                    self.write_char(b',')?;
                    self.new_line()?;
                    self.write_json(item)?;
                }

                self.dedent();
                self.new_line()?;
                self.write_char(b']')
            }
            Value::Object(..) => self.write_object(json),
            Value::Lambda { .. } | Value::NativeFn { .. } => self.write(b"\"\""),
        }
    }
}

/// In-Memory Generator, this uses a Vec to store the JSON result.
pub struct DumpSerializer {
    code: Vec<u8>,
}

impl DumpSerializer {
    pub fn new() -> Self {
        DumpSerializer {
            code: Vec::with_capacity(1024),
        }
    }

    pub fn consume(self) -> String {
        // Original strings were unicode, numbers are all ASCII,
        // therefore this is safe.
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

impl Default for DumpSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serializer for DumpSerializer {
    type T = Vec<u8>;

    fn write(&mut self, slice: &[u8]) -> io::Result<()> {
        self.code.extend_from_slice(slice);
        Ok(())
    }

    #[inline(always)]
    fn write_char(&mut self, ch: u8) -> io::Result<()> {
        self.code.push(ch);
        Ok(())
    }

    #[inline(always)]
    fn get_writer(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    #[inline(always)]
    fn write_min(&mut self, _: &[u8], min: u8) -> io::Result<()> {
        self.code.push(min);
        Ok(())
    }
}

/// Pretty In-Memory Generator, this uses a Vec to store the JSON result and add indent.
pub struct PrettySerializer {
    code: Vec<u8>,
    dent: u16,
    spaces_per_indent: u16,
}

impl PrettySerializer {
    pub fn new(spaces: u16) -> Self {
        PrettySerializer {
            code: Vec::with_capacity(1024),
            dent: 0,
            spaces_per_indent: spaces,
        }
    }

    pub fn consume(self) -> String {
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

impl Serializer for PrettySerializer {
    type T = Vec<u8>;

    #[inline(always)]
    fn write(&mut self, slice: &[u8]) -> io::Result<()> {
        self.code.extend_from_slice(slice);
        Ok(())
    }

    #[inline(always)]
    fn write_char(&mut self, ch: u8) -> io::Result<()> {
        self.code.push(ch);
        Ok(())
    }

    #[inline(always)]
    fn get_writer(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    #[inline(always)]
    fn write_min(&mut self, slice: &[u8], _: u8) -> io::Result<()> {
        self.code.extend_from_slice(slice);
        Ok(())
    }

    fn new_line(&mut self) -> io::Result<()> {
        self.code.push(b'\n');
        for _ in 0..(self.dent * self.spaces_per_indent) {
            self.code.push(b' ');
        }
        Ok(())
    }

    fn indent(&mut self) {
        self.dent += 1;
    }

    fn dedent(&mut self) {
        self.dent -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // found while fuzzing the DumpGenerator
    #[test]
    fn should_not_panic_on_bad_bytes() {
        let data = [0, 12, 128, 88, 64, 99].to_vec();
        let s = unsafe { String::from_utf8_unchecked(data) };

        let mut generator = DumpSerializer::new();
        generator.write_string(&s).unwrap();
    }

    #[test]
    fn should_not_panic_on_bad_bytes_2() {
        let data = b"\x48\x48\x48\x57\x03\xE8\x48\x48\xE8\x03\x8F\x48\x29\x48\x48";
        let s = unsafe { String::from_utf8_unchecked(data.to_vec()) };

        let mut generator = DumpSerializer::new();
        generator.write_string(&s).unwrap();
    }
}
