use std::path::Path;

pub fn get_raw_image_data_from_file(path: impl AsRef<Path>) -> Result<Box<[u8]>, &'static str> {
    let path = path.as_ref();
    if let Some("png") = path.extension().map(|s| s.to_str()).flatten() {
    } else {
        return Err("file must be a png");
    }

    let data = std::fs::read(path).map_err(|_| "file does not exist")?;

    parse_png(&data)
}

fn parse_png(data: &[u8]) -> Result<Box<[u8]>, &'static str> {
    let data = data.into_iter().copied();
    if !data.expect_data([
        '\u{89}' as u8,
        '\u{50}' as u8,
        '\u{4E}' as u8,
        '\u{47}' as u8,
        '\u{0D}' as u8,
        '\u{0A}' as u8,
        '\u{1A}' as u8,
        '\u{0A}' as u8,
    ]) {
        return Err("header error");
    }
    todo!();
}

trait Expect<T, E, V>
where
    T: PartialEq,
    E: IntoIterator<Item = V>,
    V: Into<T>,
{
    fn expect_data(self, expected: E) -> bool;
}

impl<I, T, E, V> Expect<T, E, V> for I
where
    T: PartialEq,
    I: Iterator<Item = T>,
    E: IntoIterator<Item = V>,
    V: Into<T>,
{
    fn expect_data(mut self, expected: E) -> bool {
        for item in expected.into_iter() {
            match self.next() {
                Some(v) if v != item.into() => return false,
                None => return false,
                _ => (),
            }
        }
        true
    }
}
