use crate::{
    error::CompileError,
    lang::{
        ast::{
            Instr, LangValue, Operator, Program, Register, StackIdentifier,
            ValueSource,
        },
        Compiler,
    },
};
use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::{char, digit1, multispace0, one_of},
    combinator::{all_consuming, map_res},
    multi::many0,
    sequence::{delimited, preceded, tuple},
    Compare, IResult, InputLength, InputTake,
};

fn arg_delim(input: &str) -> IResult<&str, char> {
    one_of(" \t")(input)
}

/// Parses one instruction keyword, not including its arguments. This type
/// signature was stolen from tag_no_case. This is just a wrapper around
/// tag_no_case, in case we want to make parsing case senstitive or something
/// in the future.
fn instr<T, Input>(instr: T) -> impl Fn(Input) -> IResult<Input, Input>
where
    Input: InputTake + Compare<T>,
    T: InputLength + Clone,
{
    tag_no_case(instr)
}

/// Parses a register identifer, something like "R0". Does not parse any
/// whitespace around it.
fn reg_ident(input: &str) -> IResult<&str, Register> {
    let (input, val) = preceded(
        tag_no_case("R"),
        map_res(digit1, |s: &str| s.parse::<Register>()),
    )(input)?;
    Ok((input, val))
}

/// Parses a stack identifier, like "S1". Does not parse any whitespace around
/// it.
fn stack_ident(input: &str) -> IResult<&str, StackIdentifier> {
    let (input, val) = preceded(
        multispace0,
        preceded(
            tag_no_case("S"),
            map_res(digit1, |s: &str| s.parse::<StackIdentifier>()),
        ),
    )(input)?;
    Ok((input, val))
}

/// Parses a `LangValue`, like "10" or "-3", not including any surrounding
/// whitespace. (Negatives don't actually work yet).
fn lang_value(input: &str) -> IResult<&str, LangValue> {
    map_res(digit1, |s: &str| s.parse::<LangValue>())(input)
}

/// Parses either a `LangValue` or `Register`.
fn parse_value_source(input: &str) -> IResult<&str, ValueSource> {
    alt((
        // "1" => ValueSource::Const(1)
        map_res(lang_value, |val: LangValue| -> Result<ValueSource, ()> {
            Ok(ValueSource::Const(val))
        }),
        // R1 => ValueSource::Register(1)
        map_res(reg_ident, |reg: Register| -> Result<ValueSource, ()> {
            Ok(ValueSource::Register(reg))
        }),
    ))(input)
}

fn parse_read(input: &str) -> IResult<&str, Instr> {
    // input is remaining stuff to parse
    // tuple is output values, we throw away the first two because that's
    // "Read" and the whitespace delim
    // >>> Read R0
    let (input, (_, _, reg)) =
        tuple((instr("Read"), arg_delim, reg_ident))(input)?;
    Ok((input, Instr::Operator(Operator::Read(reg))))
}

fn parse_write(input: &str) -> IResult<&str, Instr> {
    // >>> Write R0
    let (input, (_, _, reg)) =
        tuple((instr("Write"), arg_delim, reg_ident))(input)?;
    Ok((input, Instr::Operator(Operator::Write(reg))))
}

fn parse_set(input: &str) -> IResult<&str, Instr> {
    // >>> Set R0 10
    let (input, (_, _, reg, _, src)) = tuple((
        instr("Set"),
        arg_delim,
        reg_ident,
        arg_delim,
        parse_value_source,
    ))(input)?;
    Ok((input, Instr::Operator(Operator::Set(reg, src))))
}

fn parse_add(input: &str) -> IResult<&str, Instr> {
    // >>> Add R0 R1
    let (input, (_, _, dst, _, src)) = tuple((
        instr("Add"),
        arg_delim,
        reg_ident,
        arg_delim,
        parse_value_source,
    ))(input)?;
    Ok((input, Instr::Operator(Operator::Add(dst, src))))
}

fn parse_sub(input: &str) -> IResult<&str, Instr> {
    // >>> Sub R0 R1
    let (input, (_, _, dst, _, src)) = tuple((
        instr("Sub"),
        arg_delim,
        reg_ident,
        arg_delim,
        parse_value_source,
    ))(input)?;
    Ok((input, Instr::Operator(Operator::Sub(dst, src))))
}

fn parse_mul(input: &str) -> IResult<&str, Instr> {
    // >>> Mul R0 R1
    let (input, (_, _, dst, _, src)) = tuple((
        instr("Mul"),
        arg_delim,
        reg_ident,
        arg_delim,
        parse_value_source,
    ))(input)?;
    Ok((input, Instr::Operator(Operator::Mul(dst, src))))
}

fn parse_push(input: &str) -> IResult<&str, Instr> {
    // >>> Push R0 S1
    let (input, (_, _, src, _, stack)) = tuple((
        instr("Push"),
        arg_delim,
        parse_value_source,
        arg_delim,
        stack_ident,
    ))(input)?;
    Ok((input, Instr::Operator(Operator::Push(src, stack))))
}

