
pub trait BackwardMove { fn back(&mut self); }

impl<I, T> BackwardMove for IterWithAutoBack<I, T> 
where
    T: Copy,
    I: std::iter::Iterator<Item = T>
{
    fn back(&mut self) { self.back() }
}

impl<I, T, const N: usize> BackwardMove for IterWithAutoBackN<I, T, N> 
where
    T: Copy,
    I: std::iter::Iterator<Item = T>
{
    fn back(&mut self) { self.back() }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// struct that impl BackwardMove but supposed never back 
pub struct IterPanicOnBack<I> { iter: I }

impl<I> IterPanicOnBack<I> 
where I: std::iter::Iterator
{
    pub fn new(iter: I) -> Self { Self { iter } }
}

impl<I> BackwardMove for IterPanicOnBack<I> 
where I: std::iter::Iterator
{
    fn back(&mut self) { panic!("panic cause we cant use back on IterPanicOnBack") }
}

impl<I> Iterator for IterPanicOnBack<I>
where
    I: std::iter::Iterator
{
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> { self.iter.next() }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub struct IterWithAutoBack<I, T>
where
    T: Copy,
    I: std::iter::Iterator<Item = T>
{
    iter: I,
    back: Vec<T>,
    cur_back_pos: Option<usize>, 
}

impl<I, T> IterWithAutoBack<I, T> 
where
    T: Copy,
    I: std::iter::Iterator<Item = T>
{
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            back: Vec::new(),
            cur_back_pos: None,
        }
    }

    pub fn back(&mut self) {
        let index = match self.cur_back_pos {
            None => 0,
            Some(index) => index + 1,
        };
        if index == self.back.len() { panic!("too many backward iterate") }
        self.cur_back_pos = Some(index);
    }
}

