pub fn uint(data: &[u8]) -> usize {
    let bytes = data.len();
    let mut value: usize = 0;

    for i in 1..bytes+1 {
        let byte = data[bytes - i] as usize;
        value = (value << 8) | byte;
    }

    value
}

#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn test_byte() {
        let buf = [0xf8];
        assert_eq!(uint(&buf), 0xf8);
    }


    #[test]
    fn test_word() {
        let buf = [0xf8, 0x09];
        assert_eq!(uint(&buf), 0x09f8);
    }

    #[test]
    fn test_dword() {
        let buf = [0xf8, 0x09, 0x34, 0x12];
        assert_eq!(uint(&buf), 0x123409f8);
    }
}
