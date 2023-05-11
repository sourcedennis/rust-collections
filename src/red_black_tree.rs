// stdlib imports
use std::fmt;
use std::fmt::{Display, Formatter};
// local imports
use crate::id_vec::IdVec;

use std::collections::HashSet; // temp


pub struct RedBlackTree< T: Ord > {
  nodes: IdVec< NodeRB< T > >
}

struct NodeRB< T: Ord > {
  child: [u32; 2],
  parent: u32,
  color: Color,
  val: T
}

enum Color { Red, Black }

impl Color {
  fn is_red( &self ) -> bool {
    if let Color::Red = self {
      true
    } else {
      false
    }
  }
}

impl< T: Ord > RedBlackTree< T > {
  pub fn new( ) -> Self {
    RedBlackTree { nodes: IdVec::new( ) }
  }

  pub fn insert( &mut self, val: T ) {
    let p_idx = self.find_parent( &val );

    let node =
      NodeRB {
        child: [u32::MAX; 2]
      , parent: p_idx
      , color: Color::Red
      , val
      };
    let idx = self.nodes.push( node );

    if p_idx == u32::MAX {
      // we have no nodes. so our new node is the root
      assert!( idx == 0 );
    } else {
      let dir = ( self.nodes[ idx ].val >= self.nodes[ p_idx ].val ) as usize;
      assert!( self.nodes[ p_idx ].child[ dir ] == u32::MAX );
      self.nodes[ p_idx ].child[ dir ] = idx;
      self.fix_violation( idx );
    }
  }

  pub fn peek( &self, val: &T ) -> Option< &T > {
    let idx = self.right_idx( &val )?;

    if self.nodes[ idx ].val == *val {
      Some( &self.nodes[ idx ].val )
    } else {
      None
    }
  }

  pub fn peek_geq( &self, val: &T ) -> Option< &T > {
    let idx = self.right_idx( &val )?;
    Some( &self.nodes[ idx ].val )
  }

  pub fn len( &self ) -> usize {
    self.nodes.len( )
  }

  pub fn check_invariants( &self ) -> bool {
    if self.len( ) == 0 {
      true
    } else {
      let mut visited: HashSet< u32 > = HashSet::default( );
      self.check_invariants_rec( u32::MAX, 0, &mut visited ).is_some( )
        && visited.len( ) == self.len( )
    }
  }

  /// Returns the black height, if its the same for all paths.
  fn check_invariants_rec( &self, p_idx: u32, idx: u32, visited: &mut HashSet< u32 > ) -> Option< usize > {
    if idx == u32::MAX {
      Some( 0 )
    } else if visited.insert( idx ) {
      if self.nodes[ idx ].parent != p_idx {
        None // wrong parent
      } else if self.nodes[ idx ].color.is_red( ) && p_idx != u32::MAX && self.nodes[ p_idx ].color.is_red( ) {
        None // a red node has a red parent
      } else {
        let c0 = self.nodes[ idx ].child[ 0 ];
        let c1 = self.nodes[ idx ].child[ 1 ];

        if c0 != u32::MAX && self.nodes[ c0 ].val > self.nodes[ idx ].val {
          None
        } else if c1 != u32::MAX && self.nodes[ c1 ].val < self.nodes[ idx ].val {
          None
        } else {
          let black_height1 = self.check_invariants_rec( idx, c0, visited )?;
          let black_height2 = self.check_invariants_rec( idx, self.nodes[ idx ].child[ 1 ], visited )?;
  
          if black_height1 == black_height2 {
            let is_black = !self.nodes[ idx ].color.is_red( );
            Some( black_height1 + ( is_black as usize ) )
          } else {
            None
          }
        }
      }
    } else {
      None // error: visited same idx twice
    }
  }

  #[inline]
  fn fix_violation( &mut self, mut idx: u32 ) {
    let mut p_idx = self.nodes[ idx ].parent;

    while p_idx != u32::MAX && self.nodes[ p_idx ].color.is_red( ) {
      // invariant: nodes[ idx ].color.is_red( )
      let mut dir = ( self.nodes[ p_idx ].val <= self.nodes[ idx ].val ) as usize;

      let g_idx = self.nodes[ p_idx ].parent;

      if g_idx == u32::MAX { // our parent is the root
        assert!( p_idx == 0 );
        self.nodes[ p_idx ].color = Color::Black;
      } else {
        let p_dir = ( self.nodes[ g_idx ].val <= self.nodes[ p_idx ].val ) as usize;
        let u_dir = 1 - p_dir;
        let u_idx = self.nodes[ g_idx ].child[ u_dir ];

        if u_idx != u32::MAX && self.nodes[ u_idx ].color.is_red( ) {
          // both our parent and uncle are red. (grandparent is black)
          self.nodes[ p_idx ].color = Color::Black;
          self.nodes[ u_idx ].color = Color::Black;
          self.nodes[ g_idx ].color = Color::Red;
          
          // now our grandparent may violate our invariant. iterate.
          idx = g_idx;
          p_idx = self.nodes[ g_idx ].parent;
        } else { // uncle is black (or does not exist)
          // parent is red, uncle is black. (grandparent is black)

          if dir != p_dir {
            self.rotate( p_idx, 1 - dir );
            dir = 1 - dir;
          }

          self.rotate( g_idx, 1 - dir );
          // TODO: We could include recoloring into rotation. Now it happens twice..
          self.nodes[ g_idx ].color = Color::Black; // contains our "previous parent" now
          self.nodes[ p_idx ].color = Color::Red; // contains our "previous grandparent" now
          // note `idx` remains the same, but now has `g_idx` as parent
          p_idx = g_idx;
        }
      }
    } // otherwise: either, we are the root, or our parent is black.
  }

