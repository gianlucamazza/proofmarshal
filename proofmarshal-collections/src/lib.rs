#![feature(never_type)]
#![feature(unwrap_infallible)]

#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use proofmarshal_core::*;

pub mod merklesum;
pub mod tree;
//pub mod mmr;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
