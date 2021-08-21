pub trait Camera {
    fn write(&mut self, data: &[u8]) -> u8;
}
