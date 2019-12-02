use crate::{
    error::CompileError,
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
    character::complete::{alpha1, char, digit1, multispace0, space1},
    combinator::{all_consuming, cut, map, map_res, peek},
    error::{
        context, convert_error, ErrorKind, ParseError, VerboseError,
        VerboseErrorKind,
    },
    lib::std::ops::RangeTo,
    multi::many0,
    sequence::{delimited, preceded, tuple},
    AsChar, Compare, IResult, InputTake, InputTakeAtPosition, Slice,
};

fn one_arg<I, O, E: ParseError<I>, F>(
    arg_parser: F,
) -> impl Fn(I) -> IResult<I, O, E>
where
    I: InputTakeAtPosition,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    F: Fn(I) -> IResult<I, O, E>,
{
    preceded(space1, arg_parser)
}

fn two_args<I, O1, O2, E: ParseError<I>, F, G>(
    arg_parser_one: F,
    arg_parser_two: G,
) -> impl Fn(I) -> IResult<I, (O1, O2), E>
where
    I: InputTakeAtPosition + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    F: Fn(I) -> IResult<I, O1, E>,
    G: Fn(I) -> IResult<I, O2, E>,
{
    tuple((one_arg(arg_parser_one), one_arg(arg_parser_two)))
}

/// Parses one instruction keyword, and uses the passed parser to parse the
/// arguments
fn instr<'a, Input: 'a, O, F, Error: ParseError<Input>>(
    instr: &'static str,
    arg_parser: F,
) -> impl Fn(Input) -> IResult<Input, O, Error>
where
    Input: InputTake + Clone + Compare<&'static str> + Slice<RangeTo<usize>>,
    F: Fn(Input) -> IResult<Input, O, Error>,
{
    preceded(
        context(instr, tag_no_case(instr)),
        context(instr, cut(arg_parser)),
    )
}

/// Parses a register identifer, something like "RX0". Does not parse any
/// whitespace around it.
fn reg_ident(input: &str) -> IResult<&str, RegisterRef, VerboseError<&str>> {
    let (input, val) = context(
        "Register",
        alt((
            // "RLI" => RegisterRef::InputLength
            map(tag_no_case(REG_INPUT_LEN), |_| RegisterRef::InputLength),
            // "RSx" => RegisterRef::StackLength(x)
            preceded(
                tag_no_case(REG_STACK_LEN_PREFIX),
                cut(map_res(digit1, |s: &str| {
                    s.parse::<StackIdentifier>().map(RegisterRef::StackLength)
                })),
            ),
            // "RXx" => RegisterRef::User(x)
            preceded(
                tag_no_case(REG_USER_PREFIX),
                cut(map_res(digit1, |s: &str| {
                    s.parse::<UserRegisterIdentifier>().map(RegisterRef::User)
                })),
            ),
        )),
    )(input)?;
    Ok((input, val))
}

