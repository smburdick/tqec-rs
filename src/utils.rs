use std::collections::HashMap;



pub fn concat_ints_as_bits<I, B>(
    ints: I,
    bit_lengths: B,
) -> u32
where
    I: IntoIterator<Item = u32>,
    B: IntoIterator<Item = u32>,
{
    let mut result = 0u32;
    let mut shift = 0u32;

    for (x, bits) in ints.into_iter().zip(bit_lengths) {
        result |= x << shift;
        shift += bits;
    }

    result
}

pub fn solve_linear_system(basis: &mut HashMap<u32, (u32, u32)>, x: u32, update_basis: bool) -> Result<Vec<u32>, &'static str> {
  // TODO: decide on the integer types (u32 or u64)
  let mut mask: u32 = 1 << basis.keys().len();
  let mut _x = x;
  while _x != 0 {
    let highest_bit: u32 = u32_bit_len(_x) - 1;
    if !basis.contains_key(&highest_bit) {
      if update_basis {
        basis.insert(highest_bit, (x, mask));
      }
      return Ok(Vec::new());
    }
    if let Some((pivot, pivot_mask)) = basis.get(&highest_bit) {
      _x ^= *pivot;
      mask ^= *pivot_mask;
    } else {
      return Err("Basis is not solvable.");
    }
  }
  Ok(int_to_bit_indices(mask).into_iter().rev().collect())
}

pub fn int_to_bit_indices(x: u32) -> Vec<u32> {
  let mut to_return: Vec<u32> = Vec::new();
  for i in 1..u32_bit_len(x) {
    if (x >> i & 1) != 0 {
      to_return.push(i)
    }
  }
  to_return
}

fn u32_bit_len(x: u32) -> u32 {
  u32::BITS - x.leading_zeros() 
}
