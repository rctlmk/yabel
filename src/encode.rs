/// Bencode trait.
pub trait Bencode {
    /// Performs the encoding.
    fn encode(self) -> Vec<u8>;
}

impl<B: Bencode, T: Iterator<Item = B>> Bencode for T {
    fn encode(self) -> Vec<u8> {
        self.flat_map(|i| i.encode().into_iter()).collect()
    }
}