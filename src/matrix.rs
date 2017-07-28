//! Matrix library code (public for pedagogical reasons).

use std::ops::{Add, Mul,Index, IndexMut};
use num::traits::{Zero, One};
use std::cmp::PartialEq;
use std::fmt;
use std::fmt::Debug;

use complex::Complex;




// Nalgebra crate doesn't support Complex operations on Matrices yet:
// https://github.com/sebcrozet/nalgebra/issues/266
// So we need to make our own implementation

// TODO WARNING We are hitting this: https://github.com/rust-lang/rust/pull/42816 , stack overflow
// because Rust default stack size for threads is about ~350Kbs? So, we need to use the heap
// to create bigger Matrices. I changed all the original static memory Matrix implementation
// to comply with the dynamic memory allocation model


/// Represents a square matrix
// #[allow(missing_copy_implementations)]
pub struct Matrix<T=Complex> {
    size: usize,
    elements : Vec<T>,
}

impl <T> Matrix<T>
    where T: PartialEq + Debug + Clone + Zero + One + Mul<Output = T> + Copy {
    /// Construct a new zero-initialized matrix of given size.
    pub fn new(size: usize) -> Matrix<T> {
        Matrix {
            size: size,
            elements: vec![T::zero(); size*size],
        }
    }

    pub fn new_from_value(size: usize, value: T) -> Matrix<T> {
        Matrix {
            size: size,
            elements: vec![value; size*size],
        }
    }

    pub fn new_from_vector(size: usize, elements: Vec<T>) -> Matrix<T> {
        assert!(size * size == elements.len());

        let mut m = Matrix::<T>::new(size);

        for (i, elem) in elements.iter().enumerate() {
            m.set(i / size, i % size, elem);
        }

        m
    }

    pub fn new_from_row_slice(elements: &[T]) -> Matrix<T> {
        debug!("new_from_row_slice: elements.len={}", elements.len());
        let size = f64::sqrt(elements.len() as f64) as usize;
        let mut m = Matrix::<T>::new(size);
        for (i, elem) in elements.iter().enumerate() {
            m.set(i / size, i % size, elem);
        }
        m
    }

    pub fn as_slice(&self) -> &[T] {
        self.elements.as_slice()
    }

    /// Construct a new identity matrix of given size.
    pub fn identity(size: usize) -> Matrix<T> {
        let mut elements = vec![T::zero(); size*size];

        for i in 0..size {
            elements[i * size + i] = T::one();
        }

        Matrix {
            size: size,
            elements: elements,
        }
    }

    /// Embed another matrix into this one, overrising elements.
    ///
    /// Embed with top-left position at (i, j).
    ///
    /// # Panics
    ///
    /// We panic if this matrix isn't large enough.
    pub fn embed(&mut self, other: &Matrix<T>, i: usize, j: usize) {
        assert!(i + other.size <= self.size);
        assert!(j + other.size <= self.size);

        for x in 0..other.size {
            for y in 0..other.size {
                let value = other.get(x, y);
                self.set(i + x, i + y, value);
            }
        }
    }

    /// Permute the rows to generate a new matrix.
    ///
    /// Row _i_ goes to row _perutation[i]_.
    ///
    /// # Panics
    ///
    /// We panic if set(permutation) != {0, ..., self.size - 1}.
    pub fn permute_rows(&self, permutation: Vec<usize>) -> Matrix<T> {
        assert_eq!(self.size, permutation.len());
        assert!(Matrix::<T>::permutation_valid(&permutation));

        let mut m = Matrix::<T>::new(self.size);

        for (source_i, target_i) in permutation.iter().enumerate() {
            for j in 0..self.size {
                m.set(*target_i, j, self.get(source_i, j));
            }
        }

        m
    }

    /// Permute the columns to generate a new matrix.
    ///
    /// Column _i_ goes to column _perutation[i]_.
    ///
    /// # Panics
    ///
    /// We panic if set(permutation) != {0, ..., self.size - 1}.
    pub fn permute_columns(&self, permutation: Vec<usize>) -> Matrix<T> {
        assert_eq!(self.size, permutation.len());
        assert!(Matrix::<T>::permutation_valid(&permutation));

        let mut m = Matrix::<T>::new(self.size);

        for (source_j, target_j) in permutation.iter().enumerate() {
            for i in 0..self.size {
                m.set(i, *target_j, self.get(i, source_j));
            }
        }
        m
    }

    /// Tests whether the permutation is valid.
    fn permutation_valid(permutation: &Vec<usize>) -> bool {
        let mut sorted = permutation.clone();
        sorted.sort();
        for (i, val) in sorted.iter().enumerate() {
            if i != *val {
                return false;
            }
        }

        return true;
    }

    // kronecker product of two matrices
    pub fn kronecker(&self, matrix: &Matrix<T>) -> Matrix<T> {
        debug!("kronecker: self.size={} matrix.size={}", self.size, matrix.size);
        //assert_eq!(self.size, matrix.size);
        let mut res = Matrix::<T>::new(self.size * matrix.size);

        let mut offset = 0;
        for col1 in 0..self.size {
            for col2 in 0..matrix.size {
                for row1 in 0..self.size {
                    let coeff = self.get(row1,col1);
                    //debug!("kronecker: c1:{} c2:{} r1:{} {:?}", col1, col2, row1,coeff);
                    for row2 in 0..matrix.size {
                        res.elements[offset] = *coeff * *matrix.get(row2,col2);
                        //debug!("kronecker: c1:{} c2:{} r1:{} r2:{} {:?}", col1, col2, row1, row2, res.elements[offset]);
                        // No post-increment in Rust, :(
                        offset += 1;
                    }
                }
            }
        }
        debug!("kronecker: Resulting matrix size = {}", res.size);
        res
    }

    /// Size of the matrix.
    pub fn size(&self) -> usize {
        self.size
    }

    // TODO: Return a reference? Let's wait for the benchmarks
    /// Get the element at position `(i, j)`.
    ///pub fn get(&self, i: usize, j: usize) -> T {
    ///    self.elements[i * self.size + j].clone()
    ///}
    pub fn get(&self, i: usize, j: usize) -> &T {
        &self.elements[i * self.size + j]
    }

    /// Set the element in position `(i, j)` to `value`.
    pub fn set(&mut self, i: usize, j: usize, value: &T) {
        self.elements[i * self.size + j] = *value
    }

    /// Approximately equal test.
    pub fn approx_eq(&self, other: &Matrix<T>) -> bool {
        if self.size != other.size {
            return false;
        }

        for i in 0..self.size {
            for j in 0..self.size {
                if self.get(i, j) != other.get(i, j) {
                    return false;
                }
            }
        }
        true
    }

    /// Dot product between a Matrix and a Vector
    // TODO Implement as a method of Matrix, not a static one.
    pub fn dot(matrix: &Matrix<Complex>, vector: &Vec<f64>) -> Matrix<Complex> {
        assert_eq!(matrix.size(), vector.len());
        let mut v = Vec::<Complex>::with_capacity(vector.len());
        let mut offset = 0;
        let mut end = matrix.size();
        for _ in 0..matrix.size(){
            v.push(matrix.elements[offset..end].iter().zip(vector.iter()).fold(Complex::zero(), |acc, (a,b)| acc + *a * *b));
            offset = end;
            end += matrix.size();
        }

        Matrix::new_from_vector(f64::sqrt(v.len() as f64) as usize, v)
    }

    /// It shows the rust code needed to declare this matrix statically
    /// Pretty usefull for testing purposes
    pub fn show_rust_code(matrix: &Matrix<Complex>) {
        print!("*****************************************\n\n");
        println!("let m = Matrix::new_from_row_slice(&[");
        for i in 0..matrix.size() {
            let mut line_break_counter = 0;
            for j in 0..matrix.size() {
                let c = matrix.get(i, j);
                print!("Complex::new({}f64,{}f64), ", c.re(), c.im());
                line_break_counter += 1;
                if line_break_counter > 3 {
                    println!("");
                    line_break_counter = 0;
                }
            }
        }
        println!("]);\n\n");
        print!("*****************************************\n");
    }
}

