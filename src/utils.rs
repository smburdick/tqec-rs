use std::collections::HashMap;

use quizx::{graph::{GraphLike, V}, vec_graph::Graph};

use crate::{pauli::Pauli, positioned::vertex_type_to_pauli};


pub fn concat_ints_as_bits<I, B>(
    ints: I,
    bit_lengths: B,
) -> usize
where
    I: IntoIterator<Item = usize>,
    B: IntoIterator<Item = usize>,
{
    let mut result = 0usize;
    let mut shift = 0usize;

    for (x, bits) in ints.into_iter().zip(bit_lengths) {
        result += x << shift;
        shift += bits;
    }

    result
}

//     return sum(x << shift for x, shift in zip(ints, chain([0], accumulate(bit_length))))

// FIXME: in the case where x = 10, should return zero, when it's currently not.
pub fn solve_linear_system(basis: &mut HashMap<usize, (usize, usize)>, x: usize, update_basis: bool) -> Result<Vec<usize>, &'static str> {
  // TODO: decide on the integer types (usize or u64)
  let mut mask: usize = 1 << basis.keys().len();
  let mut _x = x;
  while _x != 0 {
    let highest_bit: usize = usize_bit_len(_x) - 1;
    if !basis.contains_key(&highest_bit) {
      if update_basis {
        basis.insert(highest_bit, (_x, mask));
      }
      return Err("Is this really an error?"); 
    }
    let (pivot, pivot_mask) = basis.get(&highest_bit).expect("Missing Basis bro");
    _x ^= *pivot;
    mask ^= *pivot_mask;
  }
  Ok(int_to_bit_indices(mask).into_iter().rev().collect())
}

pub fn int_to_bit_indices(x: usize) -> Vec<usize> {
  let mut to_return: Vec<usize> = Vec::new();
  for i in 0..usize_bit_len(x) {
    if (x >> i & 1) != 0 {
      to_return.push(i)
    }
  }
  to_return
}

fn usize_bit_len(x: usize) -> usize {
  (usize::BITS - x.leading_zeros()) as usize
}

pub fn zx_to_pauli(g: &Graph, v: V) -> Pauli {
  let (vt, phase) = (g.vertex_type(v), g.phase(v));
  let res= vertex_type_to_pauli(vt, phase);
  if res.is_ok() {
    res.unwrap()
  } else {
    todo!("")
  }
}
