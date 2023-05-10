// stdlib imports
use std::cmp::{Ordering, Reverse};
use std::mem::{MaybeUninit, ManuallyDrop};
use std::alloc::{Global, Allocator};
// local imports
use crate::alloc::AllocatorL1;


// # Definition: Min-Max Heap
// At even levels, store the element less-than-or-equal to all its descendants.
// At odd levels, store the element greater-than-or-equal to all its
// descendants.
//
// The root (at index 1) resides at level 0, which is even.


/// 
#[derive(Debug)]
pub struct MinMaxHeap< T: Ord, A: Allocator = Global >( Vec< MaybeUninit< T >, AllocatorL1< A > > );

impl< T: Ord > MinMaxHeap< T, Global > {
  #[must_use]
  pub fn new( ) -> MinMaxHeap< T > {
    let mut v = Vec::new_in( AllocatorL1::new( Global ) );
    v.push( MaybeUninit::uninit( ) );
    MinMaxHeap( v )
  }

  #[must_use]
  pub fn with_capacity( capacity : usize ) -> MinMaxHeap< T > {
    let mut v = Vec::with_capacity_in( capacity + 1, AllocatorL1::new( Global ) );
    v.push( MaybeUninit::uninit( ) );
    MinMaxHeap( v )
  }
}

impl< T: Ord, A: Allocator > MinMaxHeap< T, A > {
  pub fn len( &self ) -> usize {
    self.0.len( ) - 1
  }

  pub fn push( &mut self, val : T ) {
    let idx = self.0.len( );

    if idx == 1 {
      self.0.push( MaybeUninit::new( val ) );
    } else {
      let p_idx = parent_idx( idx );
      let ptr: *mut MaybeUninit< T > = self.0.as_mut_ptr( );

      // assert: parent_idx >= 0  (because idx > 0)
      let p_val: ManuallyDrop< T > = unsafe { read_init( ptr.add( p_idx ) ) };
  
      if is_at_even_level( idx ) {
        // element `idx` is on a min level.
        // so, its parent is on a max level.
  
        if *p_val < val {
          // violation: parent is not greater-than-or-equal
          unsafe {
            self.0.push( to_init( p_val ) );
            // note that `p_val` is the vector twice now, so we have to overwrite it
            // in `push_up_max`

            // bubble up `val` to the max
            // note that `self.0.push` may have changed the `ptr`
            push_up_max( self.0.as_mut_ptr( ), p_idx, val );
          }
        } else {
          unsafe {
            self.0.reserve( 1 );
            push_up_min( self.0.as_mut_ptr( ), idx, val );
            self.0.set_len( idx + 1 );
          }
        }
      } else {
        // element `idx` is on a max level.
        // so, its parent is on a min level.
  
        if *p_val > val {
          // violation: parent is not less-than-or-equal
          unsafe {
            self.0.push( to_init( p_val ) );
            // note that `p_val` is the vector twice now, so we have to overwrite it
            // in `push_up_max`

            // note that `self.0.push` may have changed the `ptr`
            push_up_min( self.0.as_mut_ptr( ), p_idx, val );
          }
        } else {
          unsafe {
            self.0.reserve( 1 );
            push_up_max( self.0.as_mut_ptr( ), idx, val );
            self.0.set_len( idx + 1 );
          }
        }
      }
    }
  }

  pub fn peek_min( &self ) -> Option< &T > {
    if self.0.len( ) == 1 {
      None
    } else {
      Some( unsafe { self.0.get_unchecked( 1 ).assume_init_ref( ) } )
    }
  }

  pub fn peek_max( &self ) -> Option< &T > {
    if self.0.len( ) > 3 {
      let v1 = unsafe { self.0.get_unchecked( 2 ).assume_init_ref( ) };
      let v2 = unsafe { self.0.get_unchecked( 3 ).assume_init_ref( ) };

      if v1 >= v2 {
        Some( v1 )
      } else {
        Some( v2 )
      }
    } else if self.0.len( ) == 1 {
      None
    } else if self.0.len( ) == 2 {
      Some( unsafe { self.0.get_unchecked( 1 ).assume_init_ref( ) } )
    } else { // self.0.len( ) == 3
      Some( unsafe { self.0.get_unchecked( 2 ).assume_init_ref( ) } )
    }
  }