///
/// Traits implementation
///
impl<T> fmt::Debug for Matrix<T>
    where T: PartialEq + Debug + Clone + Zero + One + Copy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Matrix(size={}, elements=[", self.size).ok();
        for i in 0..self.size {
            write!(f, "\n").ok();
            for j in 0..self.size {
                write!(f, "[{:?}]  ", self.get(i, j)).ok();
            }
        }
        write!(f, "]")
    }
}

impl<T> fmt::Display for Matrix<T>
    where T: PartialEq + Debug + Clone + Zero + One + Copy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Cuadratic matrices always
        write!(f, "\nMatrix {}x{}\n[", self.size, self.size).ok();
        for i in 0..self.size {
            write!(f, "[").ok();
            for j in 0..self.size {
                write!(f, "{:?}, ", self.get(i, j)).ok();
            }
            write!(f, "]\n").ok();
        }
        write!(f, "]\n").ok();
        Ok(())
    }
}

impl<T> PartialEq for Matrix<T>
    where T: PartialEq + Debug + Clone + Zero + One {
    fn eq(&self, other: &Matrix<T>) -> bool {
        assert_eq!(self.size, other.size);

        for i in 0..self.size {
            if self.elements[i] != other.elements[i] {
                return false;
            }
        }

        true
    }
}

