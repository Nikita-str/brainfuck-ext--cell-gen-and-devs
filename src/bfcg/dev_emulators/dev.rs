
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

#[macro_export]
macro_rules! dev_std_precheck_read_byte {
    ($dev:ident, $default:ident) => {
        if $dev.in_infinity_state() { return $default }
        if $dev.have_error() {  $dev.infinity = true; return $default }
        if !$dev.test_can_read_byte() { $dev.error = true; return $default }
    };
}


#[macro_export]
macro_rules! dev_std_precheck_write_byte {
    ($dev:ident) => {
        // Dev not blocked on write in any case 
        if $dev.in_infinity_state() || $dev.have_error() { return }
    }
}

#[macro_export]
macro_rules! dev_std_realise_in_inf {
    () => { fn in_infinity_state(&self) -> bool { self.infinity } };
}

#[macro_export]
macro_rules! dev_std_realise_have_error {
    () => { fn have_error(&self) -> bool { self.error | self.inner.error() } };
}