  pub fn pop_min( &mut self ) -> Option< T > {
    let vec_len = self.0.len( );

    match vec_len.cmp( &2 ) {
      Ordering::Greater => { // vec_len > 2
        unsafe {
          let ptr = self.0.as_mut_ptr( );
      
          let final_idx = vec_len - 1;
    
          let min_val = std::ptr::read( ptr.add( 1 ) ).assume_init( );
          let final_val = std::ptr::read( ptr.add( final_idx ) ).assume_init( );
    
          let new_len = final_idx;
          self.0.set_len( new_len );
    
          // now insert `final_val` at index 1, but also push it down,
          // such that the heap is valid
          push_down_min( ptr, new_len, 1, final_val );
    
          Some( min_val )
        }
      },
      Ordering::Less => { // vec_len < 2
        None
      },
      Ordering::Equal => { // vec_len == 2
        // the heap contains only the minimum element. simply pop it
        unsafe {
          let new_len = vec_len - 1;
          self.0.set_len( new_len );
          Some( std::ptr::read( self.0.as_ptr( ).add( new_len ) ).assume_init( ) )
        }
      }
    }
  }

  pub fn pop_max( &mut self ) -> Option< T > {
    let vec_len = self.0.len( );

    if vec_len > 3 {
      let ptr = self.0.as_mut_ptr( );

      unsafe {
        let (idx, max_val) =
          {
            let m_val1 = read_init( ptr.add( 2 ) );
            let m_val2 = read_init( ptr.add( 3 ) );

            if m_val1 >= m_val2 {
              (2, ManuallyDrop::into_inner( m_val1 ) )
            } else {
              (3, ManuallyDrop::into_inner( m_val2 ) )
            }
          };
  
        let final_idx = vec_len - 1;
  
        let final_val = std::ptr::read( ptr.add( final_idx ) );
  
        let new_len = final_idx;
        self.0.set_len( new_len );
  
        // now insert `final_val` at `idx`, but also push it down,
        // such that the heap is valid
        push_down_max( ptr, new_len, idx, final_val.assume_init( ) );

        Some( max_val )
      }
    } else if vec_len >= 2 { // 2 or 3
      // pop
      unsafe {
        let new_len = vec_len - 1;
        self.0.set_len( new_len );
        Some( std::ptr::read( self.0.as_ptr( ).add( new_len ) ).assume_init( ) )
      }
    } else { // self.len( ) == 1, (note that length 0 is impossible)
      None
    }
  }

  pub fn clear( &mut self ) {
    // this is like calling `self.0.truncate( 1 )`
    // but a `MaybeUninit` cannot be dropped. so we ensure that
    unsafe {
      let remaining_len = self.0.len( ) - 1;
      let s = std::ptr::slice_from_raw_parts_mut( self.0.as_mut_ptr().add( 1 ), remaining_len );
      self.0.set_len( 1 );
      // On `MaybeUninit` deconstructors are never called. On its contained element `T` they are.
      // (note that transmutation is safe on transparent types - i.e., MaybeUninit)
      std::ptr::drop_in_place( std::mem::transmute::< _, *mut [T] >( s ) );
    }
  }
}

impl< T: Ord, A: Allocator > Drop for MinMaxHeap< T, A > {
  fn drop( &mut self ) {
    // Our elements are `MaybeUninit`, which means `drop` won't be called. We
    // manually call it here.
    self.clear( );
  }
}

// the index of the /first child/. the second child is immediately next to it.
// that is, child2_idx( idx ) = child_idx( idx ) + 1
fn child_idx( idx : usize ) -> usize {
  idx * 2
}

fn parent_idx( idx : usize ) -> usize {
  idx / 2
}

fn grandparent_idx( idx : usize ) -> usize {
  idx / 4
}

/// Returns `true` if the index resides on an even level.
/// (Root is at level 0)
fn is_at_even_level( idx : usize ) -> bool {
  ( idx.leading_zeros( ) & 0x1 ) != 0
}

unsafe fn push_up_max< A: Ord >( ptr: *mut MaybeUninit< A >, idx: usize, val: A ) {
  // note that transmutation is safe on transparent types - i.e., Reverse
  push_up_min::< Reverse< A > >( std::mem::transmute( ptr ), idx, Reverse( val ) )
}

// precondition: idx is in bounds of the `ptr` array
// precondition: idx is on an even level
// `val` is the value we're pushing up
//
// when calling this, the heap is in an invalid state. hence, unsafe
// `val` still needs to be moved in
unsafe fn push_up_min< A: Ord >( ptr: *mut MaybeUninit< A >, mut idx: usize, val: A ) {
  // println!( "push up min" );

  while idx > 3 { // while has grandparent
    // invariant: element `idx` is on an even (min) level

    let gp_idx = grandparent_idx( idx );
    let gp_val: ManuallyDrop< A > = read_init( ptr.add( gp_idx ) ); // gp_val = ptr[ gp_idx ]

    if *gp_val > val {
      // violation: not a min-max heap
      std::ptr::write( ptr.add( idx ), to_init( gp_val ) );

      idx = gp_idx;
    } else {
      break; // its a heap now
    }
  }

  std::ptr::write( ptr.add( idx ), MaybeUninit::new( val ) );
}

unsafe fn push_down_max< A: Ord >( ptr: *mut MaybeUninit< A >, len: usize, idx : usize, val: A ) {
  // note that transmutation is safe on transparent types - i.e., Reverse
  push_down_min::< Reverse< A > >( std::mem::transmute( ptr ), len, idx, Reverse( val ) )
}