///
/// Ops (Add, Mul, ...)
///
macro_rules! impl_ref_ops {
    ($base:tt, $type:ty, $Op:tt, $func:ident, ($sel:ident,$rhs:ident) $action:block) => {
        impl<'a> $Op<&'a $base<$type>> for &'a $base<$type>{
            type Output = $base<$type>;
            fn $func($sel, $rhs: &'a $base<$type>) -> Self::Output
                $action
        }
    }
}

macro_rules! impl_ref_ops_mixed {
    ($base:tt, ($type:ty, $input_type:ty), $Op:tt, $func:ident, ($sel:ident,$rhs:ident) $action:block) => {
        impl<'a> $Op<&'a $base<$input_type>> for &'a $base<$type>{
            type Output = $base<Complex>;
            fn $func($sel, $rhs: &'a $base<$input_type>) -> Self::Output
                $action
        }
    }
}


impl_ref_ops!(Matrix, Complex, Add, add, (self, rhs) {
    assert_eq!(self.size, rhs.size);
    let mut m = Matrix::<Complex>::new(self.size);
    for i in 0..self.size {
        for j in 0..self.size {
            m.set(i, j, &(*self.get(i, j) + *rhs.get(i, j)));
        }
    }
    m
});

impl_ref_ops!(Matrix, Complex, Mul, mul, (self, rhs){
    assert_eq!(self.size, rhs.size);
    let mut m = Matrix::<Complex>::new(self.size);
    for i in 0..self.size {
        for j in 0..self.size {
            let mut val = Complex::zero();
            for k in 0..self.size {
                val = val +  self.get(i, k) * rhs.get(k, j);
            }
            m.set(i, j, &val);
        }
    }
    m
});

impl_ref_ops!(Matrix, f64, Mul, mul, (self, rhs){
    assert_eq!(self.size, rhs.size);
    let mut m = Matrix::<f64>::new(self.size);
    for i in 0..self.size {
        for j in 0..self.size {
            let mut val = f64::zero();
            for k in 0..self.size {
                val = val +  *self.get(i, k) * *rhs.get(k, j);
            }
            m.set(i, j, &val);
        }
    }
    m
});

impl_ref_ops_mixed!(Matrix, (f64, Complex), Mul, mul, (self,rhs){
    assert_eq!(self.size, rhs.size);
    let mut m = Matrix::<Complex>::new(self.size);
    for i in 0..self.size {
        for j in 0..self.size {
            let mut val = Complex::zero();
            for k in 0..self.size {
                val = val +  *rhs.get(k, j) * *self.get(i, k)
            }
            m.set(i, j, &val);
        }
    }
    m
});

impl_ref_ops_mixed!(Matrix, (Complex, f64), Mul, mul, (self,rhs){
    assert_eq!(self.size, rhs.size);
    let mut m = Matrix::<Complex>::new(self.size);
    for i in 0..self.size {
        for j in 0..self.size {
            let mut val = Complex::zero();
            for k in 0..self.size {
                val = val + *self.get(i, k) * *rhs.get(k, j)
            }
            m.set(i, j, &val);
        }
    }
    m
});

