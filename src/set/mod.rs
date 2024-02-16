mod borrowed;
mod owned;

pub mod ones;

pub use borrowed::Borrowed as Slice;
pub use owned::Owned as Set;
