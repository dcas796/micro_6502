pub trait ReadWritable {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, byte: u8);
}
