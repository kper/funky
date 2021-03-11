use crate::icfg::convert::Convert;
use crate::icfg::graphviz::render_to;
use insta::assert_snapshot;
use std::io::Cursor;

macro_rules! ir {
    ($name:expr, $ir:expr) => {
        let mut convert = Convert::new();

        let res = convert.visit($ir).unwrap();

        assert_snapshot!($name, format!("{:#?}", res));

        //let mut dot = String::new();
        let mut dot = Cursor::new(Vec::new());
        render_to(&res, &mut dot);

        assert_snapshot!(
            format!("{}_dot", $name),
            std::str::from_utf8(dot.get_ref()).unwrap()
        );
    };
}

#[test]
fn test_ir_const() {
    ir!(
        "test_ir_const",
        "
         define test {
            %0 = 1
         };
    "
    );
}

#[test]
fn test_ir_double_const() {
    ir!(
        "test_ir_double_const",
        "
         define test {
            %0 = 1
            %1 = 1
         };
    "
    );
}

#[test]
fn test_ir_assignment() {
    ir!(
        "test_ir_assignment",
        "
         define test {
            %1 = 1
            %0 = %1
         };
    "
    );
}

#[test]
fn test_ir_double_assignment() {
    ir!(
        "test_ir_double_assignment",
        "
         define test {
            %1 = 1
            %0 = %1
            %0 = %1
         };
    "
    );
}

#[test]
fn test_ir_block() {
    ir!(
        "test_ir_block",
        "define test {
            BLOCK 0
            %0 = 1
            GOTO 1
            BLOCK 1
            %1 = 2
        };"
    );
}

#[test]
fn test_ir_killing() {
    ir!(
        "test_ir_killing",
        "define test {
            %0 = 1
            %0 = 2
        };"
    );
}

#[test]
fn test_ir_unop() {
    ir!(
        "test_ir_unop",
        "define test {
            %0 = 1
            %1 = op %0
            %1 = op %0   
        };"
    );
}