use matrix::*;
use complex::Complex;
use std::ops::{Index, IndexMut};
use num::traits::{Zero, One};
use std::cmp::PartialEq;
use std::fmt;
use std::fmt::{Debug};

#[derive(Debug)]
pub struct Gate<T=Complex>
    where T: PartialEq + Debug + Clone + Zero + One + Copy {
        pub size: usize,
        pub matrix: Matrix<T>
}

impl <T> Gate<T>
    where T: PartialEq + Debug + Clone + Zero + One + Copy{
    pub fn new(size: usize, matrix: Matrix<T>) -> Gate<T> {
        Gate {
            size: size,
            matrix: matrix
        }
    }

    pub fn from_slice(array: &[T]) ->Gate<T> {
        Gate {
            size: f64::log2(array.len() as f64) as usize,
            matrix: Matrix::<T>::new_from_row_slice(array)
        }
    }
}

// TODO make macros!
// We want to emulate 2D Array indices, so we use a tuple like (row, col)
impl Index<(usize,usize)> for Gate<Complex> {
    type Output = Complex;
    fn index<'a>(&'a self, index: (usize,usize)) -> &'a Complex {
        match index {
            (x,y) => &self.matrix[(x, y)]
        }
    }
}

impl IndexMut<(usize,usize)> for Gate<Complex> {
    fn index_mut<'a>(&'a mut self, index: (usize,usize)) -> &'a mut Complex {
        match index {
            (x,y) => &mut self.matrix[(x, y)]
        }
    }
}

// We want to emulate 2D Array indices, so we use a tuple like (row, col)
impl Index<(usize,usize)> for Gate<f64> {
    type Output = f64;
    fn index<'a>(&'a self, index: (usize,usize)) -> &'a f64 {
        match index {
            (x,y) => &self.matrix[(x, y)]
        }
    }
}

impl IndexMut<(usize,usize)> for Gate<f64> {
    fn index_mut<'a>(&'a mut self, index: (usize,usize)) -> &'a mut f64 {
        match index {
            (x,y) => &mut self.matrix[(x, y)]
        }
    }
}

impl fmt::Display for Gate<Complex> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Gate({}): {}", self.size, self.matrix)
    }
}


impl fmt::Display for Gate<f64> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Gate({}): {}", self.size, self.matrix)
    }
}
