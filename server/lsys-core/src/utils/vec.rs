
pub trait VecStringJoin {
    fn string_join(&self,sep:&str)->String;
}
impl<T> VecStringJoin for Vec<T>
where T:ToString{
    fn string_join(&self,sep:&str)->String{
        let val=self.iter().map(|e|e.to_string()).collect::<Vec<String>>();
        val.join(sep)
    }
}
