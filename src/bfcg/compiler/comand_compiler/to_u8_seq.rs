
pub trait ToU8Seq<Iter> 
where Iter: Iterator<Item = u8>
{
    fn to_u8_seq(&self) -> Iter;
}