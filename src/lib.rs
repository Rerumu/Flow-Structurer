// Resources:
// "Efficient Control Flow Restructuring for GPUs",
//     by Nico Reissmann, Thomas L. Falch, Benjamin A. Bjørnseth,
//        Helge Bahmann, Jan Christian Meyer, and Magnus Jahre.

pub mod branch;
pub mod pass;
pub mod repeat;
pub mod view;

pub use set;