// precondition: idx < len
// `val` is intended to move in at index 1
unsafe fn push_down_min< A: Ord >( ptr: *mut MaybeUninit< A >, len: usize, mut idx : usize, mut val: A ) {
  loop {
    // invariant: is_at_even_level( idx )

    if let Some( (c_idx, c_val, is_grandchild) ) = min_level2_idx( ptr, len, idx ) {
      if *c_val < val {
        // violation: the minimum should be at `idx`
        std::ptr::write( ptr.add( idx ), to_init( c_val ) );

        if is_grandchild {
          let cp_idx = parent_idx( c_idx );
          let cp_val = read_init( ptr.add( cp_idx ) );

          if *cp_val < val {
            // violation: c's parent is a max node, and should be greater

            std::ptr::write( ptr.add( cp_idx ), MaybeUninit::new( val ) );
            val = ManuallyDrop::into_inner( cp_val );
          }
          idx = c_idx;
        } else {
          std::ptr::write( ptr.add( c_idx ), MaybeUninit::new( val ) );
          break; // it has no further children
        }
      } else {
        std::ptr::write( ptr.add( idx ), MaybeUninit::new( val ) );
        break; // it's a heap now
      }
    } else {
      std::ptr::write( ptr.add( idx ), MaybeUninit::new( val ) );
      break; // it's a heap now
    }
  }

  // println!( "push down min, return" );
}

fn min_child_idx< A: Ord >( ptr: *mut MaybeUninit< A >, len: usize, idx : usize ) -> Option< (usize, ManuallyDrop< A >) > {
  let c1_idx = child_idx( idx );
  let c2_idx = c1_idx + 1;

  if c2_idx < len { // it has 2 children
    let c1_val: ManuallyDrop< A > = unsafe { read_init( ptr.add( c1_idx ) ) };
    let c2_val: ManuallyDrop< A > = unsafe { read_init( ptr.add( c2_idx ) ) };

    if c2_val < c1_val {
      Some( (c2_idx, c2_val) )
    } else {
      Some( (c1_idx, c1_val) )
    }
  } else if c1_idx < len { // it has only 1 child
    let c1_val: ManuallyDrop< A > = unsafe { read_init( ptr.add( c1_idx ) ) };
    Some( (c1_idx, c1_val) )
  } else {
    None
  }
}

/// Returns the index of the minimal element within 2 levels under the given
/// element. That is, the depth directly below `idx`, or one deeper.
/// 
/// The boolean indicates whether the index is a grandchild.
fn min_level2_idx< A: Ord >( ptr: *mut MaybeUninit< A >, len: usize, idx : usize ) -> Option< (usize, ManuallyDrop< A >, bool) > {
  let c1_idx = child_idx( idx );
  let c2_idx = c1_idx + 1;

  if let Some( (gc1_idx, gc1_val) ) = min_child_idx( ptr, len, c1_idx ) {
    if let Some( (gc2_idx, gc2_val) ) = min_child_idx( ptr, len, c2_idx ) {

      if gc2_val < gc1_val {
        Some( ( gc2_idx, gc2_val, true ) )
      } else {
        Some( ( gc1_idx, gc1_val, true ) )
      }
    } else { // c2 has no children
      let c2_val: ManuallyDrop< A > = unsafe { read_init( ptr.add( c2_idx ) ) };

      if gc1_val < c2_val {
        Some( ( gc1_idx, gc1_val, true ) )
      } else {
        Some( ( c2_idx, c2_val, false ) )
      }
    }
  } else if c2_idx < len { // child 1 has no children. so neither does child 2
    let c1_val: ManuallyDrop< A > = unsafe { read_init( ptr.add( c1_idx ) ) };
    let c2_val: ManuallyDrop< A > = unsafe { read_init( ptr.add( c2_idx ) ) };

    if c2_val < c1_val {
      Some( ( c2_idx, c2_val, false ) )
    } else {
      Some( ( c1_idx, c1_val, false ) )
    }
  } else if c1_idx < len { // there's only a child 1
    Some( ( c1_idx, unsafe { read_init( ptr.add( c1_idx ) ) }, false ) )
  } else {
    None
  }
}


impl< T: Ord > Default for MinMaxHeap< T, Global > {
  fn default( ) -> MinMaxHeap< T > {
    MinMaxHeap::new( )
  }
}

unsafe fn read_init< T >( ptr: *const MaybeUninit< T > ) -> ManuallyDrop< T > {
  ManuallyDrop::new( std::ptr::read( ptr ).assume_init( ) )
}

fn to_init< T >( x: ManuallyDrop< T > ) -> MaybeUninit< T > {
  MaybeUninit::new( ManuallyDrop::into_inner( x ) )
}