/// Parses a stack identifier, like "S1". Does not parse any whitespace around
/// it.
fn stack_ident(
    input: &str,
) -> IResult<&str, StackIdentifier, VerboseError<&str>> {
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
fn lang_value(input: &str) -> IResult<&str, LangValue, VerboseError<&str>> {
    map_res(digit1, |s: &str| s.parse::<LangValue>())(input)
}

/// Parses either a `LangValue` or `Register`.
fn parse_value_source(
    input: &str,
) -> IResult<&str, ValueSource, VerboseError<&str>> {
    alt((
        // "1" => ValueSource::Const(1)
        map(lang_value, ValueSource::Const),
        // "RX1" => ValueSource::Register(1)
        map(reg_ident, ValueSource::Register),
    ))(input)
}

fn parse_read(input: &str) -> IResult<&str, Instr, VerboseError<&str>> {
    // input is remaining stuff to parse
    // >>> Read RX0
    let (input, reg) = instr("Read", one_arg(reg_ident))(input)?;
    Ok((input, Instr::Operator(Operator::Read(reg))))
}

fn parse_write(input: &str) -> IResult<&str, Instr, VerboseError<&str>> {
    // >>> Write RX0
    let (input, reg) = instr("Write", one_arg(reg_ident))(input)?;
    Ok((input, Instr::Operator(Operator::Write(reg))))
}

fn parse_set(input: &str) -> IResult<&str, Instr, VerboseError<&str>> {
    // >>> Set RX0 10
    let (input, (reg, src)) =
        instr("Set", two_args(reg_ident, parse_value_source))(input)?;
    Ok((input, Instr::Operator(Operator::Set(reg, src))))
}

fn parse_add(input: &str) -> IResult<&str, Instr, VerboseError<&str>> {
    // >>> Add RX0 RX1
    let (input, (dst, src)) =
        instr("Add", two_args(reg_ident, parse_value_source))(input)?;
    Ok((input, Instr::Operator(Operator::Add(dst, src))))
}

fn parse_sub(input: &str) -> IResult<&str, Instr, VerboseError<&str>> {
    // >>> Sub RX0 RX1
    let (input, (dst, src)) =
        instr("Sub", two_args(reg_ident, parse_value_source))(input)?;
    Ok((input, Instr::Operator(Operator::Sub(dst, src))))
}

fn parse_mul(input: &str) -> IResult<&str, Instr, VerboseError<&str>> {
    // >>> Mul RX0 RX1
    let (input, (dst, src)) =
        instr("Mul", two_args(reg_ident, parse_value_source))(input)?;
    Ok((input, Instr::Operator(Operator::Mul(dst, src))))
}

fn parse_push(input: &str) -> IResult<&str, Instr, VerboseError<&str>> {
    // >>> Push RX0 S1
    let (input, (src, stack)) =
        instr("Push", two_args(parse_value_source, stack_ident))(input)?;
    Ok((input, Instr::Operator(Operator::Push(src, stack))))
}

fn parse_pop(input: &str) -> IResult<&str, Instr, VerboseError<&str>> {
    // >>> Pop S1 RX0
    let (input, (stack, reg)) =
        instr("POP", two_args(stack_ident, reg_ident))(input)?;
    Ok((input, Instr::Operator(Operator::Pop(stack, reg))))
}

fn parse_if(input: &str) -> IResult<&str, Instr, VerboseError<&str>> {
    // >>> If RX0 { ... }
    let (input, reg) = instr("If", one_arg(reg_ident))(input)?;
    let (input, body) = parse_body(input)?;
    Ok((input, Instr::If(reg, body)))
}

fn parse_while(input: &str) -> IResult<&str, Instr, VerboseError<&str>> {
    // >>> While RX0 { ... }
    let (input, reg) = instr("While", one_arg(reg_ident))(input)?;
    let (input, body) = parse_body(input)?;
    Ok((input, Instr::While(reg, body)))
}

fn try_each(input: &str) -> IResult<&str, Instr, VerboseError<&str>> {
    let (input, (_, res, _)) = tuple((
        multispace0,
        context(
            "Instruction",
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
        ),
        multispace0,
    ))(input)?;
    Ok((input, res))
}

// Parse the body of an if or while statement
//
// something like (\s*{<BODY>\s*})
fn parse_body(input: &str) -> IResult<&str, Vec<Instr>, VerboseError<&str>> {
    // multispace0 matches 0 or more whitespace chars (including new lines)
    let (input, res) = cut(delimited(
        preceded(multispace0, char('{')),
        many0(try_each), /* many0 will match 0 more, so the body could
                          * be empty */
        preceded(multispace0, char('}')),
    ))(input)?;
    Ok((input, res))
}

fn parse_gdlk(input: &str) -> IResult<&str, Vec<Instr>, VerboseError<&str>> {
    // parses the whole program followed by 0 or more whitespace chars

    // consume starting whitespace
    let (input, _) = multispace0(input)?;
    // make sure something is there but don't consume the input
    // TODO: make this error message nicer
    peek(alpha1)(input)?;
    let (input, res) = all_consuming(many0(try_each))(input)?;
    Ok((input, res))
}

impl Compiler<String> {
    /// Parses source code from the given input, into an abstract syntax tree.
    pub fn parse(self) -> Result<Compiler<Program>, CompileError> {
        let input_str = &self.0;
        match parse_gdlk(input_str) {
            Ok((_, body)) => {
                let prog = Program { body };
                Ok(Compiler(prog))
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                match e.errors.as_slice() {
                    [(substring, VerboseErrorKind::Nom(ErrorKind::Eof)), ..] => {
                        // If the error is EOF that means there was remaining
                        // input that was not parsed
                        // so they put in a bad keyword
                        // TODO: need to make this custom error look more like
                        // how convert_error outputs
                        Err(CompileError::ParseError(format!(
                            "Invalid keyword: {}",
                            substring
                        )))
                    }
                    _ => Err(CompileError::ParseError(convert_error(
                        &input_str, e,
                    ))),
                }
            }
            Err(nom::Err::Incomplete(_needed)) => {
                // TODO: better ass
                Err(CompileError::ParseError("ass".to_string()))
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