fn parse_pop(input: &str) -> IResult<&str, Instr> {
    // >>> Pop S1 R0
    let (input, (_, _, stack, _, reg)) =
        tuple((instr("Pop"), arg_delim, stack_ident, arg_delim, reg_ident))(
            input,
        )?;
    Ok((input, Instr::Operator(Operator::Pop(stack, reg))))
}

fn parse_if(input: &str) -> IResult<&str, Instr> {
    // >>> If R0 { ... }
    let (input, (_, _, reg)) =
        tuple((instr("If"), arg_delim, reg_ident))(input)?;
    let (input, body) = parse_body(input)?;
    Ok((input, Instr::If(reg, body)))
}

fn parse_while(input: &str) -> IResult<&str, Instr> {
    // >>> While R0 { ... }
    let (input, (_, _, reg)) =
        tuple((instr("While"), arg_delim, reg_ident))(input)?;
    let (input, body) = parse_body(input)?;
    Ok((input, Instr::While(reg, body)))
}

fn try_each(input: &str) -> IResult<&str, Instr> {
    let (input, (_, res, _)) = tuple((
        multispace0,
        alt((
            parse_read,
            parse_write,
            parse_set,
            parse_add,
            parse_sub,
            parse_mul,
            parse_push,
            parse_pop,
            parse_if,
            parse_while,
        )),
        multispace0,
    ))(input)?;
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
    let (input, res) = all_consuming(many0(try_each))(input)?;
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
            ReAd R0
            WrItE R0
        "
            ),
            Ok((
                "",
                vec![
                    Instr::Operator(Operator::Read(0)),
                    Instr::Operator(Operator::Write(0))
                ]
            ))
        )
    }

    #[test]
    fn test_set() {
        assert_eq!(
            parse_gdlk("Set R1 4"),
            Ok((
                "",
                vec![Instr::Operator(Operator::Set(1, ValueSource::Const(4)))]
            ))
        )
    }

    #[test]
    fn test_add() {
        assert_eq!(
            parse_gdlk("Add R1 R4"),
            Ok((
                "",
                vec![Instr::Operator(Operator::Add(
                    1,
                    ValueSource::Register(4)
                ))]
            ))
        )
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            parse_gdlk("Sub R1 R4"),
            Ok((
                "",
                vec![Instr::Operator(Operator::Sub(
                    1,
                    ValueSource::Register(4)
                ))]
            ))
        )
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            parse_gdlk("Mul r1 r0"),
            Ok((
                "",
                vec![Instr::Operator(Operator::Mul(
                    1,
                    ValueSource::Register(0)
                ))]
            ))
        )
    }

    #[test]
    fn test_push() {
        assert_eq!(
            parse_gdlk("Push R2 S4"),
            Ok((
                "",
                vec![Instr::Operator(Operator::Push(
                    ValueSource::Register(2),
                    4
                ))]
            ))
        )
    }

    #[test]
    fn test_pop() {
        assert_eq!(
            parse_gdlk("Pop S4 R2"),
            Ok(("", vec![Instr::Operator(Operator::Pop(4, 2))]))
        )
    }

    #[test]
    fn test_parse_if() {
        assert_eq!(
            parse_gdlk(
                "IF R10 {
            Read R10
            write R10
        }"
            ),
            Ok((
                "",
                vec![Instr::If(
                    10,
                    vec![
                        Instr::Operator(Operator::Read(10)),
                        Instr::Operator(Operator::Write(10)),
                    ]
                )]
            ))
        )
    }

    #[test]
    fn test_parse_while() {
        assert_eq!(
            parse_gdlk(
                "WHiLE R0 {
            READ R0
            Write R0
        }"
            ),
            Ok((
                "",
                vec![Instr::While(
                    0,
                    vec![
                        Instr::Operator(Operator::Read(0)),
                        Instr::Operator(Operator::Write(0)),
                    ]
                )]
            ))
        )
    }

    #[test]
    fn test_parse_empty_if_and_while() {
        assert_eq!(
            parse_gdlk("while R0 {}if R1{}"),
            Ok(("", vec![Instr::While(0, vec![]), Instr::If(1, vec![])]))
        )
    }

    #[test]
    fn test_parse_simple_file() {
        assert_eq!(
            parse_gdlk(
                "
            Read R0
            Set R0 2
            Write R0
            Read R1
            Set R1 3
            Write R1
            Read R2
            Set R2 4
            Write R2
        "
            ),
            Ok((
                "",
                vec![
                    Instr::Operator(Operator::Read(0)),
                    Instr::Operator(Operator::Set(0, ValueSource::Const(2))),
                    Instr::Operator(Operator::Write(0)),
                    Instr::Operator(Operator::Read(1)),
                    Instr::Operator(Operator::Set(1, ValueSource::Const(3))),
                    Instr::Operator(Operator::Write(1)),
                    Instr::Operator(Operator::Read(2)),
                    Instr::Operator(Operator::Set(2, ValueSource::Const(4))),
                    Instr::Operator(Operator::Write(2))
                ]
            ))
        )
    }
}
