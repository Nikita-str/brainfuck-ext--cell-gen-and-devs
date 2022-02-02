use std::collections::LinkedList;


pub const MIN_BIG_BYTE: u8 = 0x80;
pub const MAX_SMALL_BYTE: u8 = !MIN_BIG_BYTE;
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

pub fn std_se_decoding<Iter: Iterator<Item = u8>>(iter: Iter) -> Option<usize> {
    let mut ret = 0;
    let mut cur_sh = 0;

    let mut started = false;
    let mut previous_is_last = false; 
    for mut x in iter {
        started = true;
        if previous_is_last && x != 0x00 { return None }
        if x < MIN_BIG_BYTE { previous_is_last = true; } 
        else { x = x & MAX_SMALL_BYTE; }

        ret = ret | ((x as usize) << cur_sh);
        cur_sh += SHIFT;
    }

    if !started { return None }
    if !previous_is_last { return None }
    Some(ret)
}