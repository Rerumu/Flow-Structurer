// Resources:
// "Efficient Control Flow Restructuring for GPUs",
//     by Nico Reissmann, Thomas L. Falch, Benjamin A. Bjørnseth,
//        Helge Bahmann, Jan Christian Meyer, and Magnus Jahre.

pub mod branch;
pub mod nodes;
pub mod pass;
pub mod repeat;

pub use set;
