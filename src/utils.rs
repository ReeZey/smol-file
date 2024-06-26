const ALPHABET: &str = " abcdefghijklmnopqrstuvwxyz.!?12";

pub(crate) fn char_to_index(search_char: char) -> Option<usize> {
    for (index, char) in ALPHABET.chars().enumerate() {
        if search_char == char {
            return Some(index);
        }
    }
    return None;
}

pub(crate) fn index_to_char(search_index: usize) -> char {
    for (index, char) in ALPHABET.chars().enumerate() {
        if search_index == index {
            return char;
        }
    }
    return ALPHABET.chars().nth(0).unwrap();
}
