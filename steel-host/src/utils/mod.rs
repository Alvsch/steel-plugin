use wasmparser::{Parser, Payload};

pub mod discover;
pub mod memory;
pub mod sorting;

pub fn read_custom_section<'a>(
    bytes: &'a [u8],
    name: &str,
) -> wasmparser::Result<Option<&'a [u8]>> {
    for payload in Parser::new(0).parse_all(bytes) {
        match payload? {
            Payload::CustomSection(reader) if reader.name() == name => {
                return Ok(Some(reader.data()));
            }
            _ => {}
        }
    }
    Ok(None)
}
