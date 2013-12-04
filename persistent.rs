// (language: Rust master branch)

#[link(name="persistent")];

/*! Persistent data structures

This is a collection of persistent data structures
modeled after purely functional programming languages
( https://en.wikipedia.org/wiki/Persistent_data_structure ).

Currently, only an inductive linked list is implemented.

*/

use persistent::list::List;

pub mod persistent {
pub mod list {

// Is reference-counting the best choice for the shared immutable data?
use std::rc::Rc;

/// Persistent cons/nil list.
/// O(1) access to the head of the list.
/// Modified versions of the list with shared tails can be created efficiently.
/// List members cannot be moved from,
/// because a List might be sharing data with other Lists.

// Rc's Eq/Ord compares the contained data, not the pointer.
#[deriving(Clone, DeepClone, Eq, Ord, TotalEq, TotalOrd)]
pub struct List<T> {
  priv node : Rc<Node<T>>
}

#[deriving(Clone, DeepClone, Eq, Ord, TotalEq, TotalOrd)]
pub enum Node<T> {
  Nil,
  Cons(T, List<T>)
}

impl<'self, T> Iterator<&'self T> for &'self List<T> {
  fn next(&mut self) -> Option<&'self T> {
    match *self.node.borrow() {
      Nil => None,
      Cons(ref x, ref xs) => {
        *self = xs;
        Some(x)
      }
    }
  }
}

impl<T> List<T> {
  /// Lists are iterable.
  pub fn iter<'t>(&'t self) -> &'t List<T> {
    self
  }
  /// Use this to pattern match on Nil vs Cons.
  pub fn node<'t>(&'t self) -> &'t Node<T> {
    self.node.borrow()
  }
}

// Ought Freeze really be required for members of persistent lists?
// Generally, yes, because there's shared data; but what if you want
// a list of Cells or RefCells that deliberately have shared mutable identity?
// Freeze is forced by Rc.
impl<T: Freeze> List<T> {
  /// Create an empty list
  pub fn nil() -> List<T> {
    List::new(Nil)
  }
  /// Create a list from the head and the rest of the list
  pub fn cons(x:T, xs:List<T>) -> List<T> {
    List::new(Cons(x, xs))
  }
  /// Create a list from a node (you probably won't need this function).
  pub fn new(node: Node<T>) -> List<T> {
    List{node: Rc::new(node)}
  }
}
impl<T: Clone+Freeze> List<T> {
  fn reverse_impl(&self, acc : List<T>) -> List<T> {
    match *self.node.borrow() {
      Nil => acc,
      Cons(ref x, ref xs) => xs.reverse_impl(List::cons(x.clone(), acc))
    }
  }
  /// Create a copy of this list in reverse order.
  /// It cannot move the members because it can't know whether it has
  /// the only reference to them.  (Rc doesn't even provide a runtime
  /// feature to find out whether you have the last reference.)
  pub fn reverse(&self) -> List<T> {
    self.reverse_impl(List::nil())
  }
}

impl<T> Container for List<T> {
  fn len(&self) -> uint {
    let mut result = 0;
    for _ in self.iter() { result += 1; }
    result
  }
  fn is_empty(&self) -> bool {
    match *self.node.borrow() {
      Nil => true,
      Cons(_, _) => false
    }
  }
}

impl<T: Freeze> Default for List<T> {
  fn default() -> List<T> {
    List::nil()
  }
}

/* Does this even make sense for an immutable container?
impl<A> Extendable<A> for List<A> {
  fn extend<T: Iterator<A>>(&mut self, iter: &mut T) {
  }
}
*/

impl<A: Freeze> FromIterator<A> for List<A> {
  // Is it possible to write this function without a
  // stack (implicit via recursion, or explicit),
  // and without 'unsafe' code?
  fn from_iterator<T: Iterator<A>>(iter: &mut T) -> List<A> {
    match iter.next() {
      None => List::nil(),
      Some(a) => List::cons(a, FromIterator::from_iterator(iter))
    }
  }
}

#[cfg(test)]
mod test {
use super::List;
//use std::cell::RefCell;
#[test]
fn test() {
  let p0 = List::nil();
  let p1 : List<int> = List::cons(1, p0.clone());
  let p2a = List::cons(2, p1.clone());
  let p2b = List::cons(2, p1.clone());
  let p2c = List::cons(3, p1.clone());
  assert!(p0 == p0);
  assert!(p1 == p1);
  assert!(p0 != p1);
  assert!(p2a == p2b);
  assert!(p2a < p2c);
  assert!(p1 < p2c);
  assert!(p0 == p0.reverse());
  assert!(p1 == p1.reverse());
  assert!(p2a > p2a.reverse());
  assert!(p2a == p2a.reverse().reverse());
  let mut sum = 0;
  for i in p2c.iter() {
    sum += *i;
  }
  assert!(sum == 4);
  let seq = ~[1,3,2]; //why did I have to allocate here in order to get .move_iter()?
  let mut digits = 0;
  let seql : List<int> = seq.move_iter().collect();
  for i in seql.iter() {
    digits = digits * 10 + *i;
  }
  assert!(digits == 132);
  assert!(p0 == Default::default());
  // doesn't meet Freeze requirement:
  //let sdf : List<RefCell<int>> = List::nil();
}
}

}
}



// Trying out printing some stuff
fn main() {
  let p0 = List::nil();
  let p1 : List<int> = List::cons(1, p0);
  let p2 = List::cons(2, p1.clone());
  let p2a = List::cons(2, p1.clone());
  let p2b = List::cons(3, p1.clone());
  println(format!("Successyays:\n{}, {}", p2 == p2a, p2 == p2b));
  for i in p2b.iter() {
    println(format!("{}", *i))
  }
  // Is there a way not to use the temporary var name here?
  // "for i in p2b.reverse().iter()" didn't work (I guess the
  // lifetime of p2b.reverse() there didn't include the body of the
  // for loop).
  let x = p2b.reverse();
  for i in x.iter() {
    println(format!("{}", *i))
  }
}

