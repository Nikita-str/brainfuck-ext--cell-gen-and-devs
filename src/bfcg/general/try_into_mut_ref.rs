
pub trait TryIntoMutRef<T>{
    fn try_into_mut_ref(&mut self) -> Option<&mut T>;
}