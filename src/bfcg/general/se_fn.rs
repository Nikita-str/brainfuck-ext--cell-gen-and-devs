use std::collections::LinkedList;


const MIN_BIG_BYTE: u8 = 0x80;
const MAX_SMALL_BYTE: u8 = !MIN_BIG_BYTE;
const SHIFT:usize = 7;

pub fn std_se_encoding(mut to_se: usize) -> LinkedList<u8>{
    let mut ret = LinkedList::new();
    if to_se == 0 { ret.push_back(0x00) }
    while to_se > 0 {
        if to_se < (MIN_BIG_BYTE as usize) {
            ret.push_back(to_se as u8)
        } else {
            ret.push_back(MIN_BIG_BYTE | ((to_se & (MAX_SMALL_BYTE as usize)) as u8))
        }
        to_se >>= SHIFT;
    }
    ret
}