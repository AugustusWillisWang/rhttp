use crate::parser::http::TcpLine;

pub fn string_to_chunk(input: &str) -> String {
    let len = input.chars().count();
    let mut pos = 0;
    let mut s = String::new();
    let chunksize = 8;
    
    while pos < len {
        if (len - pos) > chunksize {
            s.push_str(r"8\r\n");
            s.push_str(&input[pos..pos+chunksize]);
            s.push_str(r"\r\n");
            pos += chunksize;
        } else {
            s.push_str(&format!("{}", (len - pos)));
            s.push_str(r"\r\n");
            s.push_str(&input[pos..len]);
            s.push_str(r"\r\n");
            s.push_str(r"0\r\n");
            break
        }
    }
    s
}

pub fn chunklines_to_string(lines: &mut TcpLine) -> String {
    let mut s = String::new();    
    loop {
        let mut _len = 0; // use _ prefix to stop rustc from complaining
        // better design pattern needed

        match lines.next() {
            Some(i) => {
                // read hex
                match usize::from_str_radix(&i.unwrap(), 16) { // FIXME
                    Ok(v) => { 
                        if v == 0 { break }
                        _len = v; 
                    }
                    _ => break // error
                }
            }
            _ => break // error
        }
        // read content
        match lines.next() {
            Some(Ok(j)) => {
                if j.chars().count() == _len {
                    s.push_str(&j);
                } else {
                    println!("chunk len {} and real len {} does not match", _len, j.chars().count());
                    break
                }
            }
            _ => break
        }
    }
    s
}

#[cfg(test)]
mod chunk_test {
    use super::*;
    #[test]
    fn convert_rawbody_to_chunk() {
        let raw_body = r"012345678901234567890123456789"; 
        let right_body =
r"8\r\n01234567\r\n8\r\n89012345\r\n8\r\n67890123\r\n6\r\n456789\r\n0\r\n";
        let body = string_to_chunk(raw_body);
        assert_eq!(body, right_body);
    }
}