// We want to emulate 2D Array indices, so we use a tuple-like input (row, col)
impl<T> Index<(usize,usize)> for Matrix<T> {
    type Output = T;
    fn index<'a>(&'a self, index: (usize,usize)) -> &'a T {
        match index {
            (x,y) => &self.elements[x + y * self.size]
        }
    }
}

impl<T> IndexMut<(usize,usize)> for Matrix<T> {
    fn index_mut<'a>(&'a mut self, index: (usize,usize)) -> &'a mut T {
        match index {
            (x,y) => &mut self.elements[x + y * self.size]
        }
    }
}


#[test]
fn matrix_test() {
    let m = m_real![1, 2; 3, 4];

    let mut v = Matrix::new_from_vector(2, vec![Complex::zero();4]);
    v[(0,0)] = c!(10f64, 0f64);
    v[(0,1)] = c!(20f64, 0f64);

    let mut expected = Matrix::new_from_vector(2, vec![Complex::zero();4]);
    expected[(0,0)] = c!(50f64, 0f64);
    expected[(0,1)] = c!(110f64, 0f64);

    let added = m_real![2, 4; 6, 8];

    let squared = m_real![7, 10; 15, 22];

    assert_eq!(added, &m + &m);
    assert_eq!(squared, &m * &m);
    assert_eq!(expected, &m * &v);
}

#[test]
fn embed_test() {
    let mut m = m_real![1, 2; 3, 4];
    let n = m_real![5];

    m.embed(&n, 1, 1);

    assert_eq!(m_real![1, 2; 3, 5], m);
}

#[test]
fn permutation_test() {
    let m = m_real![1, 2; 3, 4];

    assert_eq!(m_real![1, 2; 3, 4], m.permute_rows(vec![0, 1]));
    assert_eq!(m_real![3, 4; 1, 2], m.permute_rows(vec![1, 0]));

    assert_eq!(m_real![1, 2; 3, 4], m.permute_columns(vec![0, 1]));
    assert_eq!(m_real![2, 1; 4, 3], m.permute_columns(vec![1, 0]));
}

#[test]
#[should_panic(expected = "assertion failed")]
fn bad_row_permutation_test() {
    let m = m_real![1, 2; 3, 4];

    m.permute_rows(vec![0, 0]);
}

#[test]
#[should_panic(expected = "assertion failed")]
fn bad_column_permutation_test() {
    let m = m_real![1, 2; 3, 4];

    m.permute_columns(vec![0, 0]);
}

#[test]
fn kronecker_test() {
    let temp1 = Matrix::identity(2);
    let temp2 = Matrix::new_from_row_slice(&[
        Complex::one(), Complex::one(),
        Complex::one(), Complex::one()
    ]);

    let res = temp1.kronecker(&temp2);

    let expected = Matrix::new_from_row_slice(&[
        Complex::new(1f64,0f64), Complex::new(1f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64),
        Complex::new(1f64,0f64), Complex::new(1f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64),
        Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(1f64,0f64), Complex::new(1f64,0f64),
        Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(1f64,0f64), Complex::new(1f64,0f64),
    ]);
    assert_eq!(res, expected);
}

#[test]
fn dot_test() {
    let temp1 = Matrix::new_from_row_slice(&[
        Complex::new(1f64,1f64), Complex::new(2f64,2f64), Complex::new(3f64,3f64), Complex::new(4f64,4f64),
        Complex::new(4f64,4f64), Complex::new(3f64,3f64), Complex::new(2f64,2f64), Complex::new(1f64,1f64),
        Complex::new(1f64,4f64), Complex::new(2f64,3f64), Complex::new(3f64,2f64), Complex::new(4f64,1f64),
        Complex::new(4f64,1f64), Complex::new(3f64,2f64), Complex::new(2f64,3f64), Complex::new(1f64,4f64),
    ]);

    let temp2 = vec![1.1f64, 2.2f64, 3.3f64, 4.4f64];

    let expected = Matrix::new_from_row_slice(&[
        Complex::new(33f64,33f64), Complex::new(22f64,22f64), Complex::new(33f64,22f64), Complex::new(22f64,33f64),
    ]);

    let res = Matrix::<Complex>::dot(&temp1, &temp2);

    assert_eq!(res, expected);

}
