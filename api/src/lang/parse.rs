use crate::{
    error::CompileError,
    lang::{
        ast::{
            Instr, LangValue, NullaryOp, Program, StackIdentifier, StackOp,
            ValueOp,
        },
        Compiler,
    },
};
use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::{char, digit1, multispace0},
    combinator::map_res,
    multi::{many0, many1},
    sequence::{delimited, preceded, terminated},
    IResult,
};

trait InstrParser<'a> = Fn(&'a str) -> IResult<&'a str, Instr>;

fn make_nullary_op_parser<'a>(
    op_name: &'static str,
    op: NullaryOp,
) -> impl InstrParser<'a> {
    // Return a function that parses the given string into the given operator
    move |input| {
        // tag_no_case returns a (str, str) tuple
        // first element is whats left to parse
        // second is what matched
        let (input, _) = tag_no_case(op_name)(input)?;
        Ok((input, Instr::NullaryOp(op)))
    }
}

fn make_value_op_parser<'a>(
    op_name: &'static str,
    op: ValueOp,
) -> impl InstrParser<'a> {
    // Return a function that parses the given string into the given operator
    move |input| {
        let (input, _) = tag_no_case(op_name)(input)?;
        let (input, val) = preceded(
            multispace0,
            map_res(digit1, |s: &str| s.parse::<LangValue>()),
        )(input)?;
        Ok((input, Instr::ValueOp(op, val)))
    }
}

fn make_stack_op_parser<'a>(
    op_name: &'static str,
    op: StackOp,
) -> impl InstrParser<'a> {
    // Return a function that parses the given string into the given operator
    move |input| {
        let (input, _) = tag_no_case(op_name)(input)?;
        let (input, val) = preceded(
            multispace0,
            map_res(digit1, |s: &str| s.parse::<StackIdentifier>()),
        )(input)?;
        Ok((input, Instr::StackOp(op, val)))
    }
}

fn parse_if(input: &str) -> IResult<&str, Instr> {
    let (input, _) = tag_no_case("If")(input)?;
    let (input, res) = parse_body(input)?;
    Ok((input, Instr::If(res)))
}

fn parse_while(input: &str) -> IResult<&str, Instr> {
    let (input, _) = tag_no_case("While")(input)?;
    let (input, res) = parse_body(input)?;
    Ok((input, Instr::While(res)))
}

fn try_each(input: &str) -> IResult<&str, Instr> {
    let (input, res) = preceded(
        multispace0,
        alt((
            make_nullary_op_parser("Read", NullaryOp::Read),
            make_nullary_op_parser("Write", NullaryOp::Write),
            make_value_op_parser("Set", ValueOp::Set),
            make_value_op_parser("Add", ValueOp::Add),
            make_value_op_parser("Sub", ValueOp::Sub),
            make_value_op_parser("Mul", ValueOp::Mul),
            make_stack_op_parser("Push", StackOp::Push),
            make_stack_op_parser("Pop", StackOp::Pop),
            parse_if,
            parse_while,
        )),
    )(input)?;
    Ok((input, res))
}

// Parse the body of an if or while statement
//
// something like (\s*{<BODY>\s*})
fn parse_body(input: &str) -> IResult<&str, Vec<Instr>> {
    // multispace0 matches 0 or more whitespace chars (including new lines)
    let (input, res) = delimited(
        preceded(multispace0, char('{')),
        many0(try_each), // many0 will match 0 more, so the body could be empty
        preceded(multispace0, char('}')),
    )(input)?;
    Ok((input, res))
}

fn parse_gdlk(input: &str) -> IResult<&str, Vec<Instr>> {
    // parses the whole program followed by 0 or more whitespace chars
    // using many1 so the program needs at least one instr
    let (input, res) = terminated(many1(try_each), multispace0)(input)?;
    Ok((input, res))
}

impl Compiler<String> {
    /// Parses source code from the given input, into an abstract syntax tree.
    pub fn parse(self) -> Result<Compiler<Program>, CompileError> {
        match parse_gdlk(&self.0) {
            // TODO: can probably refactor the parser funcs to use
            // Verbose error to make the errors nicer
            // example: https://github.com/Geal/nom/blob/master/examples/s_expression.rs
            Ok((_, body)) => {
                let prog = Program { body };
                Ok(Compiler(prog))
            }
            Err(nom::Err::Error((_input, e))) => {
                Err(CompileError::ParseError(e.description().to_string()))
            }
            Err(nom::Err::Incomplete(_needed)) => {
                // TODO: better ass
                Err(CompileError::ParseError("ass".to_string()))
            }
            Err(nom::Err::Failure((_input, e))) => {
                Err(CompileError::ParseError(e.description().to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_read_write() {
        assert_eq!(
            parse_gdlk(
                "
            ReAd
            WrItE
        "
            ),
            Ok((
                "",
                vec![
                    Instr::NullaryOp(NullaryOp::Read),
                    Instr::NullaryOp(NullaryOp::Write)
                ]
            ))
        )
    }
    #[test]
    fn test_set() {
        assert_eq!(
            parse_gdlk("Set 4"),
            Ok(("", vec![Instr::ValueOp(ValueOp::Set, 4)]))
        )
    }

    #[test]
    fn test_push() {
        assert_eq!(
            parse_gdlk("Push 4"),
            Ok(("", vec![Instr::StackOp(StackOp::Push, 4)]))
        )
    }

    #[test]
    fn test_pop() {
        assert_eq!(
            parse_gdlk("Pop 4"),
            Ok(("", vec![Instr::StackOp(StackOp::Pop, 4)]))
        )
    }

    #[test]
    fn test_parse_if() {
        assert_eq!(
            parse_gdlk(
                "IF {
            Read
            write
        }"
            ),
            Ok((
                "",
                vec![Instr::If(vec![
                    Instr::NullaryOp(NullaryOp::Read),
                    Instr::NullaryOp(NullaryOp::Write),
                ])]
            ))
        )
    }

    #[test]
    fn test_parse_while() {
        assert_eq!(
            parse_gdlk(
                "WHiLE {
            READ
            Write
        }"
            ),
            Ok((
                "",
                vec![Instr::While(vec![
                    Instr::NullaryOp(NullaryOp::Read),
                    Instr::NullaryOp(NullaryOp::Write),
                ])]
            ))
        )
    }

    #[test]
    fn test_parse_empty_if_and_while() {
        assert_eq!(
            parse_gdlk("while {}if{}"),
            Ok(("", vec![Instr::While(vec![]), Instr::If(vec![])]))
        )
    }

    #[test]
    fn test_parse_simple_file() {
        assert_eq!(
            parse_gdlk(
                "
            Read
            Set 2
            Write
            Read
            Set 3
            Write
            Read
            Set 4
            Write
        "
            ),
            Ok((
                "",
                vec![
                    Instr::NullaryOp(NullaryOp::Read),
                    Instr::ValueOp(ValueOp::Set, 2),
                    Instr::NullaryOp(NullaryOp::Write),
                    Instr::NullaryOp(NullaryOp::Read),
                    Instr::ValueOp(ValueOp::Set, 3),
                    Instr::NullaryOp(NullaryOp::Write),
                    Instr::NullaryOp(NullaryOp::Read),
                    Instr::ValueOp(ValueOp::Set, 4),
                    Instr::NullaryOp(NullaryOp::Write)
                ]
            ))
        )
    }
}
