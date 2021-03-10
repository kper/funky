use crate::icfg::convert::Convert;
use insta::assert_snapshot;

macro_rules! ir {
    ($name:expr, $ir:expr) => {
        let mut convert = Convert::new();

        let res = convert.visit($ir).unwrap();

        assert_snapshot!($name, format!("{:#?}", res));
    };
}

#[test]
fn test_ir_const() {
    ir!("test_ir_const", "
         define test {
            %0 = 1
         };
    ");
}

#[test]
fn test_ir_double_const() {
    ir!("test_ir_const", "
         define test {
            %0 = 1
            %1 = 1
         };
    ");
}