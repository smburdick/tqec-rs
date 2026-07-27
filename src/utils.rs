

pub fn concat_bits<I>(bits: I) -> u64 where I: IntoIterator<Item = u64> { // TODO: add bit_length parameter
  let mut result = 0u64;
  for (shift, bit) in bits.into_iter().enumerate() {
    result |= bit << shift;
  }
  result
}
