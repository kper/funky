use crate::grammar::*;

#[test]
fn parse_name() {
    assert!(NameParser::new().parse("abc").is_ok());
    assert!(NameParser::new().parse("Abc").is_ok());
}

#[test]
fn parse_num() {
    assert!(NumParser::new().parse("123").is_ok());
    assert!(NumParser::new().parse("-123").is_ok());
    assert!(NumParser::new().parse("+123").is_ok());
    assert!(NumParser::new().parse("+123.123").is_ok());
}

#[test]
fn parse_reg() {
    // Cannot mix alpha and numeric
    assert!(RegParser::new().parse("%123").is_ok());
    assert!(RegParser::new().parse("%a").is_ok());
}

#[test]
fn parse_instruction() {
    assert!(InstructionParser::new().parse("%123 = %a").is_ok());
    assert!(InstructionParser::new().parse("%a = %123").is_ok());
    assert!(InstructionParser::new().parse("%a = 10").is_ok());
    assert!(InstructionParser::new().parse("BLOCK 0").is_ok());
    assert!(InstructionParser::new().parse("GOTO 0").is_ok());
    assert!(InstructionParser::new().parse("IF %4 THEN GOTO 2 ELSE GOTO 3").is_ok());
    assert!(InstructionParser::new().parse("BR TABLE GOTO 7 5 ELSE GOTO 3").is_ok());
    assert!(InstructionParser::new().parse("%a = op %b").is_ok());
    assert!(InstructionParser::new().parse("%a = %b op %c").is_ok());
    assert!(InstructionParser::new().parse("KILL %0").is_ok());
}

#[test]
fn parse_function() {

}