impl<I, T> Iterator for IterWithAutoBack<I, T>
where
    T: Copy,
    I: std::iter::Iterator<Item = T>
{
    type Item = T;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if let Some(x) = self.cur_back_pos {
            self.cur_back_pos = if x == 0 { None } else { Some(x - 1) };
            Some(self.back[self.back.len() - 1 - x])
        } else {
            let ret = self.iter.next();
            if let Some(x) = ret { self.back.push(x) }
            ret
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub struct IterWithAutoBackN<I, T, const N: usize>
where
    T: Copy,
    I: std::iter::Iterator<Item = T>
{
    iter: I,

    back: [Option<T>; N],
    cur_back_start: usize,
    cur_back_end: usize,

    back_index: usize,
}

impl<I, T, const N: usize> IterWithAutoBackN<I, T, N>
where
    T: Copy,
    I: std::iter::Iterator<Item = T>
{
    pub fn new(iter: I) -> Self {
        Self {
            back: [None; N],
            cur_back_start: 0,
            cur_back_end: 0,

            back_index: 0,
            iter,
        }
    }

    pub fn back(&mut self) {
        if N <= self.cur_back_len() { panic!("too much backward") }
        self.back_index += 1;
        assert!(self.back[self.get_back_index()].is_some());
    }

    fn cur_back_len(&self) -> usize {
        if self.cur_back_end <= self.cur_back_start { self.cur_back_end + N - self.cur_back_start - 1 }
        else { self.cur_back_end - self.cur_back_start - 1 }
    }
    fn get_back_index(&self) -> usize { (self.cur_back_end + N - self.back_index) % N }
}

impl<I, T, const N: usize> Iterator for IterWithAutoBackN<I, T, N>
where
    T: Copy,
    I: std::iter::Iterator<Item = T>
{
    type Item = T;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.back_index != 0 {
            let index = self.get_back_index();
            self.back_index -= 1;
            let ret = self.back[index];
            assert!(ret.is_some());
            ret
        } else {        
            let ret = self.iter.next();
            if (N != 0) && (ret.is_some()) {
                self.back[self.cur_back_end] = ret;
                if self.cur_back_start == 0 && self.cur_back_end < N - 1 { self.cur_back_end += 1; }
                else {
                    self.cur_back_start = (self.cur_back_start + 1) % N;
                    self.cur_back_end = (self.cur_back_end + 1) % N;
                }
            }
            ret
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// [+] TESTS

#[cfg(test)]
mod tests {
    use super::{IterWithAutoBackN, BackwardMove};

    #[allow(unused)]
    fn pr<const N: usize>(iter: &IterWithAutoBackN<std::str::Chars, char, N>) {
        println!("{:?}", iter.back);
        println!("index = {}; start = {}; end = {};", iter.back_index, iter.cur_back_start, iter.cur_back_end);
    }

    pub(in super)
    fn test_01_helper_g<Backward: BackwardMove + Iterator<Item = char>>(iter: &mut Backward) {
        assert_eq!(Some('0'), iter.next());
        assert_eq!(Some('1'), iter.next());
        iter.back();
        assert_eq!(Some('1'), iter.next());
        assert_eq!(Some('2'), iter.next());

        iter.back();
        assert_eq!(Some('2'), iter.next());
        iter.back();
        assert_eq!(Some('2'), iter.next());

        iter.back();
        iter.back();
        assert_eq!(Some('1'), iter.next());
        assert_eq!(Some('2'), iter.next());
        iter.back();
        assert_eq!(Some('2'), iter.next());

        iter.back();
        iter.back();
        iter.back();
        assert_eq!(Some('0'), iter.next());
        assert_eq!(Some('1'), iter.next());
        assert_eq!(Some('2'), iter.next());
        assert_eq!(Some('3'), iter.next());
        assert_eq!(Some('4'), iter.next());
        assert_eq!(Some('5'), iter.next());
        assert_eq!(Some('6'), iter.next());
        iter.back();
        assert_eq!(Some('6'), iter.next());
        assert_eq!(Some('7'), iter.next());
        
        iter.back();
        assert_eq!(Some('7'), iter.next());
    }

    fn test_01_helper() -> IterWithAutoBackN<std::str::Chars<'static>, char, 4> {
        let x = "0123456789";
        let mut iter = IterWithAutoBackN::<_, _, 4>::new(x.chars());
        test_01_helper_g(&mut iter);
        iter
    }

    #[test]
    fn test_01a() {
        let mut iter = test_01_helper();
        iter.back();
        iter.back();
        iter.back();
        assert_eq!(Some('5'), iter.next());
        assert_eq!(Some('6'), iter.next());
        assert_eq!(Some('7'), iter.next());
    }

    #[test]
    fn test_01b() {
        let mut iter = test_01_helper();
        iter.back();
        iter.back();
        iter.back();
        iter.back();
        assert_eq!(Some('4'), iter.next());
        assert_eq!(Some('5'), iter.next());
        assert_eq!(Some('6'), iter.next());
        iter.back();
        assert_eq!(Some('6'), iter.next());
    }

    #[test]
    #[should_panic]
    fn test_01c() {
        let mut iter = test_01_helper();
        iter.back();
        iter.back();
        iter.back();
        iter.back();
        iter.back();
    }

    #[test]
    #[should_panic]
    fn test_02_a00() {
        let x = "0123456789";
        let mut iter = IterWithAutoBackN::<_, _, 4>::new(x.chars());
        iter.back();
    }

    #[test]
    fn test_02_a01() {
        let x = "0123456789";
        let mut iter = IterWithAutoBackN::<_, _, 4>::new(x.chars());
        for _ in 0..17 {
            assert_eq!(Some('0'), iter.next());
            iter.back();
        }
    }

    fn test_02_a02_helper(to: usize) {
        const N:usize = 4;
        let x = "0123456789";
        let mut iter = IterWithAutoBackN::<_, _, N>::new(x.chars());
        for index in 0..to {
            assert_eq!(char::from_digit(index as u32, 10), iter.next());
        }
        for _ in 0..N { iter.back() }
    }

    #[test]
    fn test_02_a02() {
        const N:usize = 4;
        test_02_a02_helper(N);
        test_02_a02_helper(N + 1);
        test_02_a02_helper(N + 2);
        test_02_a02_helper(N + 3);
    }

    
    fn test_02_b02_helper(to_1: usize, to_2: usize) {
        const N:usize = 4;
        let x = "0123456789ABCDEFGH";
        let mut iter = IterWithAutoBackN::<_, _, N>::new(x.chars());
        for _ in 0..to_1 { iter.next(); }
        for _ in 0..to_2 { iter.back() }
    }

    #[test]
    fn test_02_b02() {
        for to_2 in 2..=4 {
            for to_1 in to_2..15 { 
                test_02_b02_helper(to_1, to_2) 
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_02_a03() {
        let x = "0123456789";
        let mut iter = IterWithAutoBackN::<_, _, 4>::new(x.chars());
        assert_eq!(Some('0'), iter.next());
        iter.back();
        iter.back();
    }

    fn test_02_01_helper() -> IterWithAutoBackN<std::str::Chars<'static>, char, 4> {
        let x = "0123456789";
        let mut iter = IterWithAutoBackN::<_, _, 4>::new(x.chars());
        assert_eq!(Some('0'), iter.next());
        assert_eq!(Some('1'), iter.next());
        assert_eq!(Some('2'), iter.next());
        assert_eq!(Some('3'), iter.next());
        iter.back();
        iter.back();
        iter.back();
        iter.back();
        iter
    }

    #[test]
    fn test_02_01() { test_02_01_helper(); }

    #[test]
    #[should_panic]
    fn test_02_err_01() {
        let mut iter = test_02_01_helper();
        iter.back();
    }
    
    fn test_02_02_helper() -> IterWithAutoBackN<std::str::Chars<'static>, char, 4> {
        let mut iter = test_02_01_helper();
        assert_eq!(Some('0'), iter.next());
        assert_eq!(Some('1'), iter.next());
        assert_eq!(Some('2'), iter.next());
        iter.back();
        iter.back();
        assert_eq!(Some('1'), iter.next());
        iter.back();
        iter.back();
        iter
    }

    #[test]
    fn test_02_02() { test_02_02_helper(); }
    
    #[test]
    #[should_panic]
    fn test_02_err_02() {
        let mut iter = test_02_02_helper();
        iter.back();
    }

    #[test]
    fn test_03() {
        const N:usize = 4;
        let x = "0123456789ABCDEFGH";
        let mut iter = IterWithAutoBackN::<_, _, N>::new(x.chars());
        assert_eq!(Some('0'), iter.next());
        assert_eq!(Some('1'), iter.next());
        assert_eq!(Some('2'), iter.next());
        assert_eq!(Some('3'), iter.next());
        iter.back();
        iter.back();
        assert_eq!(Some('2'), iter.next());
        assert_eq!(Some('3'), iter.next());
        assert_eq!(Some('4'), iter.next());
        assert_eq!(Some('5'), iter.next());
        assert_eq!(Some('6'), iter.next());
        assert_eq!(Some('7'), iter.next());
        assert_eq!(Some('8'), iter.next());
        assert_eq!(Some('9'), iter.next());
        iter.back();
        iter.back();
        iter.back();
        assert_eq!(Some('7'), iter.next());
        assert_eq!(Some('8'), iter.next());
        assert_eq!(Some('9'), iter.next());
        assert_eq!(Some('A'), iter.next());
        assert_eq!(Some('B'), iter.next());
        iter.back();
        iter.back();
        iter.back();
        iter.back();
        assert_eq!(Some('8'), iter.next());
        assert_eq!(Some('9'), iter.next());
        assert_eq!(Some('A'), iter.next());
        assert_eq!(Some('B'), iter.next());
        assert_eq!(Some('C'), iter.next());
    }

    pub(in super)
    fn test_04_g<Backward: BackwardMove + Iterator<Item = char>>(iter: &mut Backward) {
        assert_eq!(Some('0'), iter.next());
        assert_eq!(Some('1'), iter.next());
        assert_eq!(Some('2'), iter.next());
        assert_eq!(Some('3'), iter.next());
        iter.back();
        assert_eq!(Some('3'), iter.next());
        assert_eq!(Some('4'), iter.next());
        assert_eq!(Some('5'), iter.next());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
        iter.back();
        assert_eq!(Some('5'), iter.next());
        assert_eq!(None, iter.next());
        iter.back();
        iter.back();
        assert_eq!(Some('4'), iter.next());
        assert_eq!(Some('5'), iter.next());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
        iter.back();
        iter.back();
        iter.back();
        assert_eq!(Some('3'), iter.next());
        assert_eq!(Some('4'), iter.next());
        assert_eq!(Some('5'), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_04() {
        const N:usize = 4;
        let x = "012345";
        let mut iter = IterWithAutoBackN::<_, _, N>::new(x.chars());
        test_04_g(&mut iter);
    }
}


#[cfg(test)]
mod tests_a {
    use super::IterWithAutoBack;
    
    #[test]
    fn test_01() {
        let x = "0123456789";
        let mut iter = IterWithAutoBack::new(x.chars());
        super::tests::test_01_helper_g(&mut iter);
    }


    #[test]
    fn test_02_d0() {
        let x = "0123456789";
        let mut iter = IterWithAutoBack::new(x.chars());
        assert_eq!(Some('0'), iter.next());
        assert_eq!(Some('1'), iter.next());
        iter.back();
        assert_eq!(Some('1'), iter.next());
        iter.back();
        iter.back();
    }

    #[test]
    #[should_panic]
    fn test_02_d1() {
        let x = "0123456789";
        let mut iter = IterWithAutoBack::new(x.chars());
        assert_eq!(Some('0'), iter.next());
        assert_eq!(Some('1'), iter.next());
        iter.back();
        assert_eq!(Some('1'), iter.next());
        iter.back();
        iter.back();
        iter.back();
    }
    
    #[test]
    #[should_panic]
    fn test_02_d2() {
        let x = "0123456789";
        let mut iter = IterWithAutoBack::new(x.chars());
        iter.back();
    }

    fn test_02_b02_helper(to_1: usize, to_2: usize) {
        let x = "0123456789ABCDEFGH";
        let mut iter = IterWithAutoBack::new(x.chars());
        for _ in 0..to_1 { iter.next(); }
        for _ in 0..to_2 { iter.back() }
    }

    #[test]
    fn test_02_b02() {
        for to_2 in 2..=13 {
            for to_1 in to_2..15 { 
                test_02_b02_helper(to_1, to_2) 
            }
        }
    }

    #[test]
    fn test_04() {
        let x = "012345";
        let mut iter = IterWithAutoBack::new(x.chars());
        super::tests::test_04_g(&mut iter);
    }
}