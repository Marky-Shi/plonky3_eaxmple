use p3_air::{Air,AirBuilder,BaseAir};
use p3_field::{AbstractField,Field};
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrix;

pub struct MyAir{
    pub num_steps: usize,
    pub final_value:u32,
}

impl<F:Field> BaseAir<F> for MyAir{
    fn width(&self) -> usize{
        2 // NUM_FIBONACCI_COLS
    }    
}

impl<AB: AirBuilder> Air<AB> for MyAir{
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let next = main.row_slice(1);

        // Enforce starting values
        builder.when_first_row().assert_eq(local[0], AB::Expr::ZERO);
        builder.when_first_row().assert_eq(local[1], AB::Expr::ONE);

        // Enforce state transition constraints
        builder.when_transition().assert_eq(next[0], local[1]);
        builder.when_transition().assert_eq(next[1], local[0] + local[1]);

        // Constrain the final value
        let final_value = AB::Expr::from_canonical_u32(self.final_value);
        builder.when_last_row().assert_eq(local[1], final_value);
    }
}

pub fn generate_air_trace<F: Field>(num_steps: usize) -> RowMajorMatrix<F>{
    let mut values = Vec::with_capacity(num_steps * 2);

    let mut a = F::ZERO;
    let mut b = F::ONE;

    for _ in 0..num_steps {
        values.push(a);
        values.push(b);
        let c = a + b;
        a = b;
        b = c;
    }
    RowMajorMatrix::new(values, 2)
}
