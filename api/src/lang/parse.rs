use crate::{
    error::{CompileError, CompileErrors},
    lang::{
        ast::{
            Instr, LangValue, Operator, Program, RegisterRef, StackIdentifier,
            UserRegisterIdentifier, ValueSource,
        },
        consts::{REG_INPUT_LEN, REG_STACK_LEN_PREFIX, REG_USER_PREFIX},
        Compiler,
    },
};
use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::{char, digit1, multispace0, one_of},
    combinator::{all_consuming, map, map_res},
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

/// Parses a register identifer, something like "RX0". Does not parse any
/// whitespace around it.
fn reg_ident(input: &str) -> IResult<&str, RegisterRef> {
    let (input, val) = alt((
        // "RLI" => RegisterRef::InputLength
        map(tag_no_case(REG_INPUT_LEN), |_| RegisterRef::InputLength),
        // "RSx" => RegisterRef::StackLength(x)
        preceded(
            tag_no_case(REG_STACK_LEN_PREFIX),
            map_res(digit1, |s: &str| {
                s.parse::<StackIdentifier>().map(RegisterRef::StackLength)
            }),
        ),
        // "RXx" => RegisterRef::User(x)
        preceded(
            tag_no_case(REG_USER_PREFIX),
            map_res(digit1, |s: &str| {
                s.parse::<UserRegisterIdentifier>().map(RegisterRef::User)
            }),
        ),
    ))(input)?;
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
        map(lang_value, ValueSource::Const),
        // "RX1" => ValueSource::Register(1)
        map(reg_ident, ValueSource::Register),
    ))(input)
}

fn parse_read(input: &str) -> IResult<&str, Instr> {
    // input is remaining stuff to parse
    // tuple is output values, we throw away the first two because that's
    // "Read" and the whitespace delim
    // >>> Read RX0
    let (input, (_, _, reg)) =
        tuple((instr("Read"), arg_delim, reg_ident))(input)?;
    Ok((input, Instr::Operator(Operator::Read(reg))))
}

fn parse_write(input: &str) -> IResult<&str, Instr> {
    // >>> Write RX0
    let (input, (_, _, reg)) =
        tuple((instr("Write"), arg_delim, reg_ident))(input)?;
    Ok((input, Instr::Operator(Operator::Write(reg))))
}

fn parse_set(input: &str) -> IResult<&str, Instr> {
    // >>> Set RX0 10
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
    // >>> Add RX0 RX1
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
    // >>> Sub RX0 RX1
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
    // >>> Mul RX0 RX1
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
    // >>> Push RX0 S1
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
    // >>> Pop S1 RX0
    let (input, (_, _, stack, _, reg)) =
        tuple((instr("Pop"), arg_delim, stack_ident, arg_delim, reg_ident))(
            input,
        )?;
    Ok((input, Instr::Operator(Operator::Pop(stack, reg))))
}

fn parse_if(input: &str) -> IResult<&str, Instr> {
    // >>> If RX0 { ... }
    let (input, (_, _, reg)) =
        tuple((instr("If"), arg_delim, reg_ident))(input)?;
    let (input, body) = parse_body(input)?;
    Ok((input, Instr::If(reg, body)))
}

