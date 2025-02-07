use std::io::Read;
use std::{fs, io, str};
use voile::syntax::surf::{parse_str_err_printed, Decl};

fn read_file(file_arg: &str) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(file_arg)?;
    let mut file_content =
        Vec::with_capacity(file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0));
    file.read_to_end(&mut file_content)?;
    Ok(file_content)
}

pub fn parse_file(file_arg: &str) -> Option<Vec<Decl>> {
    // If cannot read input, return.
    let file_content = match read_file(file_arg) {
        Ok(c) => c,
        Err(io_err) => {
            eprintln!("Cannot read `{}`: {}", file_arg, io_err);
            return None;
        }
    };
    // Read file
    let file_content_utf8 = str::from_utf8(file_content.as_slice()).unwrap();
    // Parse
    parse_str_err_printed(file_content_utf8).ok()
}
