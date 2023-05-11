// stdlib imports
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
// local imports
use crate::min_max_heap::MinMaxHeap;

/// A vector, which allows deletion. Elements are never moved during their
/// lifetime.
pub struct IdVec< T, I = u32 > {
  free_ids: MinMaxHeap< u32 >,
  next_id: u32,
  data: Vec< Option< T > >,

  _phantom: PhantomData< I >
}

impl< T, I: From< u32 > + Into< u32 > > IdVec< T, I > {
  pub fn new( ) -> IdVec< T, I > {
    IdVec {
      free_ids: MinMaxHeap::new( )
    , next_id: 0
    , data: Vec::new( )
    , _phantom: PhantomData
    }
  }

  pub fn len( &self ) -> usize {
    ( self.next_id as usize ) - self.free_ids.len( )
  }

  pub fn push( &mut self, v: T ) -> I {
    if let Some( id ) = self.free_ids.pop_min( ) {
      self.data[ id as usize ] = Some( v );
      I::from( id )
    } else {
      let id = self.next_id;
      self.next_id += 1;
      self.data.push( Some( v ) );
      I::from( id )
    }
  }

  pub unsafe fn free_unchecked( &mut self, i: I ) -> T {
    let i32: u32 = i.into( );
    self.free_ids.push( i32 );

    let mut out = None;
    std::ptr::swap( &mut out, &mut self.data[ i32 as usize ] );
    out.unwrap_unchecked( )
  }

  pub fn free( &mut self, i: I ) -> T {
    let i32: u32 = i.into( );
    assert!( self.data[ i32 as usize ].is_some( ) );
    unsafe { self.free_unchecked( I::from( i32 ) ) }
  }
}

impl< T > Default for IdVec< T > {
  fn default() -> Self {
    IdVec::new( )
  }
}

impl< T, I: Into< u32 > > Index< I > for IdVec< T, I > {
  type Output = T;
  
  fn index(&self, i: I) -> &Self::Output {
    let i32: u32 = i.into( );
    self.data.index( i32 as usize ).as_ref( ).unwrap( )
  }
}

impl< T, I: Into< u32 > > IndexMut< I > for IdVec< T, I > {
  fn index_mut(&mut self, i: I) -> &mut Self::Output {
    let i32: u32 = i.into( );
    self.data.index_mut( i32 as usize ).as_mut( ).unwrap( )
  }
}