fn parse_while(input: &str) -> IResult<&str, Instr> {
    // >>> While RX0 { ... }
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
    pub fn parse(self) -> Result<Compiler<Program>, CompileErrors> {
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
        .map_err(|err| err.into())
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
                ReAd RX0
                WrItE RX0
                "
            ),
            Ok((
                "",
                vec![
                    Instr::Operator(Operator::Read(RegisterRef::User(0))),
                    Instr::Operator(Operator::Write(RegisterRef::User(0)))
                ]
            ))
        )
    }

    #[test]
    fn test_set_and_registers() {
        assert_eq!(
            parse_gdlk(
                "
                Set RX1 4
                SET RX1 RLI
                SET RX1 RS0
                "
            ),
            Ok((
                "",
                vec![
                    Instr::Operator(Operator::Set(
                        RegisterRef::User(1),
                        ValueSource::Const(4)
                    )),
                    Instr::Operator(Operator::Set(
                        RegisterRef::User(1),
                        ValueSource::Register(RegisterRef::InputLength),
                    )),
                    Instr::Operator(Operator::Set(
                        RegisterRef::User(1),
                        ValueSource::Register(RegisterRef::StackLength(0)),
                    )),
                ]
            ))
        )
    }

    #[test]
    fn test_add() {
        assert_eq!(
            parse_gdlk("Add RX1 RX4"),
            Ok((
                "",
                vec![Instr::Operator(Operator::Add(
                    RegisterRef::User(1),
                    ValueSource::Register(RegisterRef::User(4))
                ))]
            ))
        )
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            parse_gdlk("Sub RX1 RX4"),
            Ok((
                "",
                vec![Instr::Operator(Operator::Sub(
                    RegisterRef::User(1),
                    ValueSource::Register(RegisterRef::User(4))
                ))]
            ))
        )
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            parse_gdlk("Mul rx1 rx0"),
            Ok((
                "",
                vec![Instr::Operator(Operator::Mul(
                    RegisterRef::User(1),
                    ValueSource::Register(RegisterRef::User(0))
                ))]
            ))
        )
    }

    #[test]
    fn test_push() {
        assert_eq!(
            parse_gdlk("Push RX2 S4"),
            Ok((
                "",
                vec![Instr::Operator(Operator::Push(
                    ValueSource::Register(RegisterRef::User(2)),
                    4
                ))]
            ))
        )
    }

    #[test]
    fn test_pop() {
        assert_eq!(
            parse_gdlk("Pop S4 RX2"),
            Ok((
                "",
                vec![Instr::Operator(Operator::Pop(4, RegisterRef::User(2)))]
            ))
        )
    }

    #[test]
    fn test_parse_if() {
        assert_eq!(
            parse_gdlk(
                "IF RX10 {
            Read RX10
            write RX10
        }"
            ),
            Ok((
                "",
                vec![Instr::If(
                    RegisterRef::User(10),
                    vec![
                        Instr::Operator(Operator::Read(RegisterRef::User(10))),
                        Instr::Operator(Operator::Write(RegisterRef::User(10))),
                    ]
                )]
            ))
        )
    }

    #[test]
    fn test_parse_while() {
        assert_eq!(
            parse_gdlk(
                "WHiLE RX0 {
            READ RX0
            Write RX0
        }"
            ),
            Ok((
                "",
                vec![Instr::While(
                    RegisterRef::User(0),
                    vec![
                        Instr::Operator(Operator::Read(RegisterRef::User(0))),
                        Instr::Operator(Operator::Write(RegisterRef::User(0))),
                    ]
                )]
            ))
        )
    }

    #[test]
    fn test_parse_empty_if_and_while() {
        assert_eq!(
            parse_gdlk("while RX0 {}if RX1{}"),
            Ok((
                "",
                vec![
                    Instr::While(RegisterRef::User(0), vec![]),
                    Instr::If(RegisterRef::User(1), vec![])
                ]
            ))
        )
    }

    #[test]
    fn test_parse_simple_file() {
        assert_eq!(
            parse_gdlk(
                "
            Read RX0
            Set RX0 2
            Write RX0
            Read RX1
            Set RX1 3
            Write RX1
            Read RX2
            Set RX2 4
            Write RX2
        "
            ),
            Ok((
                "",
                vec![
                    Instr::Operator(Operator::Read(RegisterRef::User(0))),
                    Instr::Operator(Operator::Set(
                        RegisterRef::User(0),
                        ValueSource::Const(2)
                    )),
                    Instr::Operator(Operator::Write(RegisterRef::User(0))),
                    Instr::Operator(Operator::Read(RegisterRef::User(1))),
                    Instr::Operator(Operator::Set(
                        RegisterRef::User(1),
                        ValueSource::Const(3)
                    )),
                    Instr::Operator(Operator::Write(RegisterRef::User(1))),
                    Instr::Operator(Operator::Read(RegisterRef::User(2))),
                    Instr::Operator(Operator::Set(
                        RegisterRef::User(2),
                        ValueSource::Const(4)
                    )),
                    Instr::Operator(Operator::Write(RegisterRef::User(2)))
                ]
            ))
        )
    }
}
