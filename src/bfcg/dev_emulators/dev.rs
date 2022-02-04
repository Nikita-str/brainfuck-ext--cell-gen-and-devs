
pub trait Dev{
    // TODO: if need add [mut_]{read_byte, write_byte}

    fn read_byte(&mut self) -> u8;
    fn write_byte(&mut self, byte: u8);

    fn test_can_read_byte(&self) -> bool;

    fn have_error(&self) -> bool;

    /// for return say that we in inf loop
    /// 
    /// std use case:
    /// ```
    /// byte = self.read_byte();
    /// if self.have_error() { /*err handle*/ }
    /// if self.in_infinity_state() { /*inf-state handle*/ }
    /// /*byte handle*/
    /// ```
    fn in_infinity_state(&self) -> bool;
}