  /// Find a node that can become the parent of `v`, and the corresponding
  /// grandparent. Returns: (grandparent idx, parent idx)
  #[inline]
  fn find_parent( &self, v: &T ) -> u32 {
    let mut p_idx = u32::MAX;

    if self.nodes.len( ) != 0 {
      let mut idx = 0;

      while idx != u32::MAX {
        p_idx = idx;

        let dir = ( self.nodes[ p_idx ].val <= *v ) as usize;
        idx = self.nodes[ p_idx ].child[ dir ];
      }
    }

    p_idx
  }

  // /// Returns the index of the element left of `v`. This is greatest element
  // /// less-than-or-equal to `v`.
  // fn left_idx( &self, v: &T ) -> Option< u32 > {
  //   let mut idx: Option< u32 > = None;
  //   let mut next_idx = 0;

  //   while next_idx != u32::MAX {
  //     if self.nodes[ next_idx ].val > *v {
  //       next_idx = self.nodes[ next_idx ].child[ 0 ];
  //     } else { // next_idx is a candidate
  //       idx = Some( next_idx );
  //       // nodes[ idx ].val <= v
  //       next_idx = self.nodes[ next_idx ].child[ 1 ];
  //     }
  //   }

  //   idx
  // }

  /// Returns the index of the element right of `v`. This is lowest element
  /// greater-than-or-equal to `v`.
  fn right_idx( &self, v: &T ) -> Option< u32 > {
    let mut idx: Option< u32 > = None;
    let mut next_idx = 0;

    while next_idx != u32::MAX {
      if self.nodes[ next_idx ].val < *v {
        next_idx = self.nodes[ next_idx ].child[ 1 ];
      } else { // next_idx is a candidate
        idx = Some( next_idx );
        // nodes[ idx ].val >= v
        next_idx = self.nodes[ next_idx ].child[ 0 ];
      }
    }

    idx
  }
  
  // dir = 0: rotate left; dir = 1: rotate right
  //
  // `idx` references the top. during a left rotate, it swaps with its right
  // child. crucially, they actually change place in memory. afterwards `idx`
  // contains that child.
  fn rotate( &mut self, idx: u32, dir: usize ) {
    let node1 = idx;
    let node2 = self.nodes[ idx ].child[ 1 - dir ];

    debug_assert!( node1 != u32::MAX );
    debug_assert!( node2 != u32::MAX );

    unsafe {
      std::ptr::swap( &mut self.nodes[ node1 ].val, &mut self.nodes[ node2 ].val );
      std::ptr::swap( &mut self.nodes[ node1 ].color, &mut self.nodes[ node2 ].color );
    }

    self.nodes[ node1 ].child[ 1 - dir ] = self.nodes[ node2 ].child[ 1 - dir ];
    self.nodes[ node2 ].child[ 1 - dir ] = self.nodes[ node2 ].child[ dir ];
    self.nodes[ node2 ].child[ dir ] = self.nodes[ node1 ].child[ dir ];
    self.nodes[ node1 ].child[ dir ] = node2 as u32;

    let c1 = self.nodes[ node2 ].child[ dir ];
    let c3 = self.nodes[ node1 ].child[ 1 - dir ];

    if c1 != u32::MAX {
      self.nodes[ c1 ].parent = node2;
    }
    if c3 != u32::MAX {
      self.nodes[ c3 ].parent = node1;
    }
    // note that "child 2" keeps the same parent. it just became the other child
  }
}

impl< T: Display + Ord > Display for RedBlackTree< T > {
  fn fmt( &self, f: &mut Formatter<'_> ) -> fmt::Result {
    write!( f, "[" )?;
    let mut has_printed = false;
    if self.len( ) > 0 {
      let mut stack: Vec< u32 > = Vec::new( );

      let mut idx = 0;

      while idx != u32::MAX {
        stack.push( idx );
        idx = self.nodes[ idx ].child[ 0 ];
      }

      while let Some( mut idx ) = stack.pop( ) {
        if has_printed {
          write!( f, ",{}", self.nodes[ idx ].val )?;
        } else {
          write!( f, "{}", self.nodes[ idx ].val )?;
          has_printed = true;
        }

        idx = self.nodes[ idx ].child[ 1 ];
        while idx != u32::MAX {
          stack.push( idx );
          idx = self.nodes[ idx ].child[ 0 ];
        }
      }
    }
    write!( f, "]" )?;

    Ok( () )
  }
}
