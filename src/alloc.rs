// stdlib imports
use std::alloc::{Allocator, Layout, AllocError};
// external library imports
use cache_size::l1_cache_line_size;


/// A custom allocator that ensures allocations are aligned with L1 cache lines.
#[derive(Clone)]
pub struct AllocatorL1< A: Allocator >( A, usize );

impl< A: Allocator > AllocatorL1< A > {
  pub fn new( allocator: A ) -> AllocatorL1< A > {
    let cache_line_size = l1_cache_line_size( ).unwrap_or( 128 );

    let cache_line_size =
      if cache_line_size.is_power_of_two( ) {
        cache_line_size
      } else {
        128
      };
    AllocatorL1( allocator, cache_line_size )
  }
}

unsafe impl< A: Allocator > Allocator for AllocatorL1< A > {
  fn allocate(&self, layout: Layout) -> Result<std::ptr::NonNull<[u8]>, AllocError> {
    // we perform the power-of-2 check in the constructor. so, safe here
    self.0.allocate( unsafe { fix_layout( layout, self.1 ) } )
  }

  unsafe fn deallocate( &self, ptr: std::ptr::NonNull<u8>, layout: Layout ) {
    self.0.deallocate( ptr, fix_layout( layout, self.1 ) )
  }
}

unsafe fn fix_layout( layout: Layout, align: usize ) -> Layout {
  Layout::from_size_align_unchecked( layout.size(), std::cmp::max( layout.align( ), align ) )
}
