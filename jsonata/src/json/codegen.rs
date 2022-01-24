use std::io;
use std::io::Write;
use std::ptr;

use super::number::Number;
use super::util::print_dec;
use crate::value::Value;

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
pub trait Generator {
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
    fn write_number(&mut self, num: &Number) -> io::Result<()> {
        if num.is_nan() {
            return self.write(b"null");
        }
        let (positive, mantissa, exponent) = num.as_parts();
        unsafe { print_dec::write(self.get_writer(), positive, mantissa, exponent) }
    }

    #[inline(always)]
    fn write_object(&mut self, object: &Value) -> io::Result<()> {
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

    fn write_json(&mut self, json: &Value) -> io::Result<()> {
        match *json {
            Value::Null => self.write(b"null"),
            Value::String(ref string) => self.write_string(string),
            Value::Number(ref number) => self.write_number(number),
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
            _ => Ok(()),
        }
    }
}

/// In-Memory Generator, this uses a Vec to store the JSON result.
pub struct DumpGenerator {
    code: Vec<u8>,
}

impl DumpGenerator {
    pub fn new() -> Self {
        DumpGenerator {
            code: Vec::with_capacity(1024),
        }
    }

    pub fn consume(self) -> String {
        // Original strings were unicode, numbers are all ASCII,
        // therefore this is safe.
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

impl Default for DumpGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for DumpGenerator {
    type T = Vec<u8>;

    fn write(&mut self, slice: &[u8]) -> io::Result<()> {
        extend_from_slice(&mut self.code, slice);
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
pub struct PrettyGenerator {
    code: Vec<u8>,
    dent: u16,
    spaces_per_indent: u16,
}

impl PrettyGenerator {
    pub fn new(spaces: u16) -> Self {
        PrettyGenerator {
            code: Vec::with_capacity(1024),
            dent: 0,
            spaces_per_indent: spaces,
        }
    }

    pub fn consume(self) -> String {
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

impl Generator for PrettyGenerator {
    type T = Vec<u8>;

    #[inline(always)]
    fn write(&mut self, slice: &[u8]) -> io::Result<()> {
        extend_from_slice(&mut self.code, slice);
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
        extend_from_slice(&mut self.code, slice);
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

/// Writer Generator, this uses a custom writer to store the JSON result.
pub struct WriterGenerator<'a, W: 'a + Write> {
    writer: &'a mut W,
}

impl<'a, W> WriterGenerator<'a, W>
where
    W: 'a + Write,
{
    pub fn new(writer: &'a mut W) -> Self {
        WriterGenerator { writer }
    }
}

impl<'a, W> Generator for WriterGenerator<'a, W>
where
    W: Write,
{
    type T = W;

    #[inline(always)]
    fn get_writer(&mut self) -> &mut W {
        &mut self.writer
    }

    #[inline(always)]
    fn write_min(&mut self, _: &[u8], min: u8) -> io::Result<()> {
        self.writer.write_all(&[min])
    }
}

/// Pretty Writer Generator, this uses a custom writer to store the JSON result and add indent.
pub struct PrettyWriterGenerator<'a, W: 'a + Write> {
    writer: &'a mut W,
    dent: u16,
    spaces_per_indent: u16,
}

impl<'a, W> PrettyWriterGenerator<'a, W>
where
    W: 'a + Write,
{
    pub fn new(writer: &'a mut W, spaces: u16) -> Self {
        PrettyWriterGenerator {
            writer,
            dent: 0,
            spaces_per_indent: spaces,
        }
    }
}

impl<'a, W> Generator for PrettyWriterGenerator<'a, W>
where
    W: Write,
{
    type T = W;

    #[inline(always)]
    fn get_writer(&mut self) -> &mut W {
        &mut self.writer
    }

    #[inline(always)]
    fn write_min(&mut self, slice: &[u8], _: u8) -> io::Result<()> {
        self.writer.write_all(slice)
    }

    fn new_line(&mut self) -> io::Result<()> {
        self.write_char(b'\n')?;
        for _ in 0..(self.dent * self.spaces_per_indent) {
            self.write_char(b' ')?;
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

// From: https://github.com/dtolnay/fastwrite/blob/master/src/lib.rs#L68
//
// LLVM is not able to lower `Vec::extend_from_slice` into a memcpy, so this
// helps eke out that last bit of performance.
#[inline]
fn extend_from_slice(dst: &mut Vec<u8>, src: &[u8]) {
    let dst_len = dst.len();
    let src_len = src.len();

    dst.reserve(src_len);

    unsafe {
        // We would have failed if `reserve` overflowed
        dst.set_len(dst_len + src_len);

        ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr().add(dst_len), src_len);
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

        let mut generator = DumpGenerator::new();
        generator.write_string(&s).unwrap();
    }

    #[test]
    fn should_not_panic_on_bad_bytes_2() {
        let data = b"\x48\x48\x48\x57\x03\xE8\x48\x48\xE8\x03\x8F\x48\x29\x48\x48";
        let s = unsafe { String::from_utf8_unchecked(data.to_vec()) };

        let mut generator = DumpGenerator::new();
        generator.write_string(&s).unwrap();
    }
}
