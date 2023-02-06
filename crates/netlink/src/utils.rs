pub fn align_of(len: usize, align_to: usize) -> usize {
    (len + align_to - 1) & !(align_to - 1)
}

pub fn zero_terminated(s: &str) -> Vec<u8> {
    let mut v = s.as_bytes().to_vec();
    v.push(0);
    v
}
