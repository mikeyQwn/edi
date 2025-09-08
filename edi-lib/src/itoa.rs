pub fn itoa_into(mut num: u64, buffer: &mut [u8; 20]) -> &[u8] {
    let mut i = buffer.len();
    loop {
        i -= 1;
        buffer[i] = b'0' + (num % 10) as u8;
        num /= 10;
        if num == 0 {
            break;
        }
    }

    &buffer[i..]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero() {
        let mut buf = [0u8; 20];
        let s = itoa_into(0, &mut buf);
        assert_eq!(s, b"0");
    }

    #[test]
    fn single_digit() {
        let mut buf = [0u8; 20];
        let s = itoa_into(7, &mut buf);
        assert_eq!(s, b"7");
    }

    #[test]
    fn multiple_digits() {
        let mut buf = [0u8; 20];
        let s = itoa_into(42, &mut buf);
        assert_eq!(s, b"42");
    }

    #[test]
    fn large_number() {
        let mut buf = [0u8; 20];
        let s = itoa_into(1234567890, &mut buf);
        assert_eq!(s, b"1234567890");
    }

    #[test]
    fn max_u64() {
        let mut buf = [0u8; 20];
        let s = itoa_into(u64::MAX, &mut buf);
        assert_eq!(s, b"18446744073709551615");
    }

    #[test]
    fn leading_zeroes_not_included() {
        let mut buf = [b'x'; 20];
        let s = itoa_into(123, &mut buf);
        assert_eq!(s, b"123");
    }